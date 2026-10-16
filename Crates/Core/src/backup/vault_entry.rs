//! Backup entries about a skill folder of the vault: saved like a project's skill folder, but the entry
//! names the vault folder the skill lies in, so the skill is `project/skill`.

use std::path::Path;

use super::BackupError;
use super::store::{Change, Entry, Run, Subject, TREE_DIR};
use super::tree::move_tree;

impl Run {
    /// Notes that the skill folder is about to be created, so `undo` can remove it. Call before creating it.
    pub(crate) fn record_created_vault_skill(
        &self,
        index: usize,
        group_dir: &Path,
        name: &str,
    ) -> Result<(), BackupError> {
        self.write_vault_note(index, group_dir, name, Change::Created)
    }

    /// Moves `old`, the skill folder the run just replaced or removed, into the store.
    ///
    /// On `Err` nothing was stored and `old` is untouched. `Ok(Some(error))` means it is stored but `old`
    /// could not be removed afterwards (only possible across filesystems).
    pub(crate) fn keep_vault_skill(
        &self,
        index: usize,
        group_dir: &Path,
        name: &str,
        change: Change,
        old: &Path,
    ) -> Result<Option<std::io::Error>, BackupError> {
        self.write_vault_note(index, group_dir, name, change)?;
        let slot = self.dir.join(index.to_string());
        match move_tree(old, &slot.join(TREE_DIR)) {
            Ok(left) => Ok(left),
            Err(cause) => {
                drop(self.forget(index));
                Err(cause.into())
            }
        }
    }

    fn write_vault_note(
        &self,
        index: usize,
        group_dir: &Path,
        name: &str,
        change: Change,
    ) -> Result<(), BackupError> {
        self.write_note(
            index,
            &Entry {
                kind: self.kind(),
                project: group_dir.to_path_buf(),
                skill: name.to_string(),
                change,
                subject: Subject::VaultSkill,
            },
        )
    }
}
