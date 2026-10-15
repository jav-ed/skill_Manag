//! The pane of the pages that have one (skills, sync, push, delete): what the row under the cursor is, and
//! which projects it concerns.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

use crate::hit::{HitMap, Target};
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

/// A row of the pane after wrapping.
struct Row {
    text: String,
    tone: Tone,
}

/// Draws the pane of the row under the cursor, wrapped to the pane and scrolled by `work.pane_scroll`.
pub(super) fn draw(
    frame: &mut Frame,
    hits: &mut HitMap,
    work: &mut Work,
    area: Rect,
    place: Place,
) {
    let borders = match place {
        Place::Beside => Borders::LEFT,
        Place::Below => Borders::TOP,
    };
    let block = Block::default()
        .borders(borders)
        .border_style(theme::muted())
        .padding(Padding::left(1));
    let inner = block.inner(area);
    hits.push(area, Target::Pane);
    // Another row or another view starts at the top again.
    let shown = (work.view.current(), work.pane);
    if work.pane_key != shown {
        work.pane_key = shown;
        work.pane_scroll = 0;
    }
    let rows: Vec<Row> = work
        .view
        .current()
        .and_then(|index| work.items.get(index))
        .map(|item| match &item.views {
            Some(views) => views.of(work.pane),
            None => &item.card,
        })
        .unwrap_or_default()
        .iter()
        .flat_map(|note| wrap(note, usize::from(inner.width)))
        .collect();
    let (lines, scroll) = fitted(&rows, usize::from(inner.height), work.pane_scroll);
    work.pane_scroll = scroll;
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

/// The rows from `scroll` on that fit in `height` rows. When some do not, the last row says how many are
/// left out, so a long pane never looks complete when it is not. `scroll` is held to what the rows allow;
/// the scroll that was used comes back.
fn fitted(rows: &[Row], height: usize, scroll: usize) -> (Vec<Line<'static>>, usize) {
    let scroll = scroll.min(rows.len().saturating_sub(height));
    let visible = rows.get(scroll..).unwrap_or_default();
    if visible.len() <= height {
        return (visible.iter().map(line).collect(), scroll);
    }
    // One row stays free for the note about what is left out.
    let room = height.saturating_sub(1);
    let mut out: Vec<Line<'static>> = visible.iter().take(room).map(line).collect();
    out.push(Line::from(Span::styled(
        format!("… and {} more lines", visible.len() - room),
        theme::muted(),
    )));
    (out, scroll)
}

/// `note` cut into rows of at most `width` characters. A row breaks after a space or a `/`, so a path
/// falls apart at its folders and not in the middle of a name; a name longer than the row is cut. The
/// rows after the first are indented by two, except for headings and plain text, so a wrapped entry
/// still reads as one.
fn wrap(note: &Note, width: usize) -> Vec<Row> {
    let width = width.max(1);
    let indent = match note.tone {
        Tone::Heading | Tone::Plain => 0,
        Tone::Change | Tone::Quiet | Tone::Problem => 2.min(width - 1),
    };
    let mut lines = vec![String::new()];
    for chunk in note.text.split_inclusive([' ', '/']) {
        let mut rest = chunk;
        loop {
            let (used, continued) = lines
                .last()
                .map_or((0, false), |l| (l.chars().count(), lines.len() > 1));
            let bare = used <= if continued { indent } else { 0 };
            // Spaces do not start a continuation row.
            if continued && bare && rest.trim().is_empty() {
                break;
            }
            if rest.chars().count() <= width - used {
                if let Some(last) = lines.last_mut() {
                    last.push_str(rest);
                }
                break;
            }
            if !bare {
                lines.push(" ".repeat(indent));
                continue;
            }
            // Nothing else is on the row and the chunk is still too long: cut it.
            let cut = rest
                .char_indices()
                .nth(width - used)
                .map_or(rest.len(), |(at, _)| at);
            let (head, tail) = rest.split_at(cut);
            if let Some(last) = lines.last_mut() {
                last.push_str(head);
            }
            lines.push(" ".repeat(indent));
            rest = tail;
        }
    }
    lines
        .into_iter()
        .map(|text| Row {
            text: text.trim_end().to_string(),
            tone: note.tone,
        })
        .collect()
}

fn line(row: &Row) -> Line<'static> {
    let style = match row.tone {
        Tone::Heading => theme::bold(),
        Tone::Plain => ratatui::style::Style::new(),
        Tone::Change => theme::accent(),
        Tone::Quiet => theme::muted(),
        Tone::Problem => theme::error(),
    };
    Line::from(Span::styled(row.text.clone(), style))
}
