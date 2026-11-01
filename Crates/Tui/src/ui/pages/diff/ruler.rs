//! The overview at the right edge: the whole page squeezed into the height of the body, with a mark where
//! lines were added or removed and a lighter stretch where the window is. A click or a drag scrolls there.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::hit::{HitMap, Target};
use crate::screens::diff::{DiffView, Mark};
use crate::theme;

pub(super) fn draw(frame: &mut Frame, hits: &mut HitMap, view: &DiffView, area: Rect) {
    let Some(layout) = view.layout.as_ref() else {
        return;
    };
    let height = usize::from(area.height);
    // A page that fits has nothing to scroll to, and the marks would only repeat what is in view.
    if height == 0 || layout.total <= view.viewport {
        return;
    }
    hits.push(area, Target::DiffRuler);
    // Each cell of the ruler stands for a stretch of the page; it shows what that stretch changes.
    let mut cells = vec![Mark::None; height];
    for placed in layout.items.iter().filter(|p| p.mark != Mark::None) {
        let first = placed.top * height / layout.total;
        let last = (placed.top + placed.height - 1) * height / layout.total;
        for cell in cells.iter_mut().skip(first).take(last + 1 - first) {
            *cell = cell.merge(placed.mark);
        }
    }
    let window_first = view.scroll * height / layout.total;
    let window_last =
        (view.scroll + view.viewport.max(1) - 1).min(layout.total - 1) * height / layout.total;
    let lines: Vec<Line> = cells
        .iter()
        .enumerate()
        .map(|(y, mark)| {
            let track = if (window_first..=window_last).contains(&y) {
                theme::RULER_WINDOW
            } else {
                theme::RULER_TRACK
            };
            let color = match mark {
                Mark::None => return Line::from(Span::styled(" ", Style::new().bg(track))),
                Mark::Added => theme::SUCCESS,
                Mark::Removed => theme::ERROR,
                Mark::Both => theme::WARNING,
            };
            Line::from(Span::styled("█", Style::new().fg(color).bg(track)))
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), area);
}
