//! Failures of the project scan.

use std::path::PathBuf;

use crate::Hint;
use crate::brand;

#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("scan root {path} does not exist")]
    RootMissing { path: PathBuf },
    #[error("scan root {path} is not a directory")]
    RootNotADirectory { path: PathBuf },
    #[error("exclude_paths entry {entry} climbs above the filesystem root")]
    ExcludePathEscapes { entry: PathBuf },
    #[error("exclude_paths entry {entry} does not exist")]
    ExcludePathMissing { entry: PathBuf },
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

impl Hint for ScanError {
    fn hint(&self) -> Option<String> {
        match self {
            Self::RootMissing { .. } | Self::RootNotADirectory { .. } => Some(format!(
                "pass --root, set {}, or fix `root:` in <vault>/config.yaml",
                brand::ENV_ROOT
            )),
            Self::ExcludePathMissing { .. } => Some(
                "fix or remove the entry in <vault>/config.yaml; a path that matches nothing excludes nothing"
                    .to_string(),
            ),
            _ => None,
        }
    }
}

/// Something the scan could not read. Reported to the user, never swallowed, never fatal on its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanIssue {
    pub path: PathBuf,
    pub message: String,
}
