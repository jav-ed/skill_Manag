//! Bringing a run back: the saved folders return, the skills the run created go.
//!
//! An undo is a run itself. What it replaces or removes is saved again, so `undo` twice is a redo.

use std::path::{Path, PathBuf};

use super::store::{Backups, Change, Entry, LoadedEntry, Run, RunKind};
use super::tree::copy_tree;
use super::{BackupError, Finished, UndoError};
use crate::apply::{Keep, Replaces, discard, place};
use crate::events::{Event, Observer, Status};
use crate::ops::validate_name;
use crate::runid::run_id;
use crate::scan::{Target, first_link_above};

/// Which entries of the run are undone. An empty filter takes all of them.
#[derive(Debug, Default, Clone)]
pub struct Filter {
    pub project: Option<PathBuf>,
    pub skill: Option<String>,
}

impl Filter {
    fn accepts(&self, entry: &LoadedEntry) -> bool {
        self.project
            .as_ref()
            .is_none_or(|p| same_folder(p, &entry.entry.project))
            && self.skill.as_ref().is_none_or(|s| *s == entry.entry.skill)
    }
}

/// The same folder, whether it is named directly or through a link or a relative path.
fn same_folder(left: &Path, right: &Path) -> bool {
    left == right || matches!((left.canonicalize(), right.canonicalize()), (Ok(a), Ok(b)) if a == b)
}

/// What happened to one skill folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Undone {
    /// The saved folder is back in the project.
    Restored,
    /// A skill the run had created was removed.
    Removed,
    /// A created skill that is not there any more.
    AlreadyGone,
}

#[derive(Debug)]
pub struct Restored {
    pub project: PathBuf,
    pub skill: String,
    /// What the undone run had done.
    pub change: Change,
    pub result: Result<Undone, UndoError>,
}

#[derive(Debug)]
pub struct UndoReport {
    /// The run that was undone.
    pub from: String,
    pub entries: Vec<Restored>,
    /// The run that saved what the undo replaced. `None` for a dry run or when nothing was replaced.
    pub saved: Option<Finished>,
}

impl UndoReport {
    pub fn failed(&self) -> usize {
        self.entries.iter().filter(|e| e.result.is_err()).count()
    }
}

/// Undoes the run `id`, or the newest run with entries. With `dry_run` it only checks and reports.
pub fn undo(
    backups: &Backups,
    id: Option<&str>,
    filter: &Filter,
    dry_run: bool,
    observer: Observer<'_>,
) -> Result<UndoReport, BackupError> {
    let run = match id {
        Some(id) => backups.load(id)?,
        None => backups.latest()?,
    };
    if run.entries.is_empty() {
        return Err(BackupError::Empty);
    }
    let new_run = if dry_run {
        None
    } else {
        Some(backups.begin(RunKind::Undo)?)
    };
    // Names of the temporary folders, in the form that scans can tell from a leftover of a dead run.
    let token = run_id()?;
    let mut entries = Vec::new();
    for (position, entry) in run.entries.iter().filter(|e| filter.accepts(e)).enumerate() {
        let result = restore(entry, new_run.as_ref(), position, &token);
        observer(Event::TargetDone {
            target: Target {
                project: entry.entry.project.clone(),
                skill: entry.entry.skill.clone(),
                path: skill_path(&entry.entry.project, &entry.entry.skill),
            },
            status: status_of(entry.entry.change, &result),
        });
        entries.push(Restored {
            project: entry.entry.project.clone(),
            skill: entry.entry.skill.clone(),
            change: entry.entry.change,
            result,
        });
    }
    if !dry_run {
        // An empty run folder is harmless, so failing to remove it is not worth an error.
        if backups.load(&run.id).is_ok_and(|r| r.entries.is_empty()) {
            drop(fs_err::remove_dir_all(&run.dir));
        }
    }
    Ok(UndoReport {
        from: run.id,
        entries,
        saved: new_run.map(|r| r.finish(backups)).filter(|f| f.stored > 0),
    })
}

fn status_of(change: Change, result: &Result<Undone, UndoError>) -> Status {
    match result {
        Ok(Undone::Removed) => Status::Deleted,
        Ok(Undone::AlreadyGone) => Status::Unchanged,
        Ok(Undone::Restored) if change == Change::Deleted => Status::Created,
        Ok(Undone::Restored) => Status::Updated,
        Err(e) => Status::Failed(e.to_string()),
    }
}

fn skill_path(project: &Path, skill: &str) -> PathBuf {
    project.join(".agents").join("skills").join(skill)
}

enum Current {
    Missing,
    Folder,
}

