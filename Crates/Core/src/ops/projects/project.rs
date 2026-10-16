//! Checks and setup for a project directory that receives skills: `add` and `init`.

use std::path::{Path, PathBuf};

use crate::Hint;
use crate::ops::Workspace;
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
    #[error("cannot tell whether {path} is inside a git repository: {stderr}")]
    GitCheck { path: PathBuf, stderr: String },
    #[error(
        "the project scaffold is made by the script of the doc-start skill, and doc-start is not among the skills this install gives the project"
    )]
    ScaffoldNeedsSkill,
    #[error(
        "the doc-start skill of the vault has no git-tracked {file}, which the project scaffold runs"
    )]
    ScaffoldFileMissing { file: String },
    #[error("the project scaffold failed ({script}): {detail}")]
    ScaffoldRun {
        script: PathBuf,
        project: PathBuf,
        detail: String,
    },
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
            Self::ScaffoldNeedsSkill => Some(
                "install doc-start too (make it mandatory with `skillmirror mandatory add doc-start`), or pass --no-scaffold"
                    .to_string(),
            ),
            Self::ScaffoldFileMissing { .. } => Some(
                "`git add` the file in the vault, or pass --no-scaffold".to_string(),
            ),
            Self::ScaffoldRun {
                script, project, ..
            } => Some(format!(
                "the skills are installed; after fixing the cause run `bash {} {}` again",
                script.display(),
                project.display()
            )),
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

/// What [`create_new`] made, so a command that could install nothing can take it back.
#[derive(Debug)]
pub struct NewProject {
    dir: PathBuf,
    made_dir: bool,
    made_git: bool,
}

impl NewProject {
    /// Removes the repository this call made, the empty `.agents` folders a failed install left, and the
    /// project directory if it did not exist before. Anything that is not empty stays, so a skill that did
    /// get installed or a file somebody added meanwhile is never touched.
    pub fn take_back(&self) -> Result<(), ProjectError> {
        if self.made_git {
            match fs_err::remove_dir_all(self.dir.join(".git")) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.into()),
            }
        }
        remove_if_empty(&self.dir.join(".agents").join("skills"))?;
        remove_if_empty(&self.dir.join(".agents"))?;
        if self.made_dir {
            remove_if_empty(&self.dir)?;
        }
        Ok(())
    }
}

fn remove_if_empty(dir: &Path) -> Result<(), ProjectError> {
    match fs_err::remove_dir(dir) {
        Ok(()) => Ok(()),
        Err(e)
            if matches!(
                e.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::DirectoryNotEmpty
            ) =>
        {
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

/// What `init` does about git for a new project, decided before anything is made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Repo {
    /// Run `git init --initial-branch=main`. There is no remote.
    Make,
    /// The folder is inside this repository already, so no second one is made.
    Inside(PathBuf),
    /// No repository is wanted.
    Skip,
}

/// The branch a new repository starts on.
pub const BRANCH: &str = "main";

/// Whether a new project at `dir` gets a repository: not when it is not wanted, and not when `dir` (or the
/// nearest folder above it that exists) is inside one already.
pub fn repo_for(dir: &Path, wanted: bool) -> Result<Repo, ProjectError> {
    if !wanted {
        return Ok(Repo::Skip);
    }
    let start = dir.ancestors().find(|a| a.is_dir()).unwrap_or(dir);
    let out = crate::git::command()
        .arg("-C")
        .arg(start)
        .args(["rev-parse", "--show-toplevel"])
        .output()?;
    if out.status.success() {
        let top = String::from_utf8_lossy(&out.stdout).trim().to_string();
        return Ok(Repo::Inside(PathBuf::from(top)));
    }
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    if stderr.contains("not a git repository") {
        Ok(Repo::Make)
    } else {
        Err(ProjectError::GitCheck {
            path: start.to_path_buf(),
            stderr,
        })
    }
}

/// Creates the directory of a new project and, when `repo` says so, a git repository on `main`. A failure
/// of `git init` takes back what was made, so the same command can be run again.
pub fn create_new(dir: &Path, repo: &Repo) -> Result<NewProject, ProjectError> {
    check_new(dir)?;
    let made_dir = !dir.exists();
    fs_err::create_dir_all(dir)?;
    let mut project = NewProject {
        dir: dir.to_path_buf(),
        made_dir,
        made_git: false,
    };
    if *repo == Repo::Make {
        let out = crate::git::command()
            .arg("-C")
            .arg(dir)
            .args(["init", "-q", "--initial-branch", BRANCH])
            .output()?;
        // The repository counts as ours from here on, even when git reports a failure half way.
        project.made_git = true;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            drop(project.take_back());
            return Err(ProjectError::GitInit {
                path: dir.to_path_buf(),
                stderr,
            });
        }
    }
    Ok(project)
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
