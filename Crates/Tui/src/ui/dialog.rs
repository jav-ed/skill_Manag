//! The confirmation before anything is removed.

use std::collections::HashSet;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};

use crate::hit::{HitMap, Target};
use crate::num::plural;
use crate::screens::Pending;
use crate::theme;

pub(super) fn draw(
    frame: &mut Frame,
    hits: &mut HitMap,
    hover: Option<Target>,
    pending: &Pending,
    area: Rect,
) {
    hits.push(area, Target::Dismiss);
    let projects: HashSet<_> = pending.targets.iter().map(|t| &t.project).collect();
    let title = format!(
        "Delete {} in {}?",
        plural(pending.skills, "skill"),
        plural(projects.len(), "project")
    );
    let width = 62.min(area.width);
    let height = 8.min(area.height);
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
        .border_style(Style::new().fg(theme::ERROR))
        .title(Span::styled(format!(" {title} "), theme::error()));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);
    let [text, buttons] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(inner);
    frame.render_widget(
        Paragraph::new("This will inshallah permanently remove the selected skills from all matching projects.")
            .wrap(Wrap { trim: true }),
        text.inner(ratatui::layout::Margin::new(1, 1)),
    );
    let [yes, no] = Layout::horizontal([Constraint::Length(18), Constraint::Length(18)])
        .flex(Flex::Center)
        .spacing(2)
        .areas(buttons);
    draw_button(frame, hits, hover, 0, yes, "y  Delete", theme::ERROR);
    draw_button(frame, hits, hover, 1, no, "n  Cancel", theme::ACCENT);
}

fn draw_button(
    frame: &mut Frame,
    hits: &mut HitMap,
    hover: Option<Target>,
    index: usize,
    area: Rect,
    label: &str,
    color: ratatui::style::Color,
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
