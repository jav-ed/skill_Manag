//! Putting a finished folder at its destination, and keeping what it replaces.

use std::path::{Path, PathBuf};

use super::{ApplyError, Leftover, swap};
use crate::backup::{Change, Run};
use crate::plan::inspect::{self, Snapshot};
use crate::scan::Target;

/// Where the old copy goes instead of the bin.
#[derive(Clone, Copy)]
pub(crate) struct Keep<'a> {
    pub(crate) run: &'a Run,
    pub(crate) index: usize,
    pub(crate) target: &'a Target,
}

/// What the new folder takes the place of.
#[derive(Clone, Copy)]
pub(crate) enum Replaces<'a> {
    /// The destination does not exist.
    Nothing,
    /// The destination is a folder. When `expected` is given it must still look like that.
    Folder { expected: Option<&'a Snapshot> },
}

/// Moves the finished folder `stage` to `dest`. On `Err` the destination is as it was and `stage` is gone.
///
/// An old copy is moved to the backup store when `keep` is given, and deleted otherwise. Failing to
/// delete or move it is a [`Leftover`], not an error: the new copy is in place.
pub(crate) fn place(
    stage: &Path,
    dest: &Path,
    replaces: Replaces<'_>,
    keep: Option<Keep<'_>>,
) -> Result<Option<Leftover>, ApplyError> {
    match replaces {
        Replaces::Nothing => create(stage, dest, keep),
        Replaces::Folder { expected } => replace(stage, dest, expected, keep),
    }
}

fn create(
    stage: &Path,
    dest: &Path,
    keep: Option<Keep<'_>>,
) -> Result<Option<Leftover>, ApplyError> {
    if let Some(k) = keep {
        k.run
            .record_created(k.index, k.target)
            .map_err(|cause| discard(stage, cause.into()))?;
    }
    if let Err(cause) = swap::place_new(stage, dest) {
        if let Some(k) = keep {
            // A note about a skill that was never created only makes `undo` say "already gone".
            drop(k.run.forget(k.index));
        }
        return Err(discard(stage, cause));
    }
    Ok(None)
}

/// Swaps the new copy in, then checks that the copy it replaced is the one that was planned against.
/// If someone changed it meanwhile the swap is undone, so their edit survives.
fn replace(
    stage: &Path,
    dest: &Path,
    expected: Option<&Snapshot>,
    keep: Option<Keep<'_>>,
) -> Result<Option<Leftover>, ApplyError> {
    swap::exchange(stage, dest).map_err(|cause| discard(stage, cause))?;
    // After the exchange the stage path holds the previous copy.
    if let Some(expected) = expected {
        let seen = inspect::walk(stage);
        if !matches!(&seen, Ok(now) if now == expected) {
            swap::exchange(stage, dest)?;
            let cause = match seen {
                Ok(_) => ApplyError::DestinationChanged {
                    dest: dest.to_path_buf(),
                },
                Err(e) => ApplyError::Io(e),
            };
            return Err(discard(stage, cause));
        }
    }
    let Some(k) = keep else {
        return Ok(fs_err::remove_dir_all(stage)
            .err()
            .map(|e| leftover(stage, &e)));
    };
    match k.run.keep(k.index, k.target, Change::Updated, stage) {
        Ok(left) => Ok(left.map(|e| leftover(stage, &e))),
        Err(cause) => {
            // No backup, no update: the old copy goes back.
            swap::exchange(stage, dest)?;
            Err(discard(stage, cause.into()))
        }
    }
}

fn leftover(stage: &Path, cause: &std::io::Error) -> Leftover {
    Leftover {
        path: stage.to_path_buf(),
        reason: cause.to_string(),
    }
}

/// Removes a half-built stage directory; if that fails too, says so in the error.
pub(crate) fn discard(stage_dir: &Path, cause: ApplyError) -> ApplyError {
    match fs_err::remove_dir_all(stage_dir) {
        Ok(()) => cause,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => cause,
        Err(e) => ApplyError::LeftBehind {
            stage: PathBuf::from(stage_dir),
            cause: cause.to_string(),
            cleanup: e.to_string(),
        },
    }
}
