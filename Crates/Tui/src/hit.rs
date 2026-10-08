//! Hit map: rectangles recorded while drawing and queried while handling the mouse, so the
//! clickable regions can never disagree with what is on screen.

use ratatui::layout::{Position, Rect};

/// What a screen region means. One variant per kind of clickable thing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Target {
    HeaderBack,
    HeaderLink,
    MenuItem(usize),
    /// Index into the filtered list, not the viewport row.
    Row(usize),
    /// The whole list body, so the wheel works over gaps.
    ListArea,
    ScrollTrack,
    PickerRow(usize),
    PickerUp,
    PickerSelect,
    PickerHidden,
    /// A row of the mandatory checklist.
    CheckRow(usize),
    /// A button of a dialog: the first or the second one.
    Button(usize),
    /// Inside an overlay: swallow the event.
    Overlay,
    /// Behind an overlay: a click closes it.
    Dismiss,
}

/// Regions in paint order. The last region pushed is the topmost one.
#[derive(Debug, Default, Clone)]
pub(crate) struct HitMap {
    entries: Vec<(Rect, Target)>,
}

impl HitMap {
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }

    pub(crate) fn push(&mut self, rect: Rect, target: Target) {
        if rect.area() > 0 {
            self.entries.push((rect, target));
        }
    }

    /// The topmost region that contains the position.
    pub(crate) fn at(&self, pos: Position) -> Option<Target> {
        self.entries
            .iter()
            .rev()
            .find(|(rect, _)| rect.contains(pos))
            .map(|(_, target)| *target)
    }

    /// The rectangle of the topmost region with this target.
    pub(crate) fn rect_of(&self, target: Target) -> Option<Rect> {
        self.entries
            .iter()
            .rev()
            .find(|(_, t)| *t == target)
            .map(|(rect, _)| *rect)
    }
}
