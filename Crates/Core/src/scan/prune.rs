//! Which directories the scan never enters.

use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

use super::ScanError;

/// Directory names that are never worth descending into: version control, dependency trees, build output, caches.
pub const NOISE_DIRS: [&str; 16] = [
    ".git",
    "node_modules",
    "vendor",
    "dist",
    "build",
    "out",
    "target",
    ".next",
    ".nuxt",
    ".venv",
    "__pycache__",
    ".tox",
    ".pytest_cache",
    ".cache",
    ".turbo",
    ".parcel-cache",
];

/// Workspace policy from the vault config. Technical noise stays in [`NOISE_DIRS`].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ScanOptions {
    /// Directory names skipped wherever they occur.
    pub exclude_dirs: Vec<String>,
    /// Absolute paths, or paths relative to the scan root, skipped together with everything below them.
    pub exclude_paths: Vec<PathBuf>,
}

/// [`ScanOptions`] resolved against one scan root, ready to test directories.
#[derive(Debug)]
pub(crate) struct ScanFilter {
    exclude_dirs: Vec<String>,
    exclude_paths: Vec<PathBuf>,
}

impl ScanFilter {
    /// `root` must be canonical.
    pub(crate) fn new(root: &Path, options: &ScanOptions) -> Result<Self, ScanError> {
        let mut exclude_paths = Vec::new();
        for entry in options
            .exclude_paths
            .iter()
            .filter(|p| !p.as_os_str().is_empty())
        {
            let joined = if entry.is_absolute() {
                entry.clone()
            } else {
                root.join(entry)
            };
            // Climbing above the filesystem root is caught on the text; the real path is then
            // resolved through symlinks, because the walker yields paths below the canonical root.
            if lexical_clean(&joined).is_none() {
                return Err(ScanError::ExcludePathEscapes {
                    entry: entry.clone(),
                });
            }
            match fs_err::canonicalize(&joined) {
                Ok(resolved) => exclude_paths.push(resolved),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    return Err(ScanError::ExcludePathMissing {
                        entry: entry.clone(),
                    });
                }
                Err(e) => return Err(e.into()),
            }
        }
        Ok(Self {
            exclude_dirs: options.exclude_dirs.clone(),
            exclude_paths,
        })
    }

    /// True when the directory `path` (named `name`) must not be entered.
    pub(crate) fn skips(&self, path: &Path, name: &OsStr) -> bool {
        name.to_str()
            .is_some_and(|n| NOISE_DIRS.contains(&n) || self.exclude_dirs.iter().any(|d| d == n))
            || self
                .exclude_paths
                .iter()
                .any(|excluded| path.starts_with(excluded))
    }
}

/// Resolves `.` and `..` lexically. `None` when `..` would climb above the root of the path.
fn lexical_clean(path: &Path) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                if !out.pop() {
                    return None;
                }
            }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    Some(out)
}
