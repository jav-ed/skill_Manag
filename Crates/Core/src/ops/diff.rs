//! Diff: what a sync would change in each skill folder, as text. Reads, never writes.
//!
//! The old side is the project's copy and the new side is the vault, so a `-` line is something the sync
//! would take away from the project and a `+` line something it would bring in.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use similar::{ChangeTag, TextDiff};

use super::{Workspace, plan_sync};
use crate::plan::{ChangeKind, PlanError, PlanKind, SkillPlan};
use crate::scan::{ScanReport, Target, same_folder};

/// Files above this size are listed but not shown line by line.
const MAX_SHOWN_BYTES: u64 = 1024 * 1024;
/// Lines of unchanged text around each change.
const CONTEXT: usize = 3;

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

fn files_of(plan: &SkillPlan) -> Vec<FileDiff> {
    let theirs = |rel: &Path| plan.source_dir.join(rel);
    let ours = |rel: &Path| plan.target.path.join(rel);
    let mut files = Vec::new();
    for change in &plan.changes {
        let (old, new) = (ours(&change.rel), theirs(&change.rel));
        files.push(match change.kind {
            ChangeKind::Added => file_diff(&change.rel, DiffKind::Added, None, Some(&new)),
            ChangeKind::Modified => {
                file_diff(&change.rel, DiffKind::Modified, Some(&old), Some(&new))
            }
            ChangeKind::ModeChanged => mode_diff(&change.rel, &old, &new),
        });
    }
    for rel in &plan.removed {
        let old = ours(rel);
        // A folder goes with the files below it; the files are what is worth a line.
        if std::fs::symlink_metadata(&old).is_ok_and(|m| m.is_dir()) {
            continue;
        }
        files.push(file_diff(rel, DiffKind::Removed, Some(&old), None));
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    files
}

fn mode_diff(rel: &Path, old: &Path, new: &Path) -> FileDiff {
    let mode = |path: &Path| std::fs::metadata(path).map_or(0, |m| m.permissions().mode() & 0o777);
    FileDiff {
        path: rel.to_path_buf(),
        kind: DiffKind::ModeChanged,
        added_lines: 0,
        removed_lines: 0,
        modes: Some((mode(old), mode(new))),
        skipped: None,
        text: String::new(),
    }
}

fn file_diff(rel: &Path, kind: DiffKind, old: Option<&Path>, new: Option<&Path>) -> FileDiff {
    let mut file = FileDiff {
        path: rel.to_path_buf(),
        kind,
        added_lines: 0,
        removed_lines: 0,
        modes: None,
        skipped: None,
        text: String::new(),
    };
    let (old_text, new_text) = match (read(old), read(new)) {
        (Ok(a), Ok(b)) => (a, b),
        (Err(skipped), _) | (_, Err(skipped)) => {
            file.skipped = Some(skipped);
            return file;
        }
    };
    let shown = rel.display();
    let diff = TextDiff::from_lines(&old_text, &new_text);
    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Insert => file.added_lines += 1,
            ChangeTag::Delete => file.removed_lines += 1,
            ChangeTag::Equal => {}
        }
    }
    let (from, to) = (
        old.map_or_else(|| "/dev/null".to_string(), |_| format!("a/{shown}")),
        new.map_or_else(|| "/dev/null".to_string(), |_| format!("b/{shown}")),
    );
    file.text = diff
        .unified_diff()
        .context_radius(CONTEXT)
        .header(&from, &to)
        .to_string();
    file
}

/// The text of a file, or why it cannot be shown. A missing side (`None`) is an empty file.
fn read(path: Option<&Path>) -> Result<String, Skipped> {
    let Some(path) = path else {
        return Ok(String::new());
    };
    let size = std::fs::metadata(path)
        .map_err(|e| Skipped::Unreadable(e.to_string()))?
        .len();
    if size > MAX_SHOWN_BYTES {
        return Err(Skipped::TooLarge);
    }
    let bytes = std::fs::read(path).map_err(|e| Skipped::Unreadable(e.to_string()))?;
    match String::from_utf8(bytes) {
        Ok(text) if !text.contains('\0') => Ok(text),
        _ => Err(Skipped::Binary),
    }
}
