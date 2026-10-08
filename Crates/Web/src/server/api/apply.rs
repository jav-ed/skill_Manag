//! Applying a stored plan as a job, and watching the job.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use skillmirror_core::apply::{Applied, Failure, Outcome};
use skillmirror_core::backup::RunKind;
use skillmirror_core::events::Event;
use skillmirror_core::ops::run_plan;
use skillmirror_core::plan::Plan;

use super::undo::undo;
use super::{ApiError, Shared, new_id, writable};
use crate::server::jobs::{Finished, Line, View};
use crate::server::plans::Stored;
use crate::server::state::{AppState, lock};
use crate::server::views::{project_label, root_of};

#[derive(Deserialize)]
pub(crate) struct ApplyRequest {
    plan: String,
}

#[derive(Serialize)]
pub(crate) struct Started {
    job: String,
}

pub(crate) async fn apply(
    State(state): Shared,
    Json(request): Json<ApplyRequest>,
) -> Result<Json<Started>, ApiError> {
    writable(&state)?;
    let stored = lock(&state.plans).take(&request.plan).ok_or_else(|| {
        ApiError::new(
            StatusCode::NOT_FOUND,
            "That plan is gone (it was used, or it waited too long). Make a new one.",
        )
    })?;
    let total = match &stored {
        Stored::Write { plan, .. } => plan.entries.len(),
        Stored::Undo { .. } => 0,
    };
    let job = new_id()?;
    // A plan that cannot start now is put back, so the person need not make it again.
    let Some(progress) = crate::server::jobs::start(&state.jobs, job.clone(), total) else {
        lock(&state.plans).put(request.plan, stored);
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "Another change is still running. Wait for it to finish.",
        ));
    };
    let shared = Arc::clone(&state);
    let id = job.clone();
    tokio::task::spawn_blocking(move || {
        let result = match stored {
            Stored::Write { kind, plan } => write(&shared, kind, plan, &progress),
            Stored::Undo { run } => undo(&shared, &run, &progress),
        };
        lock(&shared.jobs).finish(&id, result);
        shared.forget();
    });
    Ok(Json(Started { job }))
}

fn write(
    state: &AppState,
    kind: RunKind,
    plan: Plan,
    progress: &crate::server::jobs::Progress,
) -> Result<Finished, String> {
    let root = state
        .snapshot()
        .map(|s| root_of(&s).to_path_buf())
        .unwrap_or_default();
    let observer = |event: Event| {
        if matches!(event, Event::TargetDone { .. }) {
            progress.done.fetch_add(1, Ordering::Relaxed);
        }
    };
    let outcome = run_plan(&state.config.dirs, kind, plan, &observer)
        .map_err(|e| skillmirror_core::describe(&e))?;
    let lines: Vec<Line> = outcome
        .report
        .applied
        .iter()
        .map(|applied| line_of(applied, &root))
        .collect();
    let failed = lines.iter().filter(|l| l.outcome == "failed").count();
    Ok(Finished {
        title: format!("{} finished", kind.name()),
        lines,
        failed,
        backup: (outcome.backup.stored > 0).then(|| outcome.backup.id.clone()),
        warnings: outcome
            .backup
            .prune_error
            .iter()
            .map(|e| format!("Old backups could not be removed: {e}"))
            .collect(),
    })
}

fn line_of(applied: &Applied, root: &std::path::Path) -> Line {
    let (outcome, detail) = match &applied.outcome {
        Outcome::Unchanged => ("unchanged", None),
        Outcome::Created => ("created", None),
        Outcome::Updated => ("updated", None),
        Outcome::Failed(Failure::Plan(e)) => ("failed", Some(skillmirror_core::describe(e))),
        Outcome::Failed(Failure::Apply(e)) => ("failed", Some(skillmirror_core::describe(e))),
    };
    Line {
        project: project_label(root, &applied.target.project),
        skill: applied.target.skill.clone(),
        outcome,
        detail,
    }
}

pub(crate) async fn job(
    State(state): Shared,
    Path(id): Path<String>,
) -> Result<Json<View>, ApiError> {
    lock(&state.jobs)
        .view(&id)
        .map(Json)
        .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, "No such job."))
}
