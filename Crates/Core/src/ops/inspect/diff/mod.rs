//! Diff: what a sync would change in each skill folder, as text. Reads, never writes.
//!
//! The old side is the project's copy and the new side is the vault, so a `-` line is something the sync
//! would take away from the project and a `+` line something it would bring in. `sides` is the same
//! comparison as rows for a side-by-side view.

mod files;
pub(in crate::ops) mod sides;
#[cfg(test)]
pub(in crate::ops) mod tests;

use std::path::PathBuf;

use files::files_of;

use crate::ops::{Workspace, plan_sync};
use crate::plan::{Plan, PlanError, PlanKind};
use crate::scan::{ScanReport, Target, same_folder};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffKind {
    /// The vault has the file and the project does not.
    Added,
    Modified,
    /// Same content, other permission bits.
    ModeChanged,
    /// The project has the file and the vault does not.
    Removed,
}

/// Why the lines of a file are not shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Skipped {
    Binary,
    TooLarge,
    Unreadable(String),
}

#[derive(Debug, Clone)]
pub struct FileDiff {
    pub path: PathBuf,
    pub kind: DiffKind,
    pub added_lines: usize,
    pub removed_lines: usize,
    /// Permission bits before and after, for a mode change.
    pub modes: Option<(u32, u32)>,
    pub skipped: Option<Skipped>,
    /// A unified diff with its headers; empty when the lines are not shown.
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct SkillDiff {
    pub target: Target,
    /// Sorted by path.
    pub files: Vec<FileDiff>,
}

/// Which skills and projects to compare. An empty filter takes everything.
#[derive(Debug, Default, Clone)]
pub struct DiffFilter {
    pub skill: Option<String>,
    pub project: Option<PathBuf>,
}

#[derive(Debug, Default)]
pub struct DiffReport {
    /// Only skills that differ.
    pub skills: Vec<SkillDiff>,
    /// Folders that could not be compared.
    pub failed: Vec<(Target, PlanError)>,
}

impl DiffReport {
    pub fn files(&self) -> usize {
        self.skills.iter().map(|s| s.files.len()).sum()
    }
}

/// Compares the installed skills that the vault also has with their vault copy.
pub fn diff(workspace: &Workspace, report: &ScanReport, filter: &DiffFilter) -> DiffReport {
    let mut out = DiffReport::default();
    for entry in plan_sync(workspace, report).entries {
        let wanted = filter
            .skill
            .as_ref()
            .is_none_or(|s| *s == entry.target.skill)
            && filter
                .project
                .as_ref()
                .is_none_or(|p| same_folder(p, &entry.target.project));
        if !wanted {
            continue;
        }
        match entry.result {
            Ok(plan) if plan.kind == PlanKind::Unchanged => {}
            Ok(plan) => out.skills.push(SkillDiff {
                files: files_of(&plan),
                target: entry.target,
            }),
            Err(error) => out.failed.push((entry.target, error)),
        }
    }
    out
}

/// The diffs of a plan that is already made, for the skills it would change. The interface shows them
/// before it writes: they are exactly the plan it applies.
pub fn diff_of_plan(plan: &Plan) -> Vec<SkillDiff> {
    plan.entries
        .iter()
        .filter_map(|entry| match &entry.result {
            Ok(skill_plan) if skill_plan.kind != PlanKind::Unchanged => Some(SkillDiff {
                target: entry.target.clone(),
                files: files_of(skill_plan),
            }),
            _ => None,
        })
        .collect()
}
