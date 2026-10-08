//! Failures while reading the vault.

use std::path::PathBuf;

use crate::Hint;

#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    #[error("vault {path} does not exist")]
    Missing { path: PathBuf },
    #[error("vault {path} is not a directory")]
    NotADirectory { path: PathBuf },
    #[error("skill name {name:?} exists twice in the vault: {first} and {second}")]
    DuplicateSkill {
        name: String,
        first: PathBuf,
        second: PathBuf,
    },
    #[error("group {group} has the same name as a skill: {skill}")]
    GroupSkillClash { group: PathBuf, skill: PathBuf },
    #[error("the folder name of {path} is not valid UTF-8")]
    NonUtf8Name { path: PathBuf },
    #[error("{path} is nested more than {max} group levels deep")]
    TooDeep { path: PathBuf, max: usize },
    #[error("vault {path} is not a git repository")]
    NotAGitRepo { path: PathBuf },
    #[error("git is not installed or not on PATH")]
    GitMissing,
    #[error("`git {args}` failed ({status}): {stderr}")]
    GitFailed {
        args: String,
        status: String,
        stderr: String,
    },
    #[error("unexpected output from `git {args}`: {detail}")]
    GitOutput { args: String, detail: String },
    #[error("vault path {path} has unmerged index entries")]
    Unmerged { path: PathBuf },
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

impl Hint for VaultError {
    fn hint(&self) -> Option<String> {
        match self {
            Self::NotAGitRepo { path } => Some(format!(
                "only git-tracked files are copied; run `git init` in {} and commit the skills",
                path.display()
            )),
            Self::GitMissing => Some("install git".to_string()),
            Self::DuplicateSkill { .. } => Some(
                "rename one of the folders; skill names are unique across all groups".to_string(),
            ),
            Self::GroupSkillClash { .. } => Some("rename the group or the skill".to_string()),
            Self::NonUtf8Name { .. } => {
                Some("rename the folder; skill and group names must be valid UTF-8".to_string())
            }
            Self::TooDeep { max, .. } => Some(format!(
                "move the skill up, at most {max} group folders may sit above a skill"
            )),
            Self::Unmerged { .. } => Some("resolve the merge in the vault first".to_string()),
            _ => None,
        }
    }
}
