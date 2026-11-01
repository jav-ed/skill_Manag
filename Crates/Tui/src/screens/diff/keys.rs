//! The keys of the changes page, and the jumps the keys and the buttons share.

use super::{CHANGE_MARGIN, DiffAction, DiffView, FOLDS, MODE, NEXT, PREVIOUS, ViewMode};
use crate::binding::{
    BACK, BOTTOM, CONTEXT_KEY, DOWN, HALF_DOWN, HALF_UP, MODE_KEY, NEXT_CHANGE, NEXT_FILE,
    NEXT_GROUP, PAGE_DOWN, PAGE_UP, PREVIOUS_CHANGE, PREVIOUS_FILE, PREVIOUS_GROUP, TOP, UP, VIEW,
};
use crate::input::{Code, Key};

/// What a jump goes between.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Stops {
    Changes,
    Files,
    Groups,
}

impl DiffView {
    pub(crate) fn on_key(&mut self, key: Key) -> DiffAction {
        if BACK.matches(key) || key.code == Code::Esc || VIEW.matches(key) {
            return DiffAction::Leave;
        }
        let page = rows(self.viewport.saturating_sub(1).max(1));
        let half = rows((self.viewport / 2).max(1));
        if UP.matches(key) {
            self.scroll_by(-1);
        } else if DOWN.matches(key) {
            self.scroll_by(1);
        } else if PAGE_UP.matches(key) {
            self.scroll_by(-page);
        } else if PAGE_DOWN.matches(key) {
            self.scroll_by(page);
        } else if HALF_UP.matches(key) {
            self.scroll_by(-half);
        } else if HALF_DOWN.matches(key) {
            self.scroll_by(half);
        } else if TOP.matches(key) {
            self.scroll = 0;
        } else if BOTTOM.matches(key) {
            self.scroll = self.max_scroll();
        } else if NEXT_CHANGE.matches(key) {
            self.jump(Stops::Changes, true);
        } else if PREVIOUS_CHANGE.matches(key) {
            self.jump(Stops::Changes, false);
        } else if NEXT_FILE.matches(key) {
            self.jump(Stops::Files, true);
        } else if PREVIOUS_FILE.matches(key) {
            self.jump(Stops::Files, false);
        } else if NEXT_GROUP.matches(key) {
            self.jump(Stops::Groups, true);
        } else if PREVIOUS_GROUP.matches(key) {
            self.jump(Stops::Groups, false);
        } else if MODE_KEY.matches(key) {
            self.press(MODE);
        } else if CONTEXT_KEY.matches(key) {
            self.press(FOLDS);
        }
        DiffAction::None
    }

    /// What a button of the header does; the keys with the same names do the same.
    pub(super) fn press(&mut self, button: usize) {
        match button {
            PREVIOUS => self.jump(Stops::Changes, false),
            NEXT => self.jump(Stops::Changes, true),
            MODE => {
                self.mode = match self.mode {
                    ViewMode::Split => ViewMode::Inline,
                    ViewMode::Inline => ViewMode::Split,
                };
            }
            FOLDS => {
                self.fold = !self.fold;
                self.opened.clear();
            }
            _ => {}
        }
    }

    /// Scrolls to the next (or previous) stop that is not where the page is already. A page that is at the
    /// last stop stays: it does not start again at the first.
    pub(super) fn jump(&mut self, stops: Stops, forward: bool) {
        let Some(layout) = &self.layout else {
            return;
        };
        let (tops, margin) = match stops {
            Stops::Changes => (&layout.changes, CHANGE_MARGIN),
            Stops::Files => (&layout.files, 0),
            Stops::Groups => (&layout.groups, 0),
        };
        let places = tops.iter().map(|top| top.saturating_sub(margin));
        let target = if forward {
            places.clone().find(|place| *place > self.scroll)
        } else {
            places.rev().find(|place| *place < self.scroll)
        };
        if let Some(place) = target {
            self.scroll_to(place);
        }
    }
}

fn rows(n: usize) -> isize {
    isize::try_from(n).unwrap_or(isize::MAX)
}
