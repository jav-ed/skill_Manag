//! Checks and setup for a project directory that receives skills: `add` and `init`.

use std::path::{Path, PathBuf};
use std::process::Command;

use super::Workspace;
use crate::Hint;
use crate::plan::Plan;
use crate::scan::Target;

#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("project directory {path} does not exist")]
    NotFound { path: PathBuf },
    #[error("{path} is not a directory")]
    NotADirectory { path: PathBuf },
    #[error("{path} is not empty")]
    NotEmpty { path: PathBuf },
    #[error("{path} must be a real directory, not a link or a file")]
    LayoutNotReal { path: PathBuf },
    #[error("`git init` failed in {path}: {stderr}")]
    GitInit { path: PathBuf, stderr: String },
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

impl Hint for ProjectError {
    fn hint(&self) -> Option<String> {
        match self {
            Self::NotFound { .. } => {
                Some("`skillmirror init <dir>` creates a new project".to_string())
            }
            Self::NotEmpty { .. } => {
                Some("`skillmirror add` installs skills into an existing project".to_string())
            }
            Self::LayoutNotReal { .. } => Some(
                "skills are never written through links; replace it with a real directory"
                    .to_string(),
            ),
            _ => None,
        }
    }
}

/// An existing project: `.agents` and `.agents/skills` may be missing, but must not be links when present.
pub fn check_existing(project: &Path) -> Result<(), ProjectError> {
    match fs_err::metadata(project) {
        Ok(meta) if meta.is_dir() => {}
        Ok(_) => {
            return Err(ProjectError::NotADirectory {
                path: project.to_path_buf(),
            });
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(ProjectError::NotFound {
                path: project.to_path_buf(),
            });
        }
        Err(e) => return Err(e.into()),
    }
    for sub in [
        project.join(".agents"),
        project.join(".agents").join("skills"),
    ] {
        match fs_err::symlink_metadata(&sub) {
            Ok(meta) if meta.is_dir() => {}
            Ok(_) => return Err(ProjectError::LayoutNotReal { path: sub }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => break,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

/// A new project: the directory is absent, or an existing empty directory.
pub fn check_new(dir: &Path) -> Result<(), ProjectError> {
    match fs_err::metadata(dir) {
        Ok(meta) if meta.is_dir() => {
            if fs_err::read_dir(dir)?.next().is_some() {
                return Err(ProjectError::NotEmpty {
                    path: dir.to_path_buf(),
                });
            }
            Ok(())
        }
        Ok(_) => Err(ProjectError::NotADirectory {
            path: dir.to_path_buf(),
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

/// Creates the directory of a new project and, when asked, makes it a git repository.
pub fn create_new(dir: &Path, git: bool) -> Result<(), ProjectError> {
    check_new(dir)?;
    fs_err::create_dir_all(dir)?;
    if git {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["init", "-q"])
            .output()?;
        if !out.status.success() {
            return Err(ProjectError::GitInit {
                path: dir.to_path_buf(),
                stderr: String::from_utf8_lossy(&out.stderr).trim().to_string(),
            });
        }
    }
    Ok(())
}

/// Plans the installation of `skills` into `project`, creating `.agents/skills` when the plan is applied.
/// Skills the project already has are compared like in `sync`, so adding twice is harmless.
pub fn plan_install(
    workspace: &Workspace,
    project: &Path,
    skills: &std::collections::BTreeSet<String>,
) -> Plan {
    let skills_dir = project.join(".agents").join("skills");
    let targets = skills
        .iter()
        .map(|skill| Target {
            project: project.to_path_buf(),
            skill: skill.clone(),
            path: skills_dir.join(skill),
        })
        .collect();
    Plan::for_targets_creating(&workspace.vault, &workspace.files, targets)
}
