//! The pages: server-rendered with `maud`, which escapes every value. The page script only adds behaviour.

mod history;
mod skills;
mod system;

pub(crate) use history::history;
pub(crate) use skills::{push, sync};
pub(crate) use system::{doctor, report, settings};

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use maud::{DOCTYPE, Markup, html};

use super::state::AppState;
use super::views::{project_label, root_of};

pub(super) type Shared = State<Arc<AppState>>;

const NAV: [(&str, &str); 7] = [
    ("/", "Overview"),
    ("/sync", "Sync"),
    ("/push", "Push"),
    ("/history", "History"),
    ("/doctor", "Doctor"),
    ("/settings", "Settings"),
    ("/report", "Report"),
];

pub(super) fn layout(state: &AppState, active: &str, title: &str, body: &Markup) -> Html<String> {
    let page = html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="color-scheme" content="light dark";
                title { "skillmirror · " (title) }
                link rel="icon" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16'%3E%3Ccircle cx='8' cy='8' r='7' fill='%231c6fb8'/%3E%3C/svg%3E";
                link rel="stylesheet" href="/app.css";
            }
            body {
                header.top {
                    h1 { "skillmirror" }
                    nav {
                        @for (path, name) in NAV {
                            a href=(path) class=(if path == active { "on" } else { "" }) { (name) }
                        }
                    }
                    @if state.config.allow_write {
                        span.mode.write title="Sync, push and undo can be run from these pages" { "changes allowed" }
                    } @else {
                        span.mode title="Start with --allow-write to run sync, push and undo from here" { "read-only" }
                    }
                }
                main { (*body) }
                script src="/app.js" defer {}
            }
        }
    };
    Html(page.into_string())
}

/// Something went wrong before a page could be made: say what, plainly.
pub(super) fn failure(state: &AppState, active: &str, message: &str) -> Response {
    let body = html! {
        h2 { "This page cannot be made" }
        p.bad style="white-space: pre-wrap" { (message) }
        p.muted { "`skillmirror doctor` checks the vault and the configuration." }
    };
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        layout(state, active, "error", &body),
    )
        .into_response()
}

/// Runs the slow part off the async threads and turns its failure into a page.
pub(super) async fn with_snapshot<F>(
    state: &Arc<AppState>,
    active: &'static str,
    make: F,
) -> Response
where
    F: FnOnce(&AppState, &super::state::Snapshot) -> Markup + Send + 'static,
{
    let shared = Arc::clone(state);
    let outcome = tokio::task::spawn_blocking(move || {
        shared.snapshot().map(|snapshot| make(&shared, &snapshot))
    })
    .await;
    match outcome {
        Ok(Ok(body)) => layout(state, active, active, &body).into_response(),
        Ok(Err(message)) => failure(state, active, &message),
        Err(error) => failure(state, active, &format!("the work stopped: {error}")),
    }
}

pub(crate) async fn overview(State(state): Shared) -> Response {
    with_snapshot(&state, "/", |_, snapshot| {
        let root = root_of(snapshot);
        let status = &snapshot.status;
        let drifting = status.drifting();
        let problems = status.failed();
        html! {
            h2 { "Overview" }
            p.lead { "How every project stands against the vault " span.path { (snapshot.workspace.vault.path.display()) } "." }
            div.counts {
                span.pill { (status.projects.len()) " projects" }
                span.pill.ok { (status.projects.len() - drifting) " in sync" }
                span.pill.warn { (drifting) " differ" }
                @if problems > 0 { span.pill.bad { (problems) " with a problem" } }
                span.pill.muted { "looked at " (snapshot.at) }
                button data-action="rescan" { "Rescan" }
            }
            @if status.projects.is_empty() {
                p { "No project under " span.path { (root.display()) } " has a " code { ".agents/skills" } " folder." }
            } @else {
                table {
                    thead { tr { th { "Project" } th { "State" } } }
                    tbody {
                        @for project in &status.projects {
                            tr {
                                td.path { (project_label(root, &project.project)) }
                                td {
                                    @for o in &project.outdated { span.warn { (o.skill) " outdated" } " " }
                                    @for name in &project.missing_mandatory { span.warn { (name) " missing" } " " }
                                    @for name in &project.missing_bridges { span.warn { "link " (name) " missing" } " " }
                                    @for p in &project.failed { span.bad { (p.skill) ": " (first_line(&p.message)) } " " }
                                    @for p in &project.project_problems { span.bad { "link " (p.skill) ": " (first_line(&p.message)) } " " }
                                    @for name in &project.not_in_vault { span.muted { (name) " not in the vault" } " " }
                                    @if !project.drifts() && project.failed.is_empty() && project.project_problems.is_empty() {
                                        span.ok { "in sync" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            @if !snapshot.scan.issues.is_empty() {
                h2 { "The scan could not read" }
                ul {
                    @for issue in &snapshot.scan.issues {
                        li { span.path { (issue.path.display()) } ": " (issue.message) }
                    }
                }
            }
        }
    })
    .await
}

fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or_default()
}
