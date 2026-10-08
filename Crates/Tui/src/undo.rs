//! The history page's data comes from the engine; the page only shows it.

pub(crate) use skillmirror_core::ops::{
    RunRow, Step, UndoLine, UndoView, do_undo, list_runs, plan_undo,
};
