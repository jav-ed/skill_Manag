//! The doctor, settings and report pages: what the machine and the configuration look like.

use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderValue, header};
use axum::response::{Html, IntoResponse, Response};
use maud::html;
use skillmirror_core::config::Source;
use skillmirror_core::ops::{self, Severity, report_data};

use super::{Shared, failure, layout};
use crate::render_report;

pub(crate) async fn doctor(State(state): Shared) -> Response {
    let config = state.config.clone();
    let report = tokio::task::spawn_blocking(move || {
        ops::doctor(&config.flags, Ok(config.env.clone()), &config.dirs)
    })
    .await;
    let Ok(report) = report else {
        return failure(&state, "/doctor", "the checks stopped");
    };
    let body = html! {
        h2 { "Doctor" }
        p.lead { "The machine, the configuration, the vault and every SKILL.md header. Nothing is written." }
        @if report.findings.is_empty() {
            p.ok { "Everything checked out." }
        } @else {
            table {
                thead { tr { th { "" } th { "Check" } th { "Subject" } th { "Finding" } } }
                tbody {
                    @for finding in &report.findings {
                        tr {
                            td class=(severity_class(finding.severity)) { (severity_name(finding.severity)) }
                            td { (finding.check) }
                            td.path { (finding.subject.as_deref().unwrap_or("")) }
                            td {
                                (finding.message)
                                @if let Some(hint) = &finding.hint { br; span.muted { "hint: " (hint) } }
                            }
                        }
                    }
                }
            }
        }
    };
    layout(&state, "/doctor", "doctor", &body).into_response()
}

fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Note => "note",
        Severity::Warning => "warning",
        Severity::Error => "error",
    }
}

fn severity_class(severity: Severity) -> &'static str {
    match severity {
        Severity::Note => "muted",
        Severity::Warning => "warn",
        Severity::Error => "bad",
    }
}

fn source_name(source: Source) -> &'static str {
    match source {
        Source::Flag => "flag",
        Source::Env => "environment",
        Source::PointerFile => "pointer file",
        Source::VaultConfig => "vault config",
    }
}

pub(crate) async fn settings(State(state): Shared) -> Response {
    let settings = match state.config.settings() {
        Ok(settings) => settings,
        Err(message) => return failure(&state, "/settings", &message),
    };
    let config = settings.config();
    let list = |items: Vec<String>| {
        if items.is_empty() {
            "none".to_string()
        } else {
            items.join(", ")
        }
    };
    let body = html! {
        h2 { "Settings" }
        p.lead { "What this server reads. Change it with " code { "skillmirror config" } " and " code { "skillmirror mandatory" } ", or edit config.yaml." }
        dl.settings {
            dt { "Vault" }
            dd { @match settings.vault() {
                Ok(v) => { span.path { (v.value.display()) } " " span.muted { "(" (source_name(v.source)) ")" } }
                Err(_) => { span.warn { "not set" } }
            } }
            dt { "Scan root" }
            dd { @match settings.root() {
                Ok(r) => { span.path { (r.value.display()) } " " span.muted { "(" (source_name(r.source)) ")" } }
                Err(_) => { span.warn { "not set" } }
            } }
            dt { "Mandatory" } dd { (list(config.mandatory.clone())) }
            dt { "Targets" } dd { (list(config.targets.clone())) }
            dt { "Excluded names" } dd { (list(config.exclude_dirs.clone())) }
            dt { "Excluded paths" } dd { (list(config.exclude_paths.iter().map(|p| p.display().to_string()).collect())) }
            dt { "Profiles" } dd { (list(config.profiles.keys().cloned().collect())) }
            dt { "Changes" } dd { @if state.config.allow_write { "allowed (--allow-write)" } @else { "not allowed" } }
        }
    };
    layout(&state, "/settings", "settings", &body).into_response()
}

/// The static report of the same data, served as it is written to a file.
pub(crate) async fn report(State(state): Shared) -> Response {
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
            // The report has its own inline style and script; its page says so in its own policy.
            response.headers_mut().insert(
                header::CONTENT_SECURITY_POLICY,
                HeaderValue::from_static(
                    "default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'",
                ),
            );
            response
        }
        Ok(Err(message)) => failure(&state, "/report", &message),
        Err(error) => failure(&state, "/report", &format!("the work stopped: {error}")),
    }
}
