//! The JSON of the vault page, the history, the doctor and the settings.

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Serialize;
use skillmirror_core::config::Source;
use skillmirror_core::ops::{self, Severity, SkillInfo};

use super::{ApiError, Shared, blocking};
use crate::server::state::Snapshot;
use crate::server::views::{project_label, root_of};

#[derive(Serialize)]
struct Where {
    project: String,
    /// `current`, `outdated`, `missing`, `not_in_vault` or `problem`.
    state: &'static str,
    detail: Option<String>,
}

#[derive(Serialize)]
struct VaultSkill {
    name: String,
    group: String,
    mandatory: bool,
    description: Option<String>,
    header_problem: Option<String>,
    files: Vec<String>,
    untracked: Vec<String>,
    profiles: Vec<String>,
    projects: Vec<Where>,
}

#[derive(Serialize)]
pub(crate) struct VaultView {
    skills: Vec<VaultSkill>,
    /// Folders that projects have and the vault does not.
    foreign: Vec<VaultSkill>,
}

fn counted(added: usize, changed: usize, removed: usize) -> String {
    let mut parts = Vec::new();
    for (count, what) in [(added, "added"), (changed, "changed"), (removed, "removed")] {
        if count > 0 {
            parts.push(format!("{count} {what}"));
        }
    }
    parts.join(", ")
}

/// What each skill name is in each project, from the comparison the snapshot holds.
fn places(snapshot: &Snapshot) -> BTreeMap<String, Vec<Where>> {
    let root = root_of(snapshot);
    let mut by_skill: BTreeMap<String, Vec<Where>> = BTreeMap::new();
    let mut put = |skill: &str, project: &std::path::Path, state, detail| {
        by_skill.entry(skill.to_string()).or_default().push(Where {
            project: project_label(root, project),
            state,
            detail,
        });
    };
    for p in &snapshot.status.projects {
        for name in &p.up_to_date {
            put(name, &p.project, "current", None);
        }
        for o in &p.outdated {
            put(
                &o.skill,
                &p.project,
                "outdated",
                Some(counted(o.added, o.changed, o.removed)),
            );
        }
        for name in &p.missing_mandatory {
            put(name, &p.project, "missing", None);
        }
        for name in &p.not_in_vault {
            put(name, &p.project, "not_in_vault", None);
        }
        for f in &p.failed {
            put(&f.skill, &p.project, "problem", Some(f.message.clone()));
        }
    }
    by_skill
}

fn vault_skill(info: SkillInfo, profiles: Vec<String>, projects: Vec<Where>) -> VaultSkill {
    VaultSkill {
        name: info.name,
        group: info.group.join("/"),
        mandatory: info.mandatory,
        description: info.description,
        header_problem: info.header_problem,
        files: info.files,
        untracked: info.untracked,
        profiles,
        projects,
    }
}

pub(crate) async fn vault(State(state): Shared) -> Result<Json<VaultView>, ApiError> {
    let shared = Arc::clone(&state);
    let view = blocking(move || {
        let snapshot = shared.snapshot()?;
        let mut places = places(&snapshot);
        let workspace = &snapshot.workspace;
        let mut skills: Vec<VaultSkill> = workspace
            .vault
            .skills
            .values()
            .map(|skill| {
                let info = ops::skill_info(workspace, skill);
                let profiles = ops::profiles_of(workspace, &skill.name);
                vault_skill(
                    info,
                    profiles,
                    places.remove(&skill.name).unwrap_or_default(),
                )
            })
            .collect();
        skills.sort_by(|a, b| a.group.cmp(&b.group).then_with(|| a.name.cmp(&b.name)));
        // What is left in `places` are names the vault does not have.
        let foreign = places
            .into_iter()
            .map(|(name, projects)| VaultSkill {
                name,
                group: String::new(),
                mandatory: false,
                description: None,
                header_problem: None,
                files: Vec::new(),
                untracked: Vec::new(),
                profiles: Vec::new(),
                projects,
            })
            .collect();
        Ok::<_, String>(VaultView { skills, foreign })
    })
    .await?
    .map_err(|message| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, message))?;
    Ok(Json(view))
}

#[derive(Serialize)]
struct RunView {
    id: String,
    date: String,
    command: String,
    skills: usize,
    projects: usize,
    error: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct HistoryView {
    runs: Vec<RunView>,
}

pub(crate) async fn history(State(state): Shared) -> Result<Json<HistoryView>, ApiError> {
    let dirs = state.config.dirs.clone();
    let runs = blocking(move || ops::list_runs(&dirs))
        .await?
        .map_err(|message| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, message))?;
    Ok(Json(HistoryView {
        runs: runs
            .into_iter()
            .map(|r| RunView {
                id: r.id,
                date: r.date,
                command: r.command,
                skills: r.skills,
                projects: r.projects,
                error: r.error,
            })
            .collect(),
    }))
}

#[derive(Serialize)]
struct FindingView {
    severity: &'static str,
    check: String,
    subject: Option<String>,
    message: String,
    hint: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct DoctorView {
    findings: Vec<FindingView>,
}

pub(crate) async fn doctor(State(state): Shared) -> Result<Json<DoctorView>, ApiError> {
    let config = state.config.clone();
    let report =
        blocking(move || ops::doctor(&config.flags, Ok(config.env.clone()), &config.dirs)).await?;
    Ok(Json(DoctorView {
        findings: report
            .findings
            .iter()
            .map(|f| FindingView {
                severity: match f.severity {
                    Severity::Note => "note",
                    Severity::Warning => "warning",
                    Severity::Error => "error",
                },
                check: f.check.to_string(),
                subject: f.subject.clone(),
                message: f.message.clone(),
                hint: f.hint.clone(),
            })
            .collect(),
    }))
}

#[derive(Serialize)]
struct Located {
    path: String,
    source: &'static str,
}

#[derive(Serialize)]
struct ProfileView {
    name: String,
    description: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct SettingsView {
    vault: Option<Located>,
    root: Option<Located>,
    mandatory: Vec<String>,
    targets: Vec<String>,
    exclude_dirs: Vec<String>,
    exclude_paths: Vec<String>,
    profiles: Vec<ProfileView>,
    allow_write: bool,
}

fn source_name(source: Source) -> &'static str {
    match source {
        Source::Flag => "flag",
        Source::Env => "environment",
        Source::PointerFile => "pointer file",
        Source::VaultConfig => "vault config",
    }
}

pub(crate) async fn settings(State(state): Shared) -> Result<Json<SettingsView>, ApiError> {
    let settings = state
        .config
        .settings()
        .map_err(|message| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, message))?;
    let located = |found: Result<&skillmirror_core::config::Sourced<std::path::PathBuf>, _>| {
        found.ok().map(|v| Located {
            path: v.value.display().to_string(),
            source: source_name(v.source),
        })
    };
    let config = settings.config();
    Ok(Json(SettingsView {
        vault: located(settings.vault()),
        root: located(settings.root()),
        mandatory: config.mandatory.clone(),
        targets: config.targets.clone(),
        exclude_dirs: config.exclude_dirs.clone(),
        exclude_paths: config
            .exclude_paths
            .iter()
            .map(|p| p.display().to_string())
            .collect(),
        profiles: config
            .profiles
            .iter()
            .map(|(name, p)| ProfileView {
                name: name.clone(),
                description: p.description.clone(),
            })
            .collect(),
        allow_write: state.config.allow_write,
    }))
}