fn restore(
    entry: &LoadedEntry,
    new_run: Option<&Run>,
    index: usize,
    token: &str,
) -> Result<Undone, UndoError> {
    let target = target_of(entry)?;
    let current = current_state(&target)?;
    let dry = new_run.is_none();
    match (entry.entry.change, &current) {
        (Change::Created, Current::Missing) => {
            // Nothing is left to undo, so the note goes too: a stale note would stay the newest run and
            // hide every older one from a plain `undo`.
            if !dry {
                spend(entry)?;
            }
            Ok(Undone::AlreadyGone)
        }
        (Change::Created, Current::Folder) => {
            if let Some(run) = new_run {
                remove_created(entry, &target, run, index, token)?;
            }
            Ok(Undone::Removed)
        }
        (Change::Updated | Change::Deleted, _) => {
            if !entry.has_tree {
                return Err(BackupError::BadEntry {
                    path: entry.dir.clone(),
                    reason: "the saved folder is missing".to_string(),
                }
                .into());
            }
            if let (false, Some(run)) = (dry, new_run) {
                put_back(entry, &target, &current, run, index, token)?;
            }
            Ok(Undone::Restored)
        }
    }
}

/// The project skill folder an entry points at, checked: plain name, real project, no symlinks above.
fn target_of(entry: &LoadedEntry) -> Result<Target, BackupError> {
    let bad = |reason: String| BackupError::BadEntry {
        path: entry.dir.clone(),
        reason,
    };
    let Entry { project, skill, .. } = &entry.entry;
    validate_name(skill).map_err(|e| bad(e.to_string()))?;
    if !fs_err::metadata(project).is_ok_and(|m| m.is_dir()) {
        return Err(BackupError::ProjectGone {
            path: project.clone(),
        });
    }
    let path = skill_path(project, skill);
    if let Some(link) = first_link_above(&path)? {
        return Err(BackupError::LinkedParent { path: link });
    }
    Ok(Target {
        project: project.clone(),
        skill: skill.clone(),
        path,
    })
}

fn current_state(target: &Target) -> Result<Current, BackupError> {
    match fs_err::symlink_metadata(&target.path) {
        Ok(meta) if meta.is_dir() => Ok(Current::Folder),
        Ok(_) => Err(BackupError::NotAFolder {
            path: target.path.clone(),
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Current::Missing),
        Err(e) => Err(e.into()),
    }
}

/// Takes the folder a run created out of the project and keeps it in the undo run.
fn remove_created(
    entry: &LoadedEntry,
    target: &Target,
    run: &Run,
    index: usize,
    token: &str,
) -> Result<(), UndoError> {
    let agents = agents_dir(target)?;
    let trash = agents.join(format!(".trash-{token}-{index}"));
    fs_err::rename(&target.path, &trash).map_err(BackupError::from)?;
    match run.keep(index, target, Change::Deleted, &trash) {
        Ok(None) => {}
        Ok(Some(cause)) => {
            return Err(BackupError::Leftover {
                path: trash,
                reason: cause.to_string(),
            }
            .into());
        }
        Err(cause) => {
            fs_err::rename(&trash, &target.path).map_err(BackupError::from)?;
            return Err(cause.into());
        }
    }
    spend(entry)
}

/// Copies the saved folder into place. The saved copy stays until the new one is in.
fn put_back(
    entry: &LoadedEntry,
    target: &Target,
    current: &Current,
    run: &Run,
    index: usize,
    token: &str,
) -> Result<(), UndoError> {
    let agents = agents_dir(target)?;
    let skills = target
        .path
        .parent()
        .ok_or_else(|| BackupError::NotAFolder {
            path: target.path.clone(),
        })?;
    fs_err::create_dir_all(skills).map_err(BackupError::from)?;
    let stage = agents.join(format!(".stage-{token}-{index}"));
    if let Err(cause) = copy_tree(&entry.tree(), &stage) {
        return Err(discard(&stage, cause.into()).into());
    }
    let replaces = match current {
        Current::Missing => Replaces::Nothing,
        Current::Folder => Replaces::Folder { expected: None },
    };
    let keep = Keep { run, index, target };
    if let Some(left) = place(&stage, &target.path, replaces, Some(keep))? {
        return Err(BackupError::Leftover {
            path: left.path,
            reason: left.reason,
        }
        .into());
    }
    spend(entry)
}

fn agents_dir(target: &Target) -> Result<&Path, BackupError> {
    target
        .path
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| BackupError::NotAFolder {
            path: target.path.clone(),
        })
}

/// The entry has been restored; its saved folder is not needed any more.
fn spend(entry: &LoadedEntry) -> Result<(), UndoError> {
    fs_err::remove_dir_all(&entry.dir).map_err(|e| {
        BackupError::Leftover {
            path: entry.dir.clone(),
            reason: e.to_string(),
        }
        .into()
    })
}
