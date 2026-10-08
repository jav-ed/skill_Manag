//! Making a plan: what a sync or push would write, shown with its diffs and kept for the go-ahead.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use skillmirror_core::backup::RunKind;
use skillmirror_core::ops::{
    DiffKind, Scope, SkillDiff, Skipped, diff_of_plan, plan_push_scoped, plan_sync_scoped,
    resolve_names,
};
use skillmirror_core::plan::{Plan, PlanKind};

use super::{ApiError, Shared, blocking, new_id, writable};
use crate::server::plans::Stored;
use crate::server::state::{AppState, lock};
use crate::server::views::{project_label, root_of};

#[derive(Deserialize)]
pub(crate) struct PlanRequest {
    kind: String,
    skills: Vec<String>,
    #[serde(default)]
    project: Option<String>,
}

/// One file of a plan row, as the page shows it.
#[derive(Serialize)]
pub(crate) struct DiffFile {
    path: String,
    /// `added`, `changed`, `mode` or `removed`.
    kind: &'static str,
    added: usize,
    removed: usize,
    /// The unified diff; empty when the lines are not shown.
    text: String,
    /// Why the lines are not shown, or that they were cut.
    note: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct PlanRow {
    project: String,
    skill: String,
    /// `create`, `update`, `unchanged` or `failed`.
    action: &'static str,
    detail: String,
    files: Vec<DiffFile>,
}

#[derive(Serialize)]
pub(crate) struct PlanView {
    plan: String,
    kind: String,
    rows: Vec<PlanRow>,
    create: usize,
    update: usize,
    unchanged: usize,
    failed: usize,
    projects: usize,
}

pub(crate) async fn plan(
    State(state): Shared,
    Json(request): Json<PlanRequest>,
) -> Result<Json<PlanView>, ApiError> {
    writable(&state)?;
    let kind = match request.kind.as_str() {
        "sync" => RunKind::Sync,
        "push" => RunKind::Push,
        other => {
            return Err(ApiError::new(
                StatusCode::BAD_REQUEST,
                format!("{other:?} is not something to plan; use sync or push"),
            ));
        }
    };
    if request.skills.is_empty() {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "Pick at least one skill.",
        ));
    }
    let id = new_id()?;
    let shared = Arc::clone(&state);
    let (view, plan) = blocking(move || make_plan(&shared, kind, &request, id)).await??;
    lock(&state.plans).put(view.plan.clone(), Stored::Write { kind, plan });
    Ok(Json(view))
}

/// Looks at the disk now and plans exactly the skills asked for.
fn make_plan(
    state: &AppState,
    kind: RunKind,
    request: &PlanRequest,
    id: String,
) -> Result<(PlanView, Plan), ApiError> {
    let snapshot = state.refresh().map_err(ApiError::engine)?;
    let skills: BTreeSet<String> = resolve_names(&snapshot.workspace.vault, &request.skills)
        .map_err(|e| ApiError::engine(skillmirror_core::describe(&e)))?;
    let scope = Scope {
        skills: Some(skills),
        project: request.project.as_deref().map(PathBuf::from),
    };
    let made = if kind == RunKind::Push {
        plan_push_scoped(&snapshot.workspace, &snapshot.scan, &scope)
    } else {
        plan_sync_scoped(&snapshot.workspace, &snapshot.scan, &scope)
            .map_err(skillmirror_core::Error::from)
    };
    let plan = made.map_err(|e| ApiError::engine(skillmirror_core::describe(&e)))?;
    let view = summarize(&plan, root_of(&snapshot), kind, id);
    Ok((view, plan))
}

/// Lines of one file's diff sent to the page; the rest is counted, not sent.
const DIFF_LINES: usize = 300;

fn diff_files(diff: &SkillDiff) -> Vec<DiffFile> {
    diff.files
        .iter()
        .map(|file| {
            let (text, note) = match &file.skipped {
                Some(Skipped::Binary) => (String::new(), Some("binary file".to_string())),
                Some(Skipped::TooLarge) => (String::new(), Some("too large to show".to_string())),
                Some(Skipped::Unreadable(why)) => {
                    (String::new(), Some(format!("unreadable: {why}")))
                }
                None => cut(&file.text),
            };
            DiffFile {
                path: file.path.display().to_string(),
                kind: match file.kind {
                    DiffKind::Added => "added",
                    DiffKind::Modified => "changed",
                    DiffKind::ModeChanged => "mode",
                    DiffKind::Removed => "removed",
                },
                added: file.added_lines,
                removed: file.removed_lines,
                text,
                note,
            }
        })
        .collect()
}

#[cfg(test)]
pub(crate) fn cut_for_tests(text: &str) -> (String, Option<String>) {
    cut(text)
}

/// The first lines of a diff, with a note when there are more.
fn cut(text: &str) -> (String, Option<String>) {
    let total = text.lines().count();
    if total <= DIFF_LINES {
        return (text.to_string(), None);
    }
    let shown: Vec<&str> = text.lines().take(DIFF_LINES).collect();
    (
        shown.join("\n"),
        Some(format!("{} more lines are not shown", total - DIFF_LINES)),
    )
}

fn summarize(plan: &Plan, root: &std::path::Path, kind: RunKind, id: String) -> PlanView {
    let counts = plan.counts();
    let diffs = diff_of_plan(plan);
    let mut projects = BTreeSet::new();
    let rows = plan
        .entries
        .iter()
        .map(|entry| {
            projects.insert(&entry.target.project);
            let (action, detail) = match &entry.result {
                Ok(skill) => match skill.kind {
                    PlanKind::Create => ("create", format!("{} files", skill.file_count())),
                    PlanKind::Update => (
                        "update",
                        format!(
                            "{} changed, {} removed",
                            skill.changes.len(),
                            skill.removed.len()
                        ),
                    ),
                    PlanKind::Unchanged => ("unchanged", String::new()),
                },
                Err(error) => ("failed", skillmirror_core::describe(error)),
            };
            let files = diffs
                .iter()
                .find(|d| d.target == entry.target)
                .map(diff_files)
                .unwrap_or_default();
            PlanRow {
                project: project_label(root, &entry.target.project),
                skill: entry.target.skill.clone(),
                action,
                detail,
                files,
            }
        })
        .collect();
    PlanView {
        plan: id,
        kind: kind.name().to_string(),
        rows,
        create: counts.create,
        update: counts.update,
        unchanged: counts.unchanged,
        failed: counts.failed,
        projects: projects.len(),
    }
}
