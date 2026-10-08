//! The result of planning one skill in one project.

use std::path::PathBuf;

use crate::scan::Target;

/// How the destination relates to the vault copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanKind {
    /// The skill folder does not exist yet.
    Create,
    /// The folder exists and differs from the vault.
    Update,
    /// The folder already equals the vault copy.
    Unchanged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Added,
    Modified,
    ModeChanged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    pub rel: PathBuf,
    pub kind: ChangeKind,
}

/// A tracked vault file that will exist in the destination, with the permission bits to give it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub rel: PathBuf,
    /// The nine permission bits of the working-tree file.
    pub mode: u32,
}

/// Everything needed to apply, and everything a front end wants to show, for one target.
#[derive(Debug, Clone)]
pub struct SkillPlan {
    pub target: Target,
    /// The skill folder in the vault.
    pub source_dir: PathBuf,
    pub kind: PlanKind,
    /// Every file the destination must hold, in git order.
    pub sources: Vec<SourceFile>,
    /// Files whose content or mode differs, or that are missing, in git order.
    pub changes: Vec<FileChange>,
    /// Destination entries the vault does not have; they disappear with the swap.
    pub removed: Vec<PathBuf>,
    /// The project has no `.agents/skills` yet; applying creates it. Only set by `add` and `init`.
    pub creates_skills_dir: bool,
    /// What the destination looked like when it was compared. Apply checks it again right after the
    /// swap, so an edit made while the plan waited for a confirmation is never overwritten unseen.
    pub(crate) existing: Option<super::inspect::Snapshot>,
}

impl SkillPlan {
    /// Number of files in the mirrored skill.
    pub fn file_count(&self) -> usize {
        self.sources.len()
    }
}
