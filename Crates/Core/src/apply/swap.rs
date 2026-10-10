//! Atomic placement of a staged skill with `renameat2`.

use std::path::Path;

use rustix::fs::{CWD, RenameFlags, renameat_with};

use super::ApplyError;

/// Puts `stage` at `dest`, which must not exist. Fails instead of replacing anything that appeared meanwhile.
pub(crate) fn place_new(stage: &Path, dest: &Path) -> Result<(), ApplyError> {
    rename(stage, dest, RenameFlags::NOREPLACE)
}

/// Swaps `stage` and `dest` in one step. Afterwards `stage` holds the old copy.
pub(crate) fn exchange(stage: &Path, dest: &Path) -> Result<(), ApplyError> {
    rename(stage, dest, RenameFlags::EXCHANGE)
}

fn rename(stage: &Path, dest: &Path, flags: RenameFlags) -> Result<(), ApplyError> {
    renameat_with(CWD, stage, CWD, dest, flags).map_err(|errno| ApplyError::Swap {
        stage: stage.to_path_buf(),
        dest: dest.to_path_buf(),
        source: errno.into(),
    })
}
