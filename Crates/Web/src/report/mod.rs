//! The static report: one HTML file with no outside reference, made of the matrix of skills against
//! projects, the vault as a tree, a card for every skill with the diffs of the projects that differ, and
//! what could not be read. Every value goes through `maud`, which escapes it; only the style sheet and the
//! script, both written here, are inserted as they are.

mod matrix;
mod problems;
mod skill;
mod tree;

use maud::{DOCTYPE, Markup, PreEscaped, html};
use skillmirror_core::ops::{Cell, ReportData};

/// The comment after the doctype that marks a file as written by this tool, so a later run may replace it
/// and never replaces a file of someone else's.
pub const MARKER: &str = "<!-- skillmirror report -->";

const CSS: &str = include_str!("report.css");
const JS: &str = include_str!("report.js");
/// The page may not load or send anything; its own style and script are inline.
const CSP: &str = "default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline'";

/// The last two components of a path, such as `parent/project`.
pub(crate) fn short(path: &std::path::Path) -> String {
    let parts: Vec<String> = path
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();
    let from = parts.len().saturating_sub(2);
    parts.get(from..).unwrap_or_default().join("/")
}

/// A project as the page names it: the path below the scan root, or the last two components when the
/// project is not below it.
pub(crate) fn project_name(data: &ReportData, path: &std::path::Path) -> String {
    match path.strip_prefix(&data.root) {
        Ok(below) if !below.as_os_str().is_empty() => below.display().to_string(),
        _ => short(path),
    }
}

/// The text a filter looks at: lower case, one line.
pub(crate) fn find_text(parts: &[&str]) -> String {
    parts.join(" ").to_lowercase()
}

/// Renders the whole page. `generated` is the time to print in the header, such as `2026-10-08 12:00 UTC`.
pub fn render_report(data: &ReportData, generated: &str) -> String {
    let page = html! {
        (DOCTYPE)
        (PreEscaped(MARKER))
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta http-equiv="Content-Security-Policy" content=(CSP);
                meta name="color-scheme" content="light dark";
                title { "skillmirror report" }
                style { (PreEscaped(CSS)) }
            }
            body {
                (top(data, generated))
                main {
                    (counts(data))
                    p #nomatch hidden { "No skill matches the filter." }
                    (matrix::section(data))
                    (tree::section(data))
                    (skill::sections(data))
                    (problems::section(data))
                }
                script { (PreEscaped(JS)) }
            }
        }
    };
    page.into_string()
}

fn top(data: &ReportData, generated: &str) -> Markup {
    html! {
        header.top {
            h1 { "skillmirror report" }
            div.meta {
                "vault " (data.vault.display()) " · root " (data.root.display()) " · " (generated)
            }
            input #filter type="search" placeholder="filter skills" aria-label="filter skills";
            button #theme type="button" { "light/dark" }
        }
    }
}

fn counts(data: &ReportData) -> Markup {
    let count = |wanted: fn(&Cell) -> bool| data.cells.values().filter(|c| wanted(c)).count();
    html! {
        div.counts {
            span { (data.skills.len()) " skills in the vault" }
            span { (data.projects.len()) " projects" }
            span { (count(|c| matches!(c, Cell::Current))) " up to date" }
            span { (count(|c| matches!(c, Cell::Outdated(_)))) " outdated" }
            span { (count(|c| matches!(c, Cell::MissingMandatory))) " mandatory missing" }
            span { (count(|c| matches!(c, Cell::Problem { .. }))) " problems" }
        }
    }
}

#[cfg(test)]
mod tests;
