//! The JSON the pages read: the session, the overview, the skills of sync and push, the vault with a card
//! per skill, the history, the doctor and the settings. Nothing here writes.

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Serialize;

use super::{ApiError, Shared, blocking};
use crate::server::state::{AppState, Snapshot};
use crate::server::views::{SkillRow, project_label, push_rows, root_of, sync_rows};

/// The snapshot, or the reason there is none, as an answer.
async fn snapshot(state: &Arc<AppState>) -> Result<Arc<Snapshot>, ApiError> {
    let shared = Arc::clone(state);
    blocking(move || shared.snapshot())
        .await?
        .map_err(|message| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, message))
}

#[derive(Serialize)]
pub(crate) struct SessionView {
    allow_write: bool,
    version: &'static str,
}

pub(crate) async fn session(State(state): Shared) -> Json<SessionView> {
    Json(SessionView {
        allow_write: state.config.allow_write,
        version: env!("CARGO_PKG_VERSION"),
    })
}

#[derive(Serialize)]
struct Outdated {
    skill: String,
    added: usize,
    changed: usize,
    removed: usize,
}

#[derive(Serialize)]
struct Problem {
    name: String,
    message: String,
    hint: Option<String>,
}

#[derive(Serialize)]
struct ProjectView {
    path: String,
    label: String,
    /// `in_sync`, `drift` or `problem`.
    state: &'static str,
    current: usize,
    outdated: Vec<Outdated>,
    missing: Vec<String>,
    missing_links: Vec<String>,
    not_in_vault: Vec<String>,
    problems: Vec<Problem>,
    link_problems: Vec<Problem>,
}

#[derive(Serialize)]
struct Issue {
    path: String,
    message: String,
}

#[derive(Serialize)]
pub(crate) struct OverviewView {
    at: String,
    vault: String,
    root: String,
    projects: Vec<ProjectView>,
    in_sync: usize,
    drift: usize,
    problems: usize,
    issues: Vec<Issue>,
}

fn problem(name: &str, message: &str, hint: Option<&String>) -> Problem {
    Problem {
        name: name.to_string(),
        message: message.to_string(),
        hint: hint.cloned(),
    }
}

pub(crate) async fn overview(State(state): Shared) -> Result<Json<OverviewView>, ApiError> {
    let snapshot = snapshot(&state).await?;
    let root = root_of(&snapshot);
    let projects: Vec<ProjectView> = snapshot
        .status
        .projects
        .iter()
        .map(|p| {
            let trouble = !p.failed.is_empty() || !p.project_problems.is_empty();
            ProjectView {
                path: p.project.display().to_string(),
                label: project_label(root, &p.project),
                state: if p.drifts() {
                    "drift"
                } else if trouble {
                    "problem"
                } else {
                    "in_sync"
                },
                current: p.up_to_date.len(),
                outdated: p
                    .outdated
                    .iter()
                    .map(|o| Outdated {
                        skill: o.skill.clone(),
                        added: o.added,
                        changed: o.changed,
                        removed: o.removed,
                    })
                    .collect(),
                missing: p.missing_mandatory.clone(),
                missing_links: p.missing_bridges.clone(),
                not_in_vault: p.not_in_vault.clone(),
                problems: p
                    .failed
                    .iter()
                    .map(|f| problem(&f.skill, &f.message, f.hint.as_ref()))
                    .collect(),
                link_problems: p
                    .project_problems
                    .iter()
                    .map(|f| problem(&f.skill, &f.message, f.hint.as_ref()))
                    .collect(),
            }
        })
        .collect();
    let drift = projects.iter().filter(|p| p.state == "drift").count();
    let problems = projects.iter().filter(|p| p.state == "problem").count();
    Ok(Json(OverviewView {
        at: snapshot.at.clone(),
        vault: snapshot.workspace.vault.path.display().to_string(),
        root: root.display().to_string(),
        in_sync: projects.len() - drift - problems,
        drift,
        problems,
        projects,
        issues: snapshot
            .scan
            .issues
            .iter()
            .map(|i| Issue {
                path: i.path.display().to_string(),
                message: i.message.clone(),
            })
            .collect(),
    }))
}

#[derive(Serialize)]
struct SkillRowView {
    name: String,
    group: String,
    mandatory: bool,
    current: usize,
    outdated: usize,
    missing: usize,
    problems: usize,
}

#[derive(Serialize)]
struct ProjectChoice {
    path: String,
    label: String,
}

#[derive(Serialize)]
pub(crate) struct SkillsView {
    rows: Vec<SkillRowView>,
    projects: Vec<ProjectChoice>,
}

fn skills_view(snapshot: &Snapshot, rows: Vec<SkillRow>) -> SkillsView {
    let root = root_of(snapshot);
    SkillsView {
        rows: rows
            .into_iter()
            .map(|r| SkillRowView {
                name: r.name,
                group: r.group,
                mandatory: r.mandatory,
                current: r.current,
                outdated: r.outdated,
                missing: r.missing,
                problems: r.problems,
            })
            .collect(),
        projects: snapshot
            .status
            .projects
            .iter()
            .map(|p| ProjectChoice {
                path: p.project.display().to_string(),
                label: project_label(root, &p.project),
            })
            .collect(),
    }
}

pub(crate) async fn sync_skills(State(state): Shared) -> Result<Json<SkillsView>, ApiError> {
    let snapshot = snapshot(&state).await?;
    Ok(Json(skills_view(&snapshot, sync_rows(&snapshot))))
}

pub(crate) async fn push_skills(State(state): Shared) -> Result<Json<SkillsView>, ApiError> {
    let snapshot = snapshot(&state).await?;
    Ok(Json(skills_view(&snapshot, push_rows(&snapshot))))
}
