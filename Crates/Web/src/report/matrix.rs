//! The matrix: a row for every skill, a column for every project.

use maud::{Markup, html};
use skillmirror_core::ops::{Cell, ReportData};

use super::{find_text, project_name};

/// The class, the glyph and the words of a cell.
fn look(cell: Option<&Cell>) -> (&'static str, &'static str, String) {
    match cell {
        None => ("absent", "·", "not installed".to_string()),
        Some(Cell::Current) => ("current", "✓", "up to date".to_string()),
        Some(Cell::Outdated(files)) => (
            "outdated",
            "~",
            format!("outdated: {} to change", files.len()),
        ),
        Some(Cell::MissingMandatory) => ("missing", "+", "mandatory, not installed".to_string()),
        Some(Cell::NotInVault) => ("notinvault", "?", "the vault has no such skill".to_string()),
        Some(Cell::Problem { message, .. }) => ("problem", "✗", message.clone()),
    }
}

pub(super) fn section(data: &ReportData) -> Markup {
    html! {
        h2 { "Matrix" }
        p.legend {
            "✓ up to date · ~ outdated · + mandatory, not installed · ? not in the vault · ✗ could not be compared · · not installed"
        }
        div.scroll {
            table.matrix {
                thead {
                    tr {
                        th.skill { "skill" }
                        @for project in &data.projects {
                            th title=(project.display()) { span { (project_name(data, project)) } }
                        }
                    }
                }
                tbody {
                    @for (number, skill) in data.skills.iter().enumerate() {
                        @let find = find_text(&[
                            &skill.name,
                            &skill.group.join("/"),
                            skill.description.as_deref().unwrap_or_default(),
                        ]);
                        tr data-find=(find) data-row {
                            th.skill {
                                a href={ "#skill-" (number) } { (skill.name) }
                                @if skill.mandatory { span.badge { "mandatory" } }
                            }
                            @for index in 0..data.projects.len() {
                                (cell(data, &skill.name, index, Some(number)))
                            }
                        }
                    }
                    @for (offset, name) in data.extra.iter().enumerate() {
                        tr data-find=(find_text(&[name, "not in the vault"])) data-row {
                            th.skill {
                                a href={ "#skill-" (data.skills.len() + offset) } { (name) }
                                span.badge { "not in the vault" }
                            }
                            @for index in 0..data.projects.len() {
                                (cell(data, name, index, Some(data.skills.len() + offset)))
                            }
                        }
                    }
                }
            }
        }
    }
}

fn cell(data: &ReportData, name: &str, project: usize, card: Option<usize>) -> Markup {
    let found = data.cells.get(&(name.to_string(), project));
    let (class, glyph, words) = look(found);
    let place = data
        .projects
        .get(project)
        .map(|p| project_name(data, p))
        .unwrap_or_default();
    let title = format!("{name} in {place}: {words}");
    html! {
        td class=(class) title=(title) {
            @match (found, card) {
                (Some(Cell::Outdated(_)), Some(number)) => {
                    a href={ "#d-" (number) "-" (project) } { (glyph) }
                }
                (Some(_), Some(number)) => { a href={ "#skill-" (number) } { (glyph) } }
                _ => { (glyph) }
            }
        }
    }
}
