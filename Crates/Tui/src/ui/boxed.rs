//! A bordered box over the middle of the page, for dialogs and lists that need a key press.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};

use crate::hit::{HitMap, Target};
use crate::num::to_u16;

/// Draws the box over `area` with `body` inside it, wrapped to fit, and returns the `rows_below` rows
/// under the text, where the caller puts its buttons or its hint. A click outside the box is a
/// [`Target::Dismiss`], a click inside it is swallowed.
pub(super) fn draw(
    frame: &mut Frame,
    hits: &mut HitMap,
    area: Rect,
    title: &str,
    color: Color,
    body: Vec<Line<'static>>,
    rows_below: u16,
) -> Rect {
    hits.push(area, Target::Dismiss);
    let width = 62.min(area.width);
    // Borders and the margin take four lines; long lines wrap inside the text width.
    let text_width = usize::from(width.saturating_sub(4)).max(1);
    let wrapped: usize = body
        .iter()
        .map(|line| line.width().div_ceil(text_width).max(1))
        .sum();
    let height = (to_u16(wrapped) + 4 + rows_below).max(8).min(area.height);
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
    let [text, below] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(rows_below)]).areas(inner);
    frame.render_widget(
        Paragraph::new(body).wrap(Wrap { trim: true }),
        text.inner(Margin::new(1, 1)),
    );
    below
}
