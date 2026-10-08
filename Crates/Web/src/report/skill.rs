//! A card for every skill: what it is, its files, and what it is in each project.

use maud::{Markup, html};
use skillmirror_core::ops::{Cell, DiffKind, FileDiff, ReportData, Skipped};

use super::{find_text, project_name};

/// Lines of one file's diff shown in the card; the command `skillmirror diff` shows all of them.
const LINES_SHOWN: usize = 400;

pub(super) fn sections(data: &ReportData) -> Markup {
    html! {
        h2 { "Skills" }
        @for (number, skill) in data.skills.iter().enumerate() {
            @let find = find_text(&[
                &skill.name,
                &skill.group.join("/"),
                skill.description.as_deref().unwrap_or_default(),
            ]);
            section.skill #{ "skill-" (number) } data-find=(find) {
                h3 {
                    (skill.name)
                    @if skill.mandatory { span.badge { "mandatory" } }
                }
                @if !skill.group.is_empty() {
                    div.group { "group " (skill.group.join("/")) }
                }
                @if let Some(description) = &skill.description {
                    p.desc { (description) }
                } @else if let Some(problem) = &skill.header_problem {
                    p.desc.warn { "Header of SKILL.md: " (problem) }
                } @else {
                    p.desc.warn { "The header of SKILL.md has no description." }
                }
                @if !skill.files.is_empty() {
                    ul.files { @for file in &skill.files { li { (file) } } }
                }
                (states(data, &skill.name, number))
            }
        }
        @for (offset, name) in data.extra.iter().enumerate() {
            @let number = data.skills.len() + offset;
            section.skill #{ "skill-" (number) } data-find=(find_text(&[name, "not in the vault"])) {
                h3 { (name) span.badge { "not in the vault" } }
                p.desc.warn { "Installed in these projects; the vault has no skill of this name, so sync leaves it alone." }
                (states(data, name, number))
            }
        }
    }
}

/// The projects that have a state for the skill.
fn states(data: &ReportData, name: &str, number: usize) -> Markup {
    html! {
        table.states {
            @for (index, project) in data.projects.iter().enumerate() {
                @if let Some(cell) = data.cells.get(&(name.to_string(), index)) {
                    tr {
                        td title=(project.display()) { (project_name(data, project)) }
                        td { (words(cell)) (changes(cell, number, index)) }
                    }
                }
            }
        }
    }
}

fn words(cell: &Cell) -> Markup {
    html! {
        @match cell {
            Cell::Current => span.current { "up to date" },
            Cell::Outdated(files) => span.outdated {
                "outdated, " (files.len()) @if files.len() == 1 { " file differs" } @else { " files differ" }
            },
            Cell::MissingMandatory => span.missing { "mandatory, not installed" },
            Cell::NotInVault => span.notinvault { "not in the vault" },
            Cell::Problem { message, hint } => {
                span.problem { (message) }
                @if let Some(hint) = hint { " (" (hint) ")" }
            },
        }
    }
}

fn changes(cell: &Cell, number: usize, project: usize) -> Markup {
    let Cell::Outdated(files) = cell else {
        return html! {};
    };
    let added: usize = files.iter().map(|f| f.added_lines).sum();
    let removed: usize = files.iter().map(|f| f.removed_lines).sum();
    html! {
        details.diff #{ "d-" (number) "-" (project) } {
            summary { "show changes (+" (added) " -" (removed) ")" }
            pre.diff { @for file in files { (file_diff(file)) } }
        }
    }
}

fn file_diff(file: &FileDiff) -> Markup {
    let what = match file.kind {
        DiffKind::Added => "new file",
        DiffKind::Modified => "changed",
        DiffKind::ModeChanged => "permissions",
        DiffKind::Removed => "removed",
    };
    let text: Vec<&str> = file.text.lines().skip(2).collect();
    html! {
        span.file { (file.path.display()) " (" (what) ")" }
        @if let Some((old, new)) = file.modes {
            span.note { "mode " (format!("{old:o}")) " → " (format!("{new:o}")) }
        }
        @match &file.skipped {
            Some(Skipped::Binary) => span.note { "binary file, lines not shown" },
            Some(Skipped::TooLarge) => span.note { "too large, lines not shown" },
            Some(Skipped::Unreadable(reason)) => span.note { "cannot be read: " (reason) },
            None => {}
        }
        @for line in text.iter().take(LINES_SHOWN) {
            span class=(class_of(line)) { (line) "\n" }
        }
        @if text.len() > LINES_SHOWN {
            span.note { "… " (text.len() - LINES_SHOWN) " more lines; `skillmirror diff` shows them all" }
        }
    }
}

fn class_of(line: &str) -> &'static str {
    if line.starts_with("@@") {
        "hunk"
    } else if line.starts_with('+') {
        "add"
    } else if line.starts_with('-') {
        "del"
    } else {
        ""
    }
}
