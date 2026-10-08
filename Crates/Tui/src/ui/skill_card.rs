//! The detail pane of the skills page: what the skill under the cursor is.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Wrap};

use crate::items::{Note, Tone};
use crate::screens::Work;
use crate::theme;

/// Width of the terminal from which the card sits beside the list.
const SIDE_FROM: u16 = 100;
/// Height of the terminal from which the card sits under the list.
const BELOW_FROM: u16 = 20;
/// Rows of the card under the list: half the page, but not less than this and not more than the maximum.
const BELOW_MIN: u16 = 9;
const BELOW_MAX: u16 = 14;

/// Where the card sits.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Place {
    Beside,
    Below,
}

/// Splits the page into the list and, when there is room for it, the card.
pub(super) fn split(area: Rect) -> (Rect, Option<(Rect, Place)>) {
    if area.width >= SIDE_FROM {
        let [list, card] =
            Layout::horizontal([Constraint::Fill(3), Constraint::Fill(2)]).areas(area);
        (list, Some((card, Place::Beside)))
    } else if area.height >= BELOW_FROM {
        let [list, card] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length((area.height / 2).clamp(BELOW_MIN, BELOW_MAX)),
        ])
        .areas(area);
        (list, Some((card, Place::Below)))
    } else {
        (area, None)
    }
}

pub(super) fn draw(frame: &mut Frame, work: &Work, area: Rect, place: Place) {
    let borders = match place {
        Place::Beside => Borders::LEFT,
        Place::Below => Borders::TOP,
    };
    let block = Block::default()
        .borders(borders)
        .border_style(theme::muted())
        .padding(Padding::left(1));
    let inner = block.inner(area);
    let lines = work
        .view
        .current()
        .and_then(|index| work.items.get(index))
        .map(|item| {
            fitted(
                &item.card,
                usize::from(inner.width),
                usize::from(inner.height),
            )
        })
        .unwrap_or_default();
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(block),
        area,
    );
}

/// The lines of the card that fit in `height` rows once they are wrapped to `width`. When some do not,
/// the last row says how many lines are left out, so a long card never looks complete when it is not.
fn fitted(card: &[Note], width: usize, height: usize) -> Vec<Line<'static>> {
    let rows = |note: &Note| note.text.chars().count().div_ceil(width.max(1)).max(1);
    let total: usize = card.iter().map(rows).sum();
    if total <= height {
        return card.iter().map(line).collect();
    }
    let mut used = 0;
    let mut out = Vec::new();
    for note in card {
        // One row stays free for the note about what was left out.
        if used + rows(note) + 1 > height {
            break;
        }
        used += rows(note);
        out.push(line(note));
    }
    let left = card.len() - out.len();
    out.push(line(&Note {
        text: format!("… and {left} more lines"),
        tone: Tone::Quiet,
    }));
    out
}

fn line(note: &Note) -> Line<'static> {
    let style = match note.tone {
        Tone::Heading => theme::bold(),
        Tone::Plain => ratatui::style::Style::new(),
        Tone::Change => theme::accent(),
        Tone::Quiet => theme::muted(),
        Tone::Problem => theme::error(),
    };
    Line::from(Span::styled(note.text.clone(), style))
}
