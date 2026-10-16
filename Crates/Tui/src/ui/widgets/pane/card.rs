//! The card of the skills page, and where the pane of every page that has one sits.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

use super::text::{fitted, wrap};

use crate::hit::{HitMap, Target};
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
pub(in crate::ui) enum Place {
    Beside,
    Below,
}

/// Splits the page into the list and, when there is room for it, the card.
pub(in crate::ui) fn split(area: Rect) -> (Rect, Option<(Rect, Place)>) {
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

/// Draws the card of the skill under the cursor, wrapped to the pane and scrolled by `work.pane.scroll`.
pub(in crate::ui) fn draw(
    frame: &mut Frame,
    hits: &mut HitMap,
    work: &mut Work,
    area: Rect,
    place: Place,
) {
    let block = pane_block(place);
    let inner = block.inner(area);
    hits.push(area, Target::Pane);
    work.follow_pane();
    let rows: Vec<_> = work
        .view
        .current()
        .and_then(|index| work.items.get(index))
        .map(|item| item.card.as_slice())
        .unwrap_or_default()
        .iter()
        .flat_map(|note| wrap(note, usize::from(inner.width)))
        .collect();
    let (lines, scroll) = fitted(&rows, usize::from(inner.height), work.pane.scroll);
    work.pane.scroll = scroll;
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

/// The frame of a pane: a line on the side that faces the list.
pub(in crate::ui) fn pane_block(place: Place) -> Block<'static> {
    let borders = match place {
        Place::Beside => Borders::LEFT,
        Place::Below => Borders::TOP,
    };
    Block::default()
        .borders(borders)
        .border_style(theme::muted())
        .padding(Padding::left(1))
}
