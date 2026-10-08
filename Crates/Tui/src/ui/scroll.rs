//! The scroll bar at the right edge of a list.

use ratatui::layout::Rect;
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::{Frame, symbols};

use crate::hit::{HitMap, Target};
use crate::screens::ListView;

/// Draws the bar over the last column of `area` and records it as a click target.
pub(super) fn bar(frame: &mut Frame, hits: &mut HitMap, view: &ListView, area: Rect) {
    let track = Rect::new(area.right().saturating_sub(1), area.y, 1, area.height);
    hits.push(track, Target::ScrollTrack);
    let mut state = ScrollbarState::new(view.len())
        .position(view.offset)
        .viewport_content_length(usize::from(area.height));
    let bar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None)
        .track_symbol(Some(symbols::line::VERTICAL));
    frame.render_stateful_widget(bar, track, &mut state);
}
