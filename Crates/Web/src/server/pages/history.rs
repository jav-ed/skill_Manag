//! The history page: the backup runs, each with a button to undo it when changes are allowed.

use axum::extract::State;
use axum::response::{IntoResponse, Response};
use maud::html;
use skillmirror_core::ops;

use super::{Shared, failure, layout};

pub(crate) async fn history(State(state): Shared) -> Response {
    let dirs = state.config.dirs.clone();
    let listed = tokio::task::spawn_blocking(move || ops::list_runs(&dirs)).await;
    let write = state.config.allow_write;
    let body = match listed {
        Ok(Ok(runs)) => html! {
            h2 { "History" }
            p.lead { "Every run that replaced or removed a skill folder kept the old copy. Undoing a run brings it back; undoing is a run too, so it can be undone." }
            @if !write {
                p.note { "Start the server with " code { "--allow-write" } " to undo a run from here." }
            }
            @if runs.is_empty() {
                p { "No run has anything to undo." }
            } @else {
                table {
                    thead { tr { th { "Run" } th { "Command" } th.num { "Skills" } th.num { "Projects" } th {} } }
                    tbody {
                        @for run in &runs {
                            tr {
                                td.path { (run.date) }
                                td { (run.command) }
                                td.num { (run.skills) }
                                td.num { (run.projects) }
                                td {
                                    @if let Some(error) = &run.error {
                                        span.bad { (error) }
                                    } @else if write {
                                        button data-action="undo" data-run=(run.id) { "Undo…" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div #panel aria-live="polite" {}
        },
        Ok(Err(message)) => return failure(&state, "/history", &message),
        Err(error) => return failure(&state, "/history", &format!("the work stopped: {error}")),
    };
    layout(&state, "/history", "history", &body).into_response()
}
