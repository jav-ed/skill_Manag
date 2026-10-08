//! Failures of the backup store and of `undo`.

use std::path::PathBuf;

use crate::Hint;

#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("cannot read the backup note {path}: {reason}")]
    Note { path: PathBuf, reason: String },
    #[error("no backup run named {id:?}")]
    NoSuchRun { id: String },
    #[error("there is no backup to restore")]
    Empty,
    #[error("the backup note {path} is not usable: {reason}")]
    BadEntry { path: PathBuf, reason: String },
    #[error("{path} is a symlink; skill folders behind a link are never touched")]
    LinkedParent { path: PathBuf },
    #[error("{path} is not a project folder any more")]
    ProjectGone { path: PathBuf },
    #[error("{path} is not a folder")]
    NotAFolder { path: PathBuf },
    #[error("done, but {path} could not be removed afterwards: {reason}")]
    Leftover { path: PathBuf, reason: String },
}

/// Why one entry of an undo failed.
#[derive(Debug, thiserror::Error)]
pub enum UndoError {
    #[error(transparent)]
    Backup(#[from] BackupError),
    #[error(transparent)]
    Apply(#[from] crate::apply::ApplyError),
}

impl Hint for UndoError {
    fn hint(&self) -> Option<String> {
        match self {
            Self::Backup(e) => e.hint(),
            Self::Apply(e) => e.hint(),
        }
    }
}

impl Hint for BackupError {
    fn hint(&self) -> Option<String> {
        match self {
            Self::NoSuchRun { .. } => Some("`skillmirror history` lists the runs".to_string()),
            Self::Empty => {
                Some("only runs that changed or removed something leave a backup".to_string())
            }
            Self::Leftover { path, .. } => Some(format!("delete {} by hand", path.display())),
            _ => None,
        }
    }
}
