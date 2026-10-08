//! Finding projects: every `.agents/skills` directory below the scan root, and the skill folders in it.

mod error;
mod links;
mod prune;
mod targets;
mod walk;

pub use error::{ScanError, ScanIssue};
pub use links::{first_link_above, same_folder};
pub use prune::{NOISE_DIRS, ScanOptions};
pub use targets::{Target, TargetSet, all_targets, push_targets, sync_targets};
pub use walk::{ScanReport, SkillsDir, scan};

#[cfg(test)]
mod safety_tests;
#[cfg(test)]
mod tests;
