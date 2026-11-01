//! Reading the files a plan would change: the old and the new text of each, or why they cannot be shown.
//! The unified text of `diff` and the rows of `sides` both start from this, so they cannot disagree about
//! what a file is.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use similar::{ChangeTag, TextDiff};

use super::{DiffKind, FileDiff, Skipped};
use crate::plan::{ChangeKind, SkillPlan};

/// Files above this size are listed but not shown line by line.
const MAX_SHOWN_BYTES: u64 = 1024 * 1024;
/// Lines of unchanged text around each change.
const CONTEXT: usize = 3;

/// One file of a plan, with both of its texts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawFile {
    pub(super) path: PathBuf,
    pub(super) kind: DiffKind,
    /// Permission bits before and after, for a mode change.
    pub(super) modes: Option<(u32, u32)>,
    pub(super) skipped: Option<Skipped>,
    /// The text in the project and in the vault. A side that has no file, a file that is skipped and a mode
    /// change leave it empty.
    pub(super) old: String,
    pub(super) new: String,
}

/// The unified diff of every file the plan changes.
pub(super) fn files_of(plan: &SkillPlan) -> Vec<FileDiff> {
    raw_files(plan).iter().map(unified).collect()
}

/// Every file the plan changes, with its texts, sorted by path.
pub(super) fn raw_files(plan: &SkillPlan) -> Vec<RawFile> {
    let theirs = |rel: &Path| plan.source_dir.join(rel);
    let ours = |rel: &Path| plan.target.path.join(rel);
    let mut files = Vec::new();
    for change in &plan.changes {
        let (old, new) = (ours(&change.rel), theirs(&change.rel));
        files.push(match change.kind {
            ChangeKind::Added => read_pair(&change.rel, DiffKind::Added, None, Some(&new)),
            ChangeKind::Modified => {
                read_pair(&change.rel, DiffKind::Modified, Some(&old), Some(&new))
            }
            ChangeKind::ModeChanged => mode_change(&change.rel, &old, &new),
        });
    }
    for rel in &plan.removed {
        let old = ours(rel);
        // A folder goes with the files below it; the files are what is worth a line.
        if std::fs::symlink_metadata(&old).is_ok_and(|m| m.is_dir()) {
            continue;
        }
        files.push(read_pair(rel, DiffKind::Removed, Some(&old), None));
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    files
}

fn mode_change(rel: &Path, old: &Path, new: &Path) -> RawFile {
    let mode = |path: &Path| std::fs::metadata(path).map_or(0, |m| m.permissions().mode() & 0o777);
    RawFile {
        path: rel.to_path_buf(),
        kind: DiffKind::ModeChanged,
        modes: Some((mode(old), mode(new))),
        skipped: None,
        old: String::new(),
        new: String::new(),
    }
}

/// A file with both texts read. A missing side (`None`) is an empty file. One side that cannot be shown
/// makes the whole file skipped, because half a comparison says nothing.
fn read_pair(rel: &Path, kind: DiffKind, old: Option<&Path>, new: Option<&Path>) -> RawFile {
    let mut file = RawFile {
        path: rel.to_path_buf(),
        kind,
        modes: None,
        skipped: None,
        old: String::new(),
        new: String::new(),
    };
    match (read(old), read(new)) {
        (Ok(a), Ok(b)) => {
            file.old = a;
            file.new = b;
        }
        (Err(skipped), _) | (_, Err(skipped)) => file.skipped = Some(skipped),
    }
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

/// The counts and the unified text of one file.
fn unified(raw: &RawFile) -> FileDiff {
    let mut file = FileDiff {
        path: raw.path.clone(),
        kind: raw.kind,
        added_lines: 0,
        removed_lines: 0,
        modes: raw.modes,
        skipped: raw.skipped.clone(),
        text: String::new(),
    };
    if raw.skipped.is_some() || raw.kind == DiffKind::ModeChanged {
        return file;
    }
    let shown = raw.path.display();
    let diff = TextDiff::from_lines(&raw.old, &raw.new);
    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Insert => file.added_lines += 1,
            ChangeTag::Delete => file.removed_lines += 1,
            ChangeTag::Equal => {}
        }
    }
    let from = if raw.kind == DiffKind::Added {
        "/dev/null".to_string()
    } else {
        format!("a/{shown}")
    };
    let to = if raw.kind == DiffKind::Removed {
        "/dev/null".to_string()
    } else {
        format!("b/{shown}")
    };
    file.text = diff
        .unified_diff()
        .context_radius(CONTEXT)
        .header(&from, &to)
        .to_string();
    file
}
