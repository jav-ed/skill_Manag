//! What the replace came to.

use ratatui::style::Style;
use ratatui::text::{Line, Span};

use super::project_text;
use crate::num::plural;
use crate::screens::Local;
use crate::screens::local::Done;
use crate::theme;

pub(super) fn lines(done: &Done, local: &Local) -> Vec<Line<'static>> {
    let mut out = vec![
        Line::from(Span::styled(
            format!(" Override results: {}", project_text(local)),
            theme::bold(),
        )),
        Line::raw(""),
    ];
    for outcome in &done.outcomes {
        match &outcome.result {
            Ok(files) => out.push(Line::from(vec![
                Span::styled("  ✓ ", Style::new().fg(theme::SUCCESS)),
                Span::raw(format!(
                    "{} replaced in the vault ({})",
                    outcome.name,
                    plural(*files, "file")
                )),
            ])),
            Err(message) => out.push(Line::from(vec![
                Span::styled("  ✗ ", theme::error()),
                Span::raw(format!(
                    "{} not replaced: {}",
                    outcome.name,
                    message.replace('\n', "  ")
                )),
            ])),
        }
        if outcome.left_out > 0 {
            out.push(Line::from(Span::styled(
                format!(
                    "    {} not taken by git (the vault's .gitignore?)",
                    plural(outcome.left_out, "file")
                ),
                theme::warning(),
            )));
        }
    }
    out.push(Line::raw(""));
    if done.outcomes.iter().any(|o| o.result.is_ok()) {
        out.push(Line::from(Span::styled(
            "  Staged in git, not committed. Commit it in the vault when it is ready.",
            theme::muted(),
        )));
    }
    if let Some(id) = &done.backup {
        out.push(Line::from(Span::styled(
            format!("  The old copies are saved as run {id}"),
            theme::muted(),
        )));
        out.push(Line::from(Span::styled(
            "  (History can undo it)",
            theme::muted(),
        )));
    }
    if let Some(spread) = &done.spread {
        out.push(Line::raw(""));
        match spread {
            Ok(spread) => {
                out.push(Line::raw(format!(
                    "  Other projects: {} written in {}.",
                    plural(spread.written, "skill folder"),
                    plural(spread.projects, "project")
                )));
                for failure in &spread.failed {
                    out.push(Line::from(vec![
                        Span::styled("  ✗ ", theme::error()),
                        Span::raw(failure.replace('\n', "  ")),
                    ]));
                }
            }
            Err(message) => out.push(Line::from(Span::styled(
                format!(
                    "  The other projects were not updated: {}",
                    message.replace('\n', "  ")
                ),
                theme::error(),
            ))),
        }
    }
    out
}
