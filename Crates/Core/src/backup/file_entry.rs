//! Backup entries about a project's AGENTS.md: one file instead of a skill folder.

use std::path::Path;

use super::BackupError;
use super::store::{Change, Entry, FILE_SLOT, Run, Subject};
use super::tree::move_file;
use crate::agents::FILE_NAME;

impl Run {
    /// Notes that the file is about to be created, so `undo` can remove it. Call before creating it.
    pub(crate) fn record_created_file(
        &self,
        index: usize,
        project: &Path,
    ) -> Result<(), BackupError> {
        self.write_file_note(index, project, Change::Created)
    }

    /// Moves `old`, the file the run just replaced or removed, into the store.
    ///
    /// On `Err` nothing was stored and `old` is untouched. `Ok(Some(error))` means it is stored but `old`
    /// could not be removed afterwards (only possible across filesystems).
    pub(crate) fn keep_file(
        &self,
        index: usize,
        project: &Path,
        change: Change,
        old: &Path,
    ) -> Result<Option<std::io::Error>, BackupError> {
        self.write_file_note(index, project, change)?;
        let slot = self.dir.join(index.to_string());
        match move_file(old, &slot.join(FILE_SLOT)) {
            Ok(left) => Ok(left),
            Err(cause) => {
                drop(self.forget(index));
                Err(cause.into())
            }
        }
    }

    fn write_file_note(
        &self,
        index: usize,
        project: &Path,
        change: Change,
    ) -> Result<(), BackupError> {
        self.write_note(
            index,
            &Entry {
                kind: self.kind(),
                project: project.to_path_buf(),
                skill: FILE_NAME.to_string(),
                change,
                subject: Subject::Agents,
            },
        )
    }
}
