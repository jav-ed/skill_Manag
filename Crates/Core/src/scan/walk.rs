//! Parallel walk of the scan root that finds every `.agents/skills` directory.

use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};

use ignore::{WalkBuilder, WalkState};

use super::prune::ScanFilter;
use super::{ScanError, ScanIssue, ScanOptions};

const AGENTS_DIR: &str = ".agents";
const SKILLS_DIR: &str = "skills";

/// One `.agents/skills` directory and the project that owns it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillsDir {
    /// The parent of `.agents`.
    pub project: PathBuf,
    /// The `.agents/skills` directory itself.
    pub dir: PathBuf,
}

/// Result of one scan: the skills directories in walk order, plus everything that could not be read.
#[derive(Debug, Default)]
pub struct ScanReport {
    pub skills_dirs: Vec<SkillsDir>,
    pub issues: Vec<ScanIssue>,
}

enum Found {
    Dir(SkillsDir),
    Issue(ScanIssue),
}

/// Walks `root` in parallel. Symlinks are never followed, noise and excluded directories are never entered,
/// and descent stops at the first `.agents/skills`. The result is sorted so it does not depend on thread timing.
pub fn scan(root: &Path, options: &ScanOptions) -> Result<ScanReport, ScanError> {
    let meta = match fs_err::metadata(root) {
        Ok(meta) => meta,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(ScanError::RootMissing {
                path: root.to_path_buf(),
            });
        }
        Err(e) => return Err(e.into()),
    };
    if !meta.is_dir() {
        return Err(ScanError::RootNotADirectory {
            path: root.to_path_buf(),
        });
    }

    // Walk the real directory, so `..` and symlinks in the given root cannot disagree with `exclude_paths`.
    let root = fs_err::canonicalize(root)?;
    let root = root.as_path();
    let filter = Arc::new(ScanFilter::new(root, options)?);
    let (tx, rx) = mpsc::channel::<Found>();
    let prune = Arc::clone(&filter);

    WalkBuilder::new(root)
        .standard_filters(false)
        .hidden(false)
        .follow_links(false)
        .filter_entry(move |entry| {
            let is_dir = entry.file_type().is_some_and(|t| t.is_dir());
            entry.depth() == 0 || !is_dir || !prune.skips(entry.path(), entry.file_name())
        })
        .build_parallel()
        .run(|| {
            let tx = tx.clone();
            Box::new(move |result| visit(result, &tx, root))
        });
    drop(tx);

    let mut report = ScanReport::default();
    for found in rx {
        match found {
            Found::Dir(dir) => report.skills_dirs.push(dir),
            Found::Issue(issue) => report.issues.push(issue),
        }
    }
    report.skills_dirs.sort_by(|a, b| a.dir.cmp(&b.dir));
    report.issues.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(report)
}

fn visit(
    result: Result<ignore::DirEntry, ignore::Error>,
    tx: &mpsc::Sender<Found>,
    root: &Path,
) -> WalkState {
    // A failed send means the receiver is gone, so there is nobody left to scan for.
    let send = |found: Found| {
        if tx.send(found).is_ok() {
            WalkState::Continue
        } else {
            WalkState::Quit
        }
    };
    let entry = match result {
        Ok(entry) => entry,
        Err(error) => {
            // An error without a path is about the walk itself, so the scan root is the place to look.
            let path = error_path(&error).unwrap_or_else(|| root.to_path_buf());
            return send(Found::Issue(ScanIssue {
                path,
                message: error.to_string(),
            }));
        }
    };
    if !entry.file_type().is_some_and(|t| t.is_dir()) || entry.file_name() != SKILLS_DIR {
        return WalkState::Continue;
    }
    let Some(agents) = entry
        .path()
        .parent()
        .filter(|p| p.file_name().is_some_and(|n| n == AGENTS_DIR))
    else {
        return WalkState::Continue;
    };
    let Some(project) = agents.parent() else {
        return WalkState::Continue;
    };
    for issue in leftovers(agents) {
        if !matches!(send(Found::Issue(issue)), WalkState::Continue) {
            return WalkState::Quit;
        }
    }
    let found = Found::Dir(SkillsDir {
        project: project.to_path_buf(),
        dir: entry.into_path(),
    });
    match send(found) {
        WalkState::Continue => WalkState::Skip,
        other => other,
    }
}

fn error_path(error: &ignore::Error) -> Option<PathBuf> {
    match error {
        ignore::Error::WithPath { path, .. } => Some(path.clone()),
        ignore::Error::WithDepth { err, .. } | ignore::Error::WithLineNumber { err, .. } => {
            error_path(err)
        }
        ignore::Error::Partial(errors) => errors.iter().find_map(error_path),
        _ => None,
    }
}

/// Folders an interrupted run leaves next to `skills`: a half-built copy or a half-deleted skill.
fn leftovers(agents: &Path) -> Vec<ScanIssue> {
    let entries = match fs_err::read_dir(agents) {
        Ok(entries) => entries,
        Err(e) => {
            return vec![ScanIssue {
                path: agents.to_path_buf(),
                message: e.to_string(),
            }];
        }
    };
    entries
        .filter_map(Result::ok)
        .filter(|e| {
            let name = e.file_name();
            name.as_bytes().starts_with(b".stage-") || name.as_bytes().starts_with(b".trash-")
        })
        .map(|e| ScanIssue {
            path: e.path(),
            message: "left over from an interrupted run; delete it by hand".to_string(),
        })
        .collect()
}
