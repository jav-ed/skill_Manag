//! Failures while writing a skill. The destination keeps its previous content unless the message says otherwise.

use std::path::PathBuf;

use crate::Hint;
use crate::backup::BackupError;

#[derive(Debug, thiserror::Error)]
pub enum ApplyError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("cannot swap {stage} into {dest}: {source}")]
    Swap {
        stage: PathBuf,
        dest: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot keep the old copy: {0}")]
    Backup(#[from] BackupError),
    #[error("cannot start the worker pool: {0}")]
    Pool(String),
    #[error("{cause}; the temporary copy {stage} could not be removed either: {cleanup}")]
    LeftBehind {
        stage: PathBuf,
        cause: String,
        cleanup: String,
    },
    #[error("{dest} changed after it was planned; nothing was written")]
    DestinationChanged { dest: PathBuf },
}

impl Hint for ApplyError {
    fn hint(&self) -> Option<String> {
        match self {
            Self::Swap { source, dest, .. } => match source.kind() {
                std::io::ErrorKind::PermissionDenied => Some(format!(
                    "no permission to write next to {}; check the folder's permissions",
                    dest.display()
                )),
                // EINVAL and ENOSYS are what filesystems without renameat2 flags answer.
                _ if matches!(source.raw_os_error(), Some(22 | 38 | 95)) => Some(
                    "the filesystem must support atomic directory exchange (renameat2)".to_string(),
                ),
                _ => None,
            },
            Self::LeftBehind { stage, .. } => Some(format!("delete {} by hand", stage.display())),
            Self::DestinationChanged { .. } => {
                Some("someone edited the folder meanwhile; run the command again to see the new difference".to_string())
            }
            _ => None,
        }
    }
}
