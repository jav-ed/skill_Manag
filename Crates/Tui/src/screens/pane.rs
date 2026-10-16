//! The state of the pane beside the list of the sync, push, delete and skills pages.

use tui_input::Input as TextInput;

use crate::items::projects::{PaneView, PathStyle};

#[derive(Debug, Default)]
pub(crate) struct PaneState {
    /// The view of the pane on the sync and push pages; it stays while the cursor moves.
    pub(crate) view: PaneView,
    /// How the pane writes a project.
    pub(crate) paths: PathStyle,
    /// The filter of its projects (`f`), and whether the keys go to it.
    pub(crate) filter: TextInput,
    pub(crate) filtering: bool,
    /// How far the pane is scrolled.
    pub(crate) scroll: usize,
    /// The row, view and filter the scroll belongs to: another one starts at the top again.
    shown: (Option<usize>, PaneView, String),
}

impl PaneState {
    /// Called when the pane is drawn, with the row under the cursor.
    pub(crate) fn follow(&mut self, row: Option<usize>) {
        let now = (row, self.view, self.filter.value().to_string());
        if self.shown != now {
            self.shown = now;
            self.scroll = 0;
        }
    }
}
