//! Undoing an entry about a project's AGENTS.md. Like a skill folder, the file that is replaced or removed
//! is saved again in the undo run, so undoing twice is a redo.

use std::path::{Path, PathBuf};

use super::store::{Change, FILE_SLOT, LoadedEntry, Run};
use super::{BackupError, UndoError, Undone};
use crate::agents::FILE_NAME;
use crate::apply::{exchange, place_new};

enum Current {
    Missing,
    File,
}

pub(super) fn restore_file(
    entry: &LoadedEntry,
    new_run: Option<&Run>,
    index: usize,
    token: &str,
) -> Result<Undone, UndoError> {
    let project = &entry.entry.project;
    if !fs_err::metadata(project).is_ok_and(|m| m.is_dir()) {
        return Err(BackupError::ProjectGone {
            path: project.clone(),
        }
        .into());
    }
    let path = project.join(FILE_NAME);
    let current = current_state(&path)?;
    match (entry.entry.change, &current) {
        (Change::Created, Current::Missing) => {
            // Nothing is left to undo, so the note goes too (see `restore` for skills).
            if new_run.is_some() {
                spend(entry)?;
            }
            Ok(Undone::AlreadyGone)
        }
        (Change::Created, Current::File) => {
            if let Some(run) = new_run {
                remove_created(entry, &path, run, index, token)?;
            }
            Ok(Undone::Removed)
        }
        (Change::Updated | Change::Deleted, _) => {
            if !entry.has_tree {
                return Err(BackupError::BadEntry {
                    path: entry.dir.clone(),
                    reason: "the saved file is missing".to_string(),
                }
                .into());
            }
            if let Some(run) = new_run {
                put_back(entry, &path, &current, run, index, token)?;
            }
            Ok(Undone::Restored)
        }
    }
}

fn current_state(path: &Path) -> Result<Current, BackupError> {
    match fs_err::symlink_metadata(path) {
        Ok(meta) if meta.is_file() => Ok(Current::File),
        Ok(_) => Err(BackupError::NotAFile {
            path: path.to_path_buf(),
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Current::Missing),
        Err(e) => Err(e.into()),
    }
}

/// Takes the file a run created out of the project and keeps it in the undo run.
fn remove_created(
    entry: &LoadedEntry,
    path: &Path,
    run: &Run,
    index: usize,
    token: &str,
) -> Result<(), UndoError> {
    let project = &entry.entry.project;
    let trash = project.join(format!(".{FILE_NAME}.trash-{token}-{index}"));
    fs_err::rename(path, &trash).map_err(BackupError::from)?;
    match run.keep_file(index, project, Change::Deleted, &trash) {
        Ok(None) => {}
        Ok(Some(cause)) => {
            return Err(BackupError::Leftover {
                path: trash,
                reason: cause.to_string(),
            }
            .into());
        }
        Err(cause) => {
            fs_err::rename(&trash, path).map_err(BackupError::from)?;
            return Err(cause.into());
        }
    }
    spend(entry)
}

/// Copies the saved file into place. The saved copy stays until the new one is in.
fn put_back(
    entry: &LoadedEntry,
    path: &Path,
    current: &Current,
    run: &Run,
    index: usize,
    token: &str,
) -> Result<(), UndoError> {
    let project = &entry.entry.project;
    let stage = project.join(format!(".{FILE_NAME}.stage-{token}-{index}"));
    fs_err::copy(entry.dir.join(FILE_SLOT), &stage).map_err(BackupError::from)?;
    match current {
        Current::Missing => {
            if let Err(cause) = run.record_created_file(index, project) {
                return Err(discard(&stage, cause.into()));
            }
            if let Err(cause) = place_new(&stage, path) {
                drop(run.forget(index));
                return Err(discard(&stage, cause.into()));
            }
        }
        Current::File => {
            if let Err(cause) = exchange(&stage, path) {
                return Err(discard(&stage, cause.into()));
            }
            match run.keep_file(index, project, Change::Updated, &stage) {
                Ok(None) => {}
                Ok(Some(cause)) => {
                    return Err(BackupError::Leftover {
                        path: stage,
                        reason: cause.to_string(),
                    }
                    .into());
                }
                Err(cause) => {
                    // No backup, no restore: the file that was there goes back.
                    exchange(&stage, path)?;
                    return Err(discard(&stage, cause.into()));
                }
            }
        }
    }
    spend(entry)
}

/// Removes a half-made stage file, and reports the original cause.
fn discard(stage: &PathBuf, cause: UndoError) -> UndoError {
    drop(fs_err::remove_file(stage));
    cause
}

/// The entry has been restored; its saved file is not needed any more.
fn spend(entry: &LoadedEntry) -> Result<(), UndoError> {
    fs_err::remove_dir_all(&entry.dir).map_err(|e| {
        BackupError::Leftover {
            path: entry.dir.clone(),
            reason: e.to_string(),
        }
        .into()
    })
}
