//! Backups of what a run replaced or removed, and `undo`.

mod clock;
mod error;
mod pathjson;
mod store;
mod tree;
mod undo;

pub use clock::{describe as describe_run, now_utc};
pub use error::{BackupError, UndoError};
pub use store::{
    Backups, Change, Entry, Finished, KEEP_RUNS, LoadedEntry, LoadedRun, Run, RunKind,
};
pub use undo::{Filter, Restored, UndoReport, Undone, undo};

#[cfg(test)]
mod apply_tests;
#[cfg(test)]
mod delete_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod undo_tests;
