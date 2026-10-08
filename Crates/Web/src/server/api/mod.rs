//! The JSON the page script talks to: make a plan, apply it, watch the job, undo a run.

mod apply;
mod plan;
mod undo;

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use super::guard::random_hex;
use super::state::AppState;

pub(crate) use apply::{apply, job};
pub(crate) use plan::plan;
pub(crate) use undo::undo_plan;

#[cfg(test)]
pub(crate) use plan::cut_for_tests;

type Shared = State<Arc<AppState>>;

/// An error answer: a status and a sentence for the person.
pub(crate) struct ApiError(StatusCode, String);

impl ApiError {
    pub(super) fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self(status, message.into())
    }

    pub(super) fn engine(message: String) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct Body {
            error: String,
        }
        (self.0, Json(Body { error: self.1 })).into_response()
    }
}

/// A blocking piece of work, run off the async threads.
pub(super) async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
) -> Result<T, ApiError> {
    tokio::task::spawn_blocking(work).await.map_err(|e| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("the work stopped: {e}"),
        )
    })
}

pub(super) fn writable(state: &AppState) -> Result<(), ApiError> {
    if state.config.allow_write {
        Ok(())
    } else {
        Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "This server only looks. Start it with `skillmirror web --allow-write` to change anything.",
        ))
    }
}

pub(super) fn new_id() -> Result<String, ApiError> {
    random_hex(16).map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e))
}

#[derive(Serialize)]
pub(crate) struct Rescanned {
    at: String,
    projects: usize,
}

pub(crate) async fn rescan(State(state): Shared) -> Result<Json<Rescanned>, ApiError> {
    let shared = Arc::clone(&state);
    let snapshot = blocking(move || shared.refresh())
        .await?
        .map_err(ApiError::engine)?;
    Ok(Json(Rescanned {
        at: snapshot.at.clone(),
        projects: snapshot.status.projects.len(),
    }))
}
