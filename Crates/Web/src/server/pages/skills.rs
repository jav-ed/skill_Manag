//! The sync and push pages: the skills to pick from, and the place where a plan appears.

use std::sync::Arc;

use axum::extract::State;
use axum::response::Response;
use maud::{Markup, html};

use super::{Shared, with_snapshot};
use crate::server::state::AppState;
use crate::server::views::{SkillRow, project_label, push_rows, root_of, sync_rows};

pub(crate) async fn sync(State(state): Shared) -> Response {
    skills_page(&state, "/sync", "Sync").await
}

pub(crate) async fn push(State(state): Shared) -> Response {
    skills_page(&state, "/push", "Push").await
}

async fn skills_page(state: &Arc<AppState>, active: &'static str, name: &'static str) -> Response {
    with_snapshot(state, active, move |app, snapshot| {
        let pushing = active == "/push";
        let rows = if pushing { push_rows(snapshot) } else { sync_rows(snapshot) };
        let root = root_of(snapshot);
        let write = app.config.allow_write;
        html! {
            h2 { (name) }
            p.lead {
                @if pushing {
                    "Installs the mandatory skills in every project that has a skills folder, and refreshes the ones already there."
                } @else {
                    "Refreshes the skills each project already has from the vault. It never adds a skill to a project that does not have it."
                }
            }
            @if !write {
                p.note { "This server only looks. Start it with " code { "skillmirror web --allow-write" } " to run a " (name.to_lowercase()) " from here." }
            }
            div.bar {
                input type="search" placeholder="Filter skills" data-filter="#skills" aria-label="Filter skills";
                @if write {
                    select #project aria-label="Project" {
                        option value="" { "All projects" }
                        @for project in &snapshot.status.projects {
                            option value=(project.project.display()) { (project_label(root, &project.project)) }
                        }
                    }
                    button data-action="select-changed" { "Select changed" }
                    button data-action="select-all" { "Select all" }
                    button data-action="select-none" { "Select none" }
                    button.primary data-action="plan" { "Plan…" }
                }
            }
            @if rows.is_empty() {
                p { "No skill to " (name.to_lowercase()) "." }
            } @else {
                (skill_table(&rows, pushing, write))
            }
            div #panel aria-live="polite" {}
        }
    })
    .await
}

fn skill_table(rows: &[SkillRow], pushing: bool, write: bool) -> Markup {
    html! {
        table #skills data-kind=(if pushing { "push" } else { "sync" }) {
            thead { tr {
                @if write { th {} }
                th { "Skill" } th { "Group" } th.num { "Installed in" } th { "State" }
            } }
            tbody {
                @for row in rows {
                    tr data-changes=(row.changes()) {
                        @if write { td { input type="checkbox" value=(row.name) aria-label=(row.name) checked[row.changes() > 0]; } }
                        td { (row.name) @if row.mandatory { span.badge { "mandatory" } } }
                        td.muted { (row.group) }
                        td.num { (row.installed()) }
                        td {
                            @if row.problems > 0 { span.bad { (row.problems) " cannot be compared " } }
                            @if row.outdated > 0 { span.warn { (row.outdated) " outdated " } }
                            @if row.missing > 0 { span.warn { (row.missing) " missing " } }
                            @if row.changes() == 0 && row.problems == 0 { span.ok { "up to date" } }
                        }
                    }
                }
            }
        }
    }
}
