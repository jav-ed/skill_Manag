//! What could not be read or made, and the links that are missing.

use maud::{Markup, html};
use skillmirror_core::ops::ReportData;

use super::project_name;

pub(super) fn section(data: &ReportData) -> Markup {
    let any = !data.issues.is_empty()
        || !data.project_problems.is_empty()
        || !data.missing_bridges.is_empty();
    if !any {
        return html! {};
    }
    let name = |index: usize| {
        data.projects
            .get(index)
            .map(|p| project_name(data, p))
            .unwrap_or_default()
    };
    html! {
        h2 { "Needs a look" }
        ul.problems {
            @for issue in &data.issues {
                li { span.problem { "scan" } " " (issue.path.display()) ": " (issue.message) }
            }
            @for (index, message) in &data.project_problems {
                li { span.problem { (name(*index)) } ": " (message) }
            }
            @for (index, link) in &data.missing_bridges {
                li { span.missing { (name(*index)) } ": the " (link) " link to .agents/skills is not made (skillmirror bridge)" }
            }
        }
    }
}
