//! Reasons a skill cannot be planned. Each one stops that skill only, never the whole run.

use std::path::PathBuf;

use crate::Hint;

#[derive(Debug, thiserror::Error)]
pub enum PlanError {
    #[error("skill {skill:?} is not in the vault")]
    NotInVault { skill: String },
    #[error("mandatory skill {skill:?} is not in the vault")]
    MandatoryNotInVault { skill: String },
    #[error("skill {skill:?} has no git-tracked files in the vault")]
    NoTrackedFiles { skill: String },
    #[error("skill {skill:?} has other tracked files but its SKILL.md is not tracked")]
    SkillFileNotTracked { skill: String },
    #[error("skill {skill:?} contains a tracked {what}: {path}")]
    UnsupportedEntry {
        skill: String,
        path: PathBuf,
        what: &'static str,
    },
    #[error("skill {skill:?} is itself a tracked {what}")]
    SkillFolderIsLink { skill: String, what: &'static str },
    #[error("tracked file {path} is missing from the vault working tree")]
    MissingSourceFile { path: PathBuf },
    #[error("tracked path {path} is not a regular file in the vault working tree")]
    NotRegularFile { path: PathBuf },
    #[error("{path} is a symlink; skills are never written through links")]
    DestinationIsSymlink { path: PathBuf },
    #[error("{path} exists and is not a directory")]
    DestinationNotADirectory { path: PathBuf },
    #[error("the skills directory {path} does not exist")]
    SkillsDirMissing { path: PathBuf },
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

impl Hint for PlanError {
    fn hint(&self) -> Option<String> {
        match self {
            Self::NoTrackedFiles { skill } => {
                Some(format!("run `git add` for {skill} in the vault and commit"))
            }
            Self::SkillFileNotTracked { skill } => Some(format!(
                "run `git add {skill}/SKILL.md` in the vault and commit; without it the copy would not be a skill"
            )),
            Self::SkillFolderIsLink { .. } => Some(
                "replace the link or submodule with a real folder of regular files".to_string(),
            ),
            Self::UnsupportedEntry { .. } => {
                Some("replace the link with a regular file in the vault".to_string())
            }
            Self::MissingSourceFile { .. } => {
                Some("restore the file or `git rm` it in the vault".to_string())
            }
            Self::MandatoryNotInVault { skill } => Some(format!(
                "remove {skill:?} from `mandatory` in <vault>/config.yaml or add the skill to the vault"
            )),
            _ => None,
        }
    }
}
