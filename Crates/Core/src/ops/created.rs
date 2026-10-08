//! What a command that grows the vault made, in order, so that a failure half way leaves the vault as it
//! was; and staging the result with `git add`.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use super::AuthorError;
use crate::vault::Vault;

/// Everything a command made, in order, so that a failure half way leaves the vault as it was.
#[derive(Default)]
pub(super) struct Created {
    files: Vec<PathBuf>,
    dirs: Vec<PathBuf>,
}

impl Created {
    /// Makes the folders down to and including `target`, remembering the ones that did not exist. The last
    /// folder must be new, so an existing skill is never written into.
    pub(super) fn make_dirs(
        &mut self,
        vault: &Path,
        parts: &[String],
        leaf: &str,
    ) -> Result<PathBuf, AuthorError> {
        let mut path = vault.to_path_buf();
        for part in parts {
            path.push(part);
            match fs_err::create_dir(&path) {
                Ok(()) => self.dirs.push(path.clone()),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => return Err(e.into()),
            }
        }
        path.push(leaf);
        match fs_err::create_dir(&path) {
            Ok(()) => self.dirs.push(path.clone()),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err(AuthorError::Occupied { path });
            }
            Err(e) => return Err(e.into()),
        }
        Ok(path)
    }

    /// Makes the folders between `base` (which exists) and `dir` that are not there yet.
    pub(super) fn make_subdirs(&mut self, base: &Path, dir: &Path) -> Result<(), AuthorError> {
        let Ok(below) = dir.strip_prefix(base) else {
            return Ok(());
        };
        let mut path = base.to_path_buf();
        for part in below {
            path.push(part);
            match fs_err::create_dir(&path) {
                Ok(()) => self.dirs.push(path.clone()),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }

    /// Creates a file that must not exist yet.
    pub(super) fn create_file(&mut self, path: &Path) -> Result<fs_err::File, AuthorError> {
        let file = fs_err::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        self.files.push(path.to_path_buf());
        Ok(file)
    }

    /// Takes back what was made; whatever is not empty or not ours stays.
    pub(super) fn undo(self) {
        for file in self.files.iter().rev() {
            drop(fs_err::remove_file(file));
        }
        for dir in self.dirs.iter().rev() {
            drop(fs_err::remove_dir(dir));
        }
    }
}

/// Stages the skill folder and tells which of `files` git did not take.
pub(super) fn stage(
    vault: &Vault,
    dir: &Path,
    files: &[PathBuf],
) -> Result<Vec<PathBuf>, AuthorError> {
    let rel = dir.strip_prefix(&vault.path).unwrap_or(dir);
    let git = |args: &[&str]| {
        let mut cmd = crate::git::command();
        cmd.env("GIT_LITERAL_PATHSPECS", "1")
            .arg("-C")
            .arg(&vault.path)
            .args(args)
            .arg("--")
            .arg(rel);
        cmd.output()
    };
    let added = git(&["add"])?;
    if !added.status.success() {
        return Err(AuthorError::GitAdd {
            stderr: String::from_utf8_lossy(&added.stderr).trim().to_string(),
        });
    }
    let listed = git(&["ls-files", "-z"])?;
    let known: BTreeSet<PathBuf> = listed
        .stdout
        .split(|b| *b == 0)
        .filter(|r| !r.is_empty())
        .filter_map(|r| {
            let path = PathBuf::from(OsStr::from_bytes(r));
            path.strip_prefix(rel).ok().map(Path::to_path_buf)
        })
        .collect();
    Ok(files
        .iter()
        .filter(|f| !known.contains(*f))
        .cloned()
        .collect())
}
