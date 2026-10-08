//! The confirmation before anything is removed or written.

use std::collections::HashSet;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::boxed;
use crate::hit::{HitMap, Target};
use crate::num::plural;
use crate::results::Kind;
use crate::screens::Pending;
use crate::theme;

/// Removed files named on the page; the rest are counted. Removing is the part nobody should miss.
const REMOVALS_SHOWN: usize = 6;
const FAILURES_SHOWN: usize = 2;

pub(super) fn draw(
    frame: &mut Frame,
    hits: &mut HitMap,
    hover: Option<Target>,
    pending: &Pending,
    area: Rect,
) {
    let (verb, color) = match pending.kind {
        Kind::Delete => ("Delete", theme::ERROR),
        Kind::Sync => ("Sync", theme::ACCENT),
        Kind::Push => ("Push", theme::ACCENT),
    };
    let projects: HashSet<_> = pending.targets.iter().map(|t| &t.project).collect();
    let title = format!(
        "{verb} {} in {}?",
        plural(pending.skills, "skill"),
        plural(projects.len(), "project")
    );
    let buttons = boxed::draw(frame, hits, area, &title, color, body(pending), 1);
    let [yes, no] = Layout::horizontal([Constraint::Length(18), Constraint::Length(18)])
        .flex(Flex::Center)
        .spacing(2)
        .areas(buttons);
    draw_button(frame, hits, hover, 0, yes, &format!("y  {verb}"), color);
    draw_button(frame, hits, hover, 1, no, "n  Cancel", theme::ACCENT);
}

/// The text between the title and the buttons.
fn body(pending: &Pending) -> Vec<Line<'static>> {
    let Some(preview) = &pending.preview else {
        return vec![
            Line::raw("The selected skills are removed from all matching projects."),
            Line::raw("A copy is kept first: History (or skillmirror undo) puts it back."),
        ];
    };
    let mut doing = Vec::new();
    if preview.created > 0 {
        doing.push(format!(
            "creates {}",
            plural(preview.created, "skill folder")
        ));
    }
    if preview.updated > 0 {
        doing.push(format!(
            "updates {}",
            plural(preview.updated, "skill folder")
        ));
    }
    let mut lines = vec![Line::raw(format!(
        "{}: {} added, {} changed, {} removed.",
        capitalized(&doing.join(" and ")),
        plural(preview.files_added, "file"),
        plural(preview.files_changed, "file"),
        plural(preview.removals.len(), "file"),
    ))];
    for removal in preview.removals.iter().take(REMOVALS_SHOWN) {
        lines.push(Line::from(Span::styled(
            format!(
                "  removes {}/{} in {}",
                removal.skill, removal.path, removal.project
            ),
            theme::warning(),
        )));
    }
    if preview.removals.len() > REMOVALS_SHOWN {
        lines.push(Line::from(Span::styled(
            format!(
                "  and {} more removed",
                preview.removals.len() - REMOVALS_SHOWN
            ),
            theme::warning(),
        )));
    }
    for failure in preview.failures.iter().take(FAILURES_SHOWN) {
        lines.push(Line::from(Span::styled(
            format!("  cannot write {failure}"),
            theme::error(),
        )));
    }
    if preview.failures.len() > FAILURES_SHOWN {
        lines.push(Line::from(Span::styled(
            format!(
                "  and {} more cannot be written",
                preview.failures.len() - FAILURES_SHOWN
            ),
            theme::error(),
        )));
    }
    lines
}

fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

pub(super) fn draw_button(
    frame: &mut Frame,
    hits: &mut HitMap,
    hover: Option<Target>,
    index: usize,
    area: Rect,
    label: &str,
    color: Color,
) {
    let mut style = Style::new().fg(color);
    if hover == Some(Target::Button(index)) {
        style = style.add_modifier(Modifier::REVERSED | Modifier::BOLD);
    }
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(format!("[ {label} ]"), style)))
            .alignment(Alignment::Center),
        area,
    );
    hits.push(area, Target::Button(index));
}
