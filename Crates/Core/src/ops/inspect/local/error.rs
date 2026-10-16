//! Why the skills of a project could not be compared with the vault.

use std::path::PathBuf;

use crate::Hint;
use crate::scan::ScanError;

#[derive(Debug, thiserror::Error)]
pub enum LocalError {
    #[error("no project with a .agents/skills folder was found at {from}, above it or below it")]
    NoProject { from: PathBuf },
    #[error("{path} has no .agents/skills folder, so it is not a project")]
    NotAProject { path: PathBuf },
    #[error("`git {command}` failed in the vault: {stderr}")]
    Git { command: String, stderr: String },
    #[error(transparent)]
    Scan(#[from] ScanError),
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

impl Hint for LocalError {
    fn hint(&self) -> Option<String> {
        match self {
            Self::NoProject { .. } => {
                Some("start it inside a project, or name one with --project DIR".to_string())
            }
            Self::NotAProject { .. } => {
                Some("leave out --project to look around the current folder instead".to_string())
            }
            Self::Scan(e) => e.hint(),
            Self::Git { .. } | Self::Io(_) => None,
        }
    }
}
