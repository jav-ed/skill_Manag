//! The changes of a plan as rows for a side-by-side view: the old line next to the new one, with the words
//! that changed marked. It starts from the same reading as `diff`, so the two cannot disagree.
//!
//! Identical changes are put together: one skill that is behind in twenty projects in the same way is one
//! group with twenty targets, not twenty pages to read. Reads, never writes.

mod group;
mod pairs;
mod rows;
#[cfg(test)]
mod tests;
mod words;

use std::ops::Range;
use std::path::PathBuf;

use super::files::{RawFile, raw_files};
use super::{DiffKind, Skipped};
use crate::agents::{Action, AgentsPlan};
use crate::describe;
use crate::plan::{Plan, PlanKind};
use crate::scan::Target;

/// The name a group of AGENTS.md changes goes under, where a skill group has the skill's name.
pub const AGENTS_FILE: &str = "AGENTS.md";

/// One line of one side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SideLine {
    /// The line number in its file, counted from 1.
    pub no: usize,
    /// The line as it is shown: no line ending, a tab as four spaces, a control character as its picture.
    pub text: String,
    /// The byte ranges of `text` that differ from the line it is paired with, sorted and apart. Empty when
    /// the whole line is new, gone, or too different for marks to help.
    pub marks: Vec<Range<usize>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SideRow {
    /// A line that is the same on both sides.
    Same {
        old_no: usize,
        new_no: usize,
        text: String,
    },
    /// A line that changed, with the line it was paired with, or a line with no partner (`None` on the
    /// other side). Never both `None`.
    Change {
        old: Option<SideLine>,
        new: Option<SideLine>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SideBody {
    /// Every line of the file, the ones that stayed too: folding them is the view's business.
    Rows(Vec<SideRow>),
    /// Binary, too large or unreadable: there are no lines.
    Skipped(Skipped),
    /// The text is the same; only the permissions changed.
    ModeOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SideFile {
    pub path: PathBuf,
    pub kind: DiffKind,
    /// Permission bits before and after, for a mode change.
    pub modes: Option<(u32, u32)>,
    pub added: usize,
    pub removed: usize,
    pub body: SideBody,
}

/// The targets that get exactly the same change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeGroup {
    pub skill: String,
    /// In plan order; never empty.
    pub targets: Vec<Target>,
    /// Sorted by path.
    pub files: Vec<SideFile>,
}

#[derive(Debug, Default)]
pub struct SideReport {
    /// By skill; of one skill, the group with most targets first.
    pub groups: Vec<ChangeGroup>,
    /// Targets that cannot be compared or applied, with the reason.
    pub failed: Vec<(Target, String)>,
}

impl SideReport {
    pub fn is_empty(&self) -> bool {
        self.groups.is_empty() && self.failed.is_empty()
    }

    /// Puts what `other` found after what this one found.
    pub fn extend(&mut self, other: Self) {
        self.groups.extend(other.groups);
        self.failed.extend(other.failed);
    }
}

/// The changes a plan would make, grouped. The old side is the project's copy and the new side is what the
/// plan brings in, as in `diff`. Skills that are up to date are left out; targets that failed are listed.
pub fn sides_of_plan(plan: &Plan) -> SideReport {
    let mut entries = Vec::new();
    let mut failed = Vec::new();
    for entry in &plan.entries {
        match &entry.result {
            Ok(skill) if skill.kind == PlanKind::Unchanged => {}
            Ok(skill) => entries.push((entry.target.clone(), raw_files(skill))),
            Err(error) => failed.push((entry.target.clone(), describe(error))),
        }
    }
    SideReport {
        groups: group::groups(entries),
        failed,
    }
}

/// The AGENTS.md files a plan would write: the file now against the file after. Where nothing is written
/// the entry is left out; an entry that failed is listed.
pub fn sides_of_agents(plan: &AgentsPlan) -> SideReport {
    let mut entries = Vec::new();
    let mut failed = Vec::new();
    for entry in &plan.entries {
        let target = Target {
            project: entry.project.clone(),
            skill: AGENTS_FILE.to_string(),
            path: entry.path.clone(),
        };
        if let Action::Failed(reason) = &entry.action {
            failed.push((target, reason.clone()));
            continue;
        }
        let Some(after) = entry.after.as_deref().filter(|_| entry.writes()) else {
            continue;
        };
        let (kind, old) = match &entry.before {
            Some(now) => (DiffKind::Modified, now.clone()),
            None => (DiffKind::Added, String::new()),
        };
        let file = RawFile {
            path: PathBuf::from(AGENTS_FILE),
            kind,
            modes: None,
            skipped: None,
            old,
            new: after.to_string(),
        };
        entries.push((target, vec![file]));
    }
    SideReport {
        groups: group::groups(entries),
        failed,
    }
}
