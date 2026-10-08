//! `/report`: the static report of the same data, served as it is written to a file.

use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use skillmirror_core::ops::report_data;

use super::state::AppState;
use crate::render_report;

/// The report has its own inline style and script and says so in its own policy; nothing may be loaded
/// or sent from it.
const POLICY: &str = "default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'";

fn failure(message: &str) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        format!("The report cannot be made.\n\n{message}\n"),
    )
        .into_response()
}

pub(crate) async fn report(State(state): State<Arc<AppState>>) -> Response {
    let shared = Arc::clone(&state);
    let made = tokio::task::spawn_blocking(move || {
        let snapshot = shared.snapshot()?;
        report_data(&snapshot.workspace, &snapshot.scan)
            .map(|data| render_report(&data, &snapshot.at))
            .map_err(|e| skillmirror_core::describe(&e))
    })
    .await;
    match made {
        Ok(Ok(page)) => {
            let mut response = Html(page).into_response();
            response.headers_mut().insert(
                header::CONTENT_SECURITY_POLICY,
                HeaderValue::from_static(POLICY),
            );
            response
        }
        Ok(Err(message)) => failure(&message),
        Err(error) => failure(&format!("the work stopped: {error}")),
    }
}
