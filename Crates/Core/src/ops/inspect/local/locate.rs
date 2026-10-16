//! Finding the project the user means: the folder they stand in, or the nearest one around it.

use std::path::{Path, PathBuf};

use super::LocalError;
use crate::scan::{ScanOptions, scan};

/// Where the project was found, seen from the folder the search started in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Found {
    /// The folder itself has `.agents/skills`.
    Here,
    /// A folder above it, no further up than the top of its git repository.
    Above,
    /// A folder below it, the only one there.
    Below,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Located {
    One {
        root: PathBuf,
        found: Found,
    },
    /// Several projects below the folder; the user has to say which one.
    Several(Vec<PathBuf>),
}

/// Looks in `from` and upwards, stopping after the top of the git repository `from` is in (a skills
/// folder of a project further up belongs to another project). Without a result there, the projects
/// below `from` are searched with the scan rules of the vault.
pub fn locate(from: &Path, options: &ScanOptions) -> Result<Located, LocalError> {
    for dir in from.ancestors() {
        if is_project(dir) {
            let found = if dir == from {
                Found::Here
            } else {
                Found::Above
            };
            return Ok(Located::One {
                root: dir.to_path_buf(),
                found,
            });
        }
        if dir.join(".git").exists() {
            break;
        }
    }
    let mut below: Vec<PathBuf> = scan(from, options)?
        .skills_dirs
        .into_iter()
        .map(|d| d.project)
        .collect();
    below.sort();
    match below.len() {
        0 => Err(LocalError::NoProject {
            from: from.to_path_buf(),
        }),
        1 => Ok(Located::One {
            root: below.swap_remove(0),
            found: Found::Below,
        }),
        _ => Ok(Located::Several(below)),
    }
}

/// A project is a folder with `.agents/skills`; a link in its place is still reported as the project,
/// so the comparison can say what is wrong with it.
fn is_project(dir: &Path) -> bool {
    fs_err::symlink_metadata(dir.join(".agents").join("skills")).is_ok()
}
