//! The local server behind `skillmirror web`: the same views as the report, live, and (when asked) sync,
//! push and undo with a plan to confirm first. It listens on the loopback address only and trusts
//! nobody who has not been given the link; see [`guard`] for what is checked on every request.

mod api;
mod guard;
mod jobs;
mod pages;
mod plans;
mod state;
mod views;

#[cfg(test)]
mod tests;

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::{HeaderValue, header};
use axum::middleware;
use axum::response::IntoResponse;
use axum::routing::{get, post};

use state::AppState;
pub use state::Config;

const CSS: &str = include_str!("assets/app.css");
const JS: &str = include_str!("assets/app.js");
/// A request body is a small JSON document; anything larger is not one of ours.
const BODY_LIMIT: usize = 64 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum ServeError {
    #[error("cannot listen on 127.0.0.1:{port}: {source}")]
    Bind { port: u16, source: std::io::Error },
    #[error("{0}")]
    Setup(String),
    #[error("the server stopped: {0}")]
    Io(#[from] std::io::Error),
}

/// How the server runs.
#[derive(Debug, Clone)]
pub struct ServeConfig {
    pub config: Config,
    /// `0` picks a free port.
    pub port: u16,
    /// The server ends after this long without a request. `None` runs until it is stopped.
    pub idle: Option<Duration>,
}

fn asset(body: &'static str, kind: &'static str) -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, HeaderValue::from_static(kind))],
        body,
    )
}

/// Every route, behind the guard.
pub(crate) fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(pages::overview))
        .route("/sync", get(pages::sync))
        .route("/push", get(pages::push))
        .route("/history", get(pages::history))
        .route("/doctor", get(pages::doctor))
        .route("/settings", get(pages::settings))
        .route("/report", get(pages::report))
        .route(
            "/app.css",
            get(|| async { asset(CSS, "text/css; charset=utf-8") }),
        )
        .route(
            "/app.js",
            get(|| async { asset(JS, "text/javascript; charset=utf-8") }),
        )
        .route("/api/plan", post(api::plan))
        .route("/api/apply", post(api::apply))
        .route("/api/undo-plan", post(api::undo_plan))
        .route("/api/rescan", post(api::rescan))
        .route("/api/job/{id}", get(api::job))
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            guard::check,
        ))
        .with_state(state)
}

/// Runs the server until Ctrl-C or until it has been idle for `idle`. `ready` is called with the link to
/// open once the port is bound. Blocks.
pub fn serve(options: ServeConfig, ready: impl FnOnce(&str)) -> Result<(), ServeError> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?;
    runtime.block_on(run(options, ready))
}

async fn run(options: ServeConfig, ready: impl FnOnce(&str)) -> Result<(), ServeError> {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, options.port));
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|source| ServeError::Bind {
            port: options.port,
            source,
        })?;
    let port = listener.local_addr()?.port();
    let token = guard::random_hex(32).map_err(ServeError::Setup)?;
    let state = Arc::new(AppState::new(options.config, port, token.clone()));
    ready(&format!("http://127.0.0.1:{port}/?token={token}"));
    let idle = options.idle;
    let watched = Arc::clone(&state);
    axum::serve(listener, router(state))
        .with_graceful_shutdown(async move {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {}
                () = idle_over(watched, idle) => {}
            }
        })
        .await?;
    Ok(())
}

/// Resolves when no request has come for `idle`; never when there is no limit.
pub(crate) async fn idle_over(state: Arc<AppState>, idle: Option<Duration>) {
    let Some(idle) = idle else {
        std::future::pending::<()>().await;
        return;
    };
    let tick = (idle / 4).clamp(Duration::from_millis(10), Duration::from_secs(5));
    loop {
        tokio::time::sleep(tick).await;
        if state.idle_for() >= idle {
            return;
        }
    }
}
