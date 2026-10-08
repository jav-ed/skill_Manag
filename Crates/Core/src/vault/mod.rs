//! The vault: master copies of skills, grouped in folders, tracked by git.

mod discover;
mod error;
mod tracked;

pub use discover::{Ignored, IgnoredReason, MAX_GROUP_DEPTH, Skill, Vault, discover};
pub use error::VaultError;
pub use tracked::{
    FileProblem, ProblemKind, SELF_PATH, SkillFiles, TrackedFile, VaultFiles, read_files,
};

#[cfg(test)]
mod safety_tests;
#[cfg(test)]
mod tests;
