//! Undoing an entry about a skill folder of the vault. Like any other folder, what the undo replaces or
//! removes is saved again in the undo run, so undoing twice is a redo. The vault is a git repository whose
//! index drives what a sync copies, so the restored folder is staged again.

use std::path::Path;

use super::store::{Change, LoadedEntry, Run};
use super::tree::copy_tree;
use super::{BackupError, UndoError, Undone};
use crate::apply::{discard, exchange, place_new};
use crate::ops::validate_name;

enum Current {
    Missing,
    Folder,
}

pub(super) fn restore_vault_skill(
    entry: &LoadedEntry,
    new_run: Option<&Run>,
    index: usize,
    token: &str,
) -> Result<Undone, UndoError> {
    let group = &entry.entry.project;
    let name = &entry.entry.skill;
    validate_name(name).map_err(|e| BackupError::BadEntry {
        path: entry.dir.clone(),
        reason: e.to_string(),
    })?;
    if !fs_err::metadata(group).is_ok_and(|m| m.is_dir()) {
        return Err(BackupError::ProjectGone {
            path: group.clone(),
        }
        .into());
    }
    let path = group.join(name);
    let current = current_state(&path)?;
    match (entry.entry.change, &current) {
        (Change::Created, Current::Missing) => {
            // Nothing is left to undo, so the note goes too (see `restore` for project skills).
            if new_run.is_some() {
                spend(entry)?;
            }
            Ok(Undone::AlreadyGone)
        }
        (Change::Created, Current::Folder) => {
            if let Some(run) = new_run {
                remove_created(entry, &path, run, index, token)?;
            }
            Ok(Undone::Removed)
        }
        (Change::Updated | Change::Deleted, _) => {
            if !entry.has_tree {
                return Err(BackupError::BadEntry {
                    path: entry.dir.clone(),
                    reason: "the saved folder is missing".to_string(),
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
        Ok(meta) if meta.is_dir() => Ok(Current::Folder),
        Ok(_) => Err(BackupError::NotAFolder {
            path: path.to_path_buf(),
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Current::Missing),
        Err(e) => Err(e.into()),
    }
}

/// Takes the folder a run created out of the vault and keeps it in the undo run.
fn remove_created(
    entry: &LoadedEntry,
    path: &Path,
    run: &Run,
    index: usize,
    token: &str,
) -> Result<(), UndoError> {
    let group = &entry.entry.project;
    let name = &entry.entry.skill;
    let trash = group.join(format!(".trash-{token}-{index}"));
    fs_err::rename(path, &trash).map_err(BackupError::from)?;
    match run.keep_vault_skill(index, group, name, Change::Deleted, &trash) {
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
    restage(group, name)?;
    spend(entry)
}

/// Copies the saved folder into place. The saved copy stays until the new one is in.
fn put_back(
    entry: &LoadedEntry,
    path: &Path,
    current: &Current,
    run: &Run,
    index: usize,
    token: &str,
) -> Result<(), UndoError> {
    let group = &entry.entry.project;
    let name = &entry.entry.skill;
    let stage = group.join(format!(".stage-{token}-{index}"));
    if let Err(cause) = copy_tree(&entry.tree(), &stage) {
        return Err(discard(&stage, cause.into()).into());
    }
    match current {
        Current::Missing => {
            if let Err(cause) = run.record_created_vault_skill(index, group, name) {
                return Err(discard(&stage, cause.into()).into());
            }
            if let Err(cause) = place_new(&stage, path) {
                drop(run.forget(index));
                return Err(discard(&stage, cause).into());
            }
        }
        Current::Folder => {
            if let Err(cause) = exchange(&stage, path) {
                return Err(discard(&stage, cause).into());
            }
            match run.keep_vault_skill(index, group, name, Change::Updated, &stage) {
                Ok(None) => {}
                Ok(Some(cause)) => {
                    return Err(BackupError::Leftover {
                        path: stage,
                        reason: cause.to_string(),
                    }
                    .into());
                }
                Err(cause) => {
                    // No backup, no restore: the folder that was there goes back.
                    exchange(&stage, path)?;
                    return Err(discard(&stage, cause.into()).into());
                }
            }
        }
    }
    restage(group, name)?;
    spend(entry)
}

/// The index has to describe the folder that is back, or the next sync would copy the other one.
fn restage(group: &Path, name: &str) -> Result<(), BackupError> {
    crate::git::stage_folder(group, name).map_err(|reason| BackupError::Unstaged {
        path: group.join(name),
        reason,
    })
}

/// The entry has been restored; its saved folder is not needed any more.
fn spend(entry: &LoadedEntry) -> Result<(), UndoError> {
    fs_err::remove_dir_all(&entry.dir).map_err(|e| {
        BackupError::Leftover {
            path: entry.dir.clone(),
            reason: e.to_string(),
        }
        .into()
    })
}
