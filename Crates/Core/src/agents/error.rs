//! Failures around the AGENTS.md file.

use std::path::PathBuf;

use crate::Hint;
use crate::apply::ApplyError;
use crate::backup::BackupError;

#[derive(Debug, thiserror::Error)]
pub enum AgentsError {
    #[error("the text file {path} cannot be used: {reason}")]
    BadSource { path: PathBuf, reason: String },
    #[error("the AGENTS.md text names the skills {missing}, which {project} would not have")]
    MissingSkills { project: PathBuf, missing: String },
    #[error("{path}: {reason}")]
    Refused { path: PathBuf, reason: String },
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Apply(#[from] ApplyError),
    #[error("cannot keep the old file: {0}")]
    Backup(#[from] BackupError),
}

impl Hint for AgentsError {
    fn hint(&self) -> Option<String> {
        match self {
            Self::BadSource { path, .. } => Some(format!(
                "fix {} or move it away: without it the built-in text is used",
                path.display()
            )),
            Self::MissingSkills { missing, .. } => Some(format!(
                "make them mandatory (`skillmirror mandatory add {}`), name them as skills, or leave AGENTS.md out (`--no-agents-md`)",
                missing.replace(", ", " ")
            )),
            Self::Apply(e) => e.hint(),
            Self::Refused { .. } | Self::Io(_) | Self::Backup(_) => None,
        }
    }
}
