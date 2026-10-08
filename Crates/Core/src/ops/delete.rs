//! Delete: remove skill folders from projects.

use std::path::{Path, PathBuf};

use crate::backup::{BackupError, Change, Run};
use crate::events::{Event, Observer, Status};
use crate::runid::run_id;
use crate::scan::{ScanReport, Target, TargetSet, all_targets, first_link_above};

#[derive(Debug, thiserror::Error)]
pub enum DeleteError {
    #[error("{name:?} is not a valid skill name")]
    InvalidName { name: String },
    #[error("{path} is not an installed skill folder")]
    NotInstalled { path: PathBuf },
    #[error("{path} is a symlink; skill folders behind a link are never removed")]
    LinkedParent { path: PathBuf },
    #[error(
        "{skill} was removed from the skills directory, but its remains at {trash} could not be deleted: {reason}"
    )]
    RemainsKept {
        skill: String,
        trash: PathBuf,
        reason: String,
    },
    #[error("{0}")]
    Backup(#[from] BackupError),
    #[error(
        "{skill} was not removed: its backup failed ({cause}), and the folder could not be put back; it is at {trash} ({reason})"
    )]
    Stranded {
        skill: String,
        trash: PathBuf,
        cause: String,
        reason: String,
    },
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

/// A skill name is one plain path component: no separators, no dots at the start, never empty.
/// The Go tool joined unchecked names, so `..` removed the whole `.agents` directory.
pub fn validate_name(name: &str) -> Result<(), DeleteError> {
    let bad = name.is_empty() || name.starts_with('.') || name.contains(['/', '\\', '\0']);
    if bad {
        Err(DeleteError::InvalidName {
            name: name.to_string(),
        })
    } else {
        Ok(())
    }
}

/// Every installed folder with this name, in walk order.
pub fn targets_named(report: &ScanReport, name: &str) -> Result<TargetSet, DeleteError> {
    validate_name(name)?;
    let mut set = all_targets(&report.skills_dirs);
    set.targets.retain(|t| t.skill == name);
    Ok(set)
}

/// The folder `<project>/.agents/skills/<name>`, which must exist as a directory or a link to one.
pub fn target_in_project(project: &Path, name: &str) -> Result<Target, DeleteError> {
    validate_name(name)?;
    let path = project.join(".agents").join("skills").join(name);
    if let Some(link) = crate::scan::first_link_above(&path)? {
        return Err(DeleteError::LinkedParent { path: link });
    }
    match fs_err::symlink_metadata(&path) {
        // A symlinked skill folder is allowed here: deleting it removes the link and nothing behind it.
        Ok(meta) if meta.is_dir() || meta.file_type().is_symlink() => Ok(Target {
            project: project.to_path_buf(),
            skill: name.to_string(),
            path,
        }),
        Ok(_) => Err(DeleteError::NotInstalled { path }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err(DeleteError::NotInstalled { path })
        }
        Err(e) => Err(e.into()),
    }
}

#[derive(Debug)]
pub struct Deleted {
    pub target: Target,
    pub result: Result<(), DeleteError>,
}

#[derive(Debug, Default)]
pub struct DeleteReport {
    pub deleted: Vec<Deleted>,
}

impl DeleteReport {
    pub fn failed(&self) -> usize {
        self.deleted.iter().filter(|d| d.result.is_err()).count()
    }
}

/// Removes each target folder, or only reports it when `dry_run` is set. One failure never stops the others.
///
/// With a `backup` run the folder is kept in the backup store instead of being deleted. If it cannot be
/// stored it is put back and that target fails. A symlinked skill folder is not stored: only the link goes.
pub fn delete(
    targets: Vec<Target>,
    dry_run: bool,
    backup: Option<&Run>,
    observer: Observer<'_>,
) -> DeleteReport {
    let run = run_id();
    let deleted = targets
        .into_iter()
        .enumerate()
        .map(|(index, target)| {
            let result = if dry_run {
                Ok(())
            } else {
                remove(&target, &run, index, backup)
            };
            let status = match &result {
                Ok(()) => Status::Deleted,
                Err(e) => Status::Failed(e.to_string()),
            };
            observer(Event::TargetDone {
                target: target.clone(),
                status,
            });
            Deleted { target, result }
        })
        .collect();
    DeleteReport { deleted }
}

fn remove(
    target: &Target,
    run: &std::io::Result<String>,
    index: usize,
    backup: Option<&Run>,
) -> Result<(), DeleteError> {
    if let Some(link) = first_link_above(&target.path)? {
        return Err(DeleteError::LinkedParent { path: link });
    }
    let not_installed = || DeleteError::NotInstalled {
        path: target.path.clone(),
    };
    match fs_err::symlink_metadata(&target.path) {
        Ok(meta) if meta.is_dir() => {
            let run = run
                .as_ref()
                .map_err(|e| std::io::Error::new(e.kind(), e.to_string()))?;
            let agents = target
                .path
                .parent()
                .and_then(Path::parent)
                .ok_or_else(not_installed)?;
            // One rename takes the skill out of `skills/`, so it is never left half deleted there.
            let trash = agents.join(format!(".trash-{run}-{index}"));
            fs_err::rename(&target.path, &trash)?;
            match backup {
                Some(run) => keep(run, target, index, trash),
                None => discard(target, trash),
            }
        }
        // A symlinked skill folder loses the link only; `remove_dir_all` never follows it.
        Ok(meta) if meta.file_type().is_symlink() => Ok(fs_err::remove_file(&target.path)?),
        Ok(_) => Err(not_installed()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(not_installed()),
        Err(e) => Err(e.into()),
    }
}

fn discard(target: &Target, trash: PathBuf) -> Result<(), DeleteError> {
    fs_err::remove_dir_all(&trash).map_err(|e| DeleteError::RemainsKept {
        skill: target.skill.clone(),
        trash,
        reason: e.to_string(),
    })
}

/// Moves the folder out of the trash into the backup store, or back into the project if that fails.
fn keep(run: &Run, target: &Target, index: usize, trash: PathBuf) -> Result<(), DeleteError> {
    match run.keep(index, target, Change::Deleted, &trash) {
        Ok(None) => Ok(()),
        Ok(Some(cause)) => Err(DeleteError::RemainsKept {
            skill: target.skill.clone(),
            trash,
            reason: cause.to_string(),
        }),
        Err(cause) => match fs_err::rename(&trash, &target.path) {
            Ok(()) => Err(cause.into()),
            Err(reason) => Err(DeleteError::Stranded {
                skill: target.skill.clone(),
                trash,
                cause: cause.to_string(),
                reason: reason.to_string(),
            }),
        },
    }
}
