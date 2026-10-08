//! The cursor and the scroll window over a filtered list. Pure logic, no drawing.

use crate::filter::Hit;
use crate::num::step;

#[derive(Debug, Default)]
pub(crate) struct ListView {
    /// The visible items: index into the item list and matched characters of the name.
    pub(crate) filtered: Vec<Hit>,
    pub(crate) cursor: usize,
    pub(crate) offset: usize,
    /// Rows that fit on screen; written by the renderer.
    pub(crate) viewport: usize,
}

impl ListView {
    pub(crate) fn set(&mut self, hits: Vec<Hit>) {
        self.filtered = hits;
        self.cursor = 0;
        self.offset = 0;
    }

    pub(crate) fn len(&self) -> usize {
        self.filtered.len()
    }

    /// The item index under the cursor.
    pub(crate) fn current(&self) -> Option<usize> {
        self.filtered.get(self.cursor).map(|(item, _)| *item)
    }

    pub(crate) fn move_by(&mut self, delta: isize) {
        self.cursor = step(self.cursor, delta, self.len());
        self.ensure_visible();
    }

    pub(crate) fn page(&mut self, direction: isize) {
        let rows = isize::try_from(self.viewport.max(1)).unwrap_or(isize::MAX);
        self.move_by(direction.saturating_mul(rows));
    }

    /// Scrolls the cursor into the window of `viewport` rows.
    pub(crate) fn ensure_visible(&mut self) {
        let rows = self.viewport.max(1);
        if self.cursor < self.offset {
            self.offset = self.cursor;
        } else if self.cursor >= self.offset + rows {
            self.offset = self.cursor + 1 - rows;
        }
    }

    /// Mouse wheel: moves the window, and the cursor follows so keys continue from what is shown.
    pub(crate) fn scroll_by(&mut self, delta: isize) {
        let span = self.len().saturating_sub(self.viewport);
        self.offset = step(self.offset, delta, span + 1);
        let last = (self.offset + self.viewport.max(1)).saturating_sub(1);
        self.cursor = self.cursor.clamp(self.offset, last.max(self.offset));
        self.cursor = self.cursor.min(self.len().saturating_sub(1));
    }

    /// A click `from_top` rows below the top of a scroll track `height` rows tall.
    pub(crate) fn jump(&mut self, from_top: usize, height: usize) {
        let span = self.len().saturating_sub(self.viewport);
        self.offset = (from_top * span / height.max(1)).min(span);
        self.scroll_by(0);
    }
}
