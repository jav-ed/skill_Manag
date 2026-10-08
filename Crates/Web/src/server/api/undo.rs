//! Undoing a run: what it would do, then (through the apply route) doing it.

use std::sync::atomic::Ordering;

use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use skillmirror_core::events::Event;
use skillmirror_core::ops::{self, Step, UndoView};

use super::{ApiError, Shared, blocking, new_id, writable};
use crate::server::jobs::{Finished, Line};
use crate::server::plans::Stored;
use crate::server::state::{AppState, lock};
use crate::server::views::{project_label, root_of};

#[derive(Deserialize)]
pub(crate) struct UndoRequest {
    run: String,
}

#[derive(Serialize)]
pub(crate) struct UndoLineView {
    project: String,
    skill: String,
    was: &'static str,
    /// `restore`, `remove`, `gone` or `failed`.
    step: &'static str,
    message: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct UndoPlanView {
    plan: String,
    run: String,
    date: String,
    command: String,
    lines: Vec<UndoLineView>,
    actionable: usize,
    failed: usize,
}

pub(crate) async fn undo_plan(
    State(state): Shared,
    Json(request): Json<UndoRequest>,
) -> Result<Json<UndoPlanView>, ApiError> {
    writable(&state)?;
    let dirs = state.config.dirs.clone();
    let run = request.run.clone();
    let view = blocking(move || ops::plan_undo(&dirs, &run))
        .await?
        .map_err(ApiError::engine)?;
    let id = new_id()?;
    lock(&state.plans).put(id.clone(), Stored::Undo { run: request.run });
    Ok(Json(undo_view(&state, view, id)))
}

fn undo_view(state: &AppState, view: UndoView, plan: String) -> UndoPlanView {
    let root = state
        .snapshot()
        .map(|s| root_of(&s).to_path_buf())
        .unwrap_or_default();
    let (actionable, failed) = (view.actionable(), view.failed());
    let lines = view
        .lines
        .iter()
        .map(|line| {
            let (step, message) = match &line.step {
                Step::Restore => ("restore", None),
                Step::Remove => ("remove", None),
                Step::Gone => ("gone", None),
                Step::Failed(message) => ("failed", Some(message.clone())),
            };
            UndoLineView {
                project: project_label(&root, std::path::Path::new(&line.project)),
                skill: line.skill.clone(),
                was: line.was,
                step,
                message,
            }
        })
        .collect();
    UndoPlanView {
        plan,
        run: view.run,
        date: view.date,
        command: view.command,
        lines,
        actionable,
        failed,
    }
}

pub(super) fn undo(
    state: &AppState,
    run: &str,
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
    let done = ops::do_undo(&state.config.dirs, run, &observer)?;
    let lines: Vec<Line> = done
        .lines
        .iter()
        .map(|line| {
            let (outcome, detail) = match &line.step {
                Step::Restore => ("restored", None),
                Step::Remove => ("removed", None),
                Step::Gone => ("gone", None),
                Step::Failed(message) => ("failed", Some(message.clone())),
            };
            Line {
                project: project_label(&root, std::path::Path::new(&line.project)),
                skill: line.skill.clone(),
                outcome,
                detail,
            }
        })
        .collect();
    let failed = lines.iter().filter(|l| l.outcome == "failed").count();
    Ok(Finished {
        title: format!("undo of {} finished", done.run),
        lines,
        failed,
        backup: done.saved_as,
        warnings: done.warnings,
    })
}
