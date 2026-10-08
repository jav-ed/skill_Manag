//! Backups of what a run replaced or removed, and `undo`.

mod clock;
mod error;
mod store;
mod tree;
mod undo;

pub use clock::describe as describe_run;
pub use error::{BackupError, UndoError};
pub use store::{
    Backups, Change, Entry, Finished, KEEP_RUNS, LoadedEntry, LoadedRun, Run, RunKind,
};
pub use undo::{Filter, Restored, UndoReport, Undone, undo};

#[cfg(test)]
mod tests;
#[cfg(test)]
mod undo_tests;
