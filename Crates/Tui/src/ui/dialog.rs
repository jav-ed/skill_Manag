//! The confirmation before anything is removed or written.

use std::collections::HashSet;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};

use crate::hit::{HitMap, Target};
use crate::num::{plural, to_u16};
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
    hits.push(area, Target::Dismiss);
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
    let body = body(pending);
    let width = 62.min(area.width);
    // Borders, the margin and the button row take five lines; long lines wrap inside the text width.
    let text_width = usize::from(width.saturating_sub(4)).max(1);
    let wrapped: usize = body
        .iter()
        .map(|line| line.width().div_ceil(text_width).max(1))
        .sum();
    let height = (to_u16(wrapped) + 5).max(8).min(area.height);
    let [row] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    let [rect] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(row);
    hits.push(rect, Target::Overlay);
    frame.render_widget(Clear, rect);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(color))
        .title(Span::styled(
            format!(" {title} "),
            Style::new().fg(color).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);
    let [text, buttons] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(inner);
    frame.render_widget(
        Paragraph::new(body).wrap(Wrap { trim: true }),
        text.inner(Margin::new(1, 1)),
    );
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
        return vec![Line::raw(
            "This will inshallah permanently remove the selected skills from all matching projects.",
        )];
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

fn draw_button(
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
