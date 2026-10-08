//! An OSC 8 hyperlink as a widget. The whole escape sequence sits in the first cell and its
//! on-screen width is forced, so the diff renderer neither splits nor miscounts it.

use std::num::NonZeroU16;

use ratatui::buffer::{Buffer, CellDiffOption};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::Widget;

pub(super) struct Link<'a> {
    pub(crate) text: &'a str,
    pub(crate) url: &'a str,
    pub(crate) style: Style,
}

impl Widget for Link<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let width = Span::raw(self.text).width().min(usize::from(area.width));
        let Some(width) = u16::try_from(width).ok().and_then(NonZeroU16::new) else {
            return;
        };
        buf.set_stringn(
            area.x,
            area.y,
            self.text,
            usize::from(area.width),
            self.style,
        );
        let sequence = format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", self.url, self.text);
        if let Some(cell) = buf.cell_mut((area.x, area.y)) {
            cell.set_symbol(&sequence)
                .set_diff_option(CellDiffOption::ForcedWidth(width));
        }
    }
}
