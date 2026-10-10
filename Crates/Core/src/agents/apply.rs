//! Writing the AGENTS.md files of a plan: each is built beside its target and swapped in with one
//! `renameat2`, the way skill folders are, so a crash never leaves a half written file and an edit made
//! while the plan waited for a confirmation is never overwritten unseen.

use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use super::{Action, AgentsEntry, AgentsError, AgentsPlan, FILE_NAME};
use crate::apply::{ApplyError, Leftover, exchange, place_new};
use crate::backup::{Change, Run};
use crate::events::{Event, Observer, Status};
use crate::runid::run_id;
use crate::scan::Target;

/// What was done to one file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Written {
    Created,
    /// The block was put in front of a file that had none.
    Inserted,
    Updated,
    /// Nothing to do: the plan did not write this one.
    Unchanged,
}

#[derive(Debug)]
pub struct Applied {
    pub project: PathBuf,
    pub path: PathBuf,
    pub result: Result<Written, AgentsError>,
    /// The old file could not be removed or moved afterwards; the new one is in place.
    pub leftover: Option<Leftover>,
}

#[derive(Debug, Default)]
pub struct AgentsReport {
    pub applied: Vec<Applied>,
}

impl AgentsReport {
    /// Files created, inserted into or updated.
    pub fn wrote(&self) -> usize {
        self.applied
            .iter()
            .filter(|a| {
                matches!(
                    a.result,
                    Ok(Written::Created | Written::Inserted | Written::Updated)
                )
            })
            .count()
    }

    pub fn failed(&self) -> usize {
        self.applied.iter().filter(|a| a.result.is_err()).count()
    }
}

/// Writes the plan. With a `run`, what a file replaces is kept for `undo` and a created file is noted.
/// `first_index` is the first slot of the run this call may use; a command that also wrote skills in the
/// same run passes the number of slots those used.
pub fn apply_agents(
    plan: &AgentsPlan,
    run: Option<&Run>,
    first_index: usize,
    observer: Observer<'_>,
) -> AgentsReport {
    let token = run_id().unwrap_or_else(|_| "0".to_string());
    let mut applied = Vec::with_capacity(plan.entries.len());
    for (offset, entry) in plan.entries.iter().enumerate() {
        let index = first_index + offset;
        let (result, leftover) = match &entry.action {
            Action::Unchanged | Action::Skipped(_) => (Ok(Written::Unchanged), None),
            Action::Failed(reason) => (
                Err(AgentsError::Refused {
                    path: entry.path.clone(),
                    reason: reason.clone(),
                }),
                None,
            ),
            Action::Create => finish(create(entry, run, index, &token)),
            Action::Insert | Action::Update => finish(replace(entry, run, index, &token)),
        };
        observer(Event::TargetDone {
            target: Target {
                project: entry.project.clone(),
                skill: FILE_NAME.to_string(),
                path: entry.path.clone(),
            },
            status: match &result {
                Ok(Written::Created) => Status::Created,
                Ok(Written::Inserted | Written::Updated) => Status::Updated,
                Ok(Written::Unchanged) => Status::Unchanged,
                Err(e) => Status::Failed(e.to_string()),
            },
        });
        applied.push(Applied {
            project: entry.project.clone(),
            path: entry.path.clone(),
            result,
            leftover,
        });
    }
    AgentsReport { applied }
}

type Done = (Written, Option<Leftover>);

fn finish(done: Result<Done, AgentsError>) -> (Result<Written, AgentsError>, Option<Leftover>) {
    match done {
        Ok((written, leftover)) => (Ok(written), leftover),
        Err(e) => (Err(e), None),
    }
}

fn stage_path(entry: &AgentsEntry, token: &str, index: usize) -> PathBuf {
    entry
        .project
        .join(format!(".{FILE_NAME}.stage-{token}-{index}"))
}

fn planned(content: Option<&String>, path: &Path) -> Result<String, AgentsError> {
    content.cloned().ok_or_else(|| AgentsError::Refused {
        path: path.to_path_buf(),
        reason: "the plan holds no content for this file".to_string(),
    })
}

fn create(
    entry: &AgentsEntry,
    run: Option<&Run>,
    index: usize,
    token: &str,
) -> Result<Done, AgentsError> {
    let content = planned(entry.after.as_ref(), &entry.path)?;
    let stage = stage_path(entry, token, index);
    write_stage(&stage, &content, 0o644)?;
    if let Some(run) = run
        && let Err(cause) = run.record_created_file(index, &entry.project)
    {
        return Err(discard(&stage, cause.into()));
    }
    if let Err(cause) = place_new(&stage, &entry.path) {
        // A note about a file that was never created only makes `undo` say "already gone".
        if let Some(run) = run {
            drop(run.forget(index));
        }
        return Err(discard(&stage, cause.into()));
    }
    Ok((Written::Created, None))
}

/// Swaps the new file in, then checks that the file it replaced is the one that was planned against. If
/// someone changed it meanwhile the swap is undone, so their edit survives.
fn replace(
    entry: &AgentsEntry,
    run: Option<&Run>,
    index: usize,
    token: &str,
) -> Result<Done, AgentsError> {
    let content = planned(entry.after.as_ref(), &entry.path)?;
    let expected = planned(entry.before.as_ref(), &entry.path)?;
    let meta = fs_err::symlink_metadata(&entry.path)?;
    if !meta.is_file() {
        return Err(AgentsError::Refused {
            path: entry.path.clone(),
            reason: "is not a regular file any more; nothing was written".to_string(),
        });
    }
    let stage = stage_path(entry, token, index);
    write_stage(&stage, &content, meta.permissions().mode() & 0o7777)?;
    exchange(&stage, &entry.path).map_err(|cause| discard(&stage, cause.into()))?;
    // After the exchange the stage path holds the previous file.
    if fs_err::read_to_string(&stage).ok().as_deref() != Some(expected.as_str()) {
        exchange(&stage, &entry.path)?;
        return Err(discard(
            &stage,
            ApplyError::DestinationChanged {
                dest: entry.path.clone(),
            }
            .into(),
        ));
    }
    let written = if entry.action == Action::Insert {
        Written::Inserted
    } else {
        Written::Updated
    };
    let Some(run) = run else {
        return Ok((
            written,
            fs_err::remove_file(&stage)
                .err()
                .map(|e| leftover(&stage, &e)),
        ));
    };
    match run.keep_file(index, &entry.project, Change::Updated, &stage) {
        Ok(left) => Ok((written, left.map(|e| leftover(&stage, &e)))),
        Err(cause) => {
            // No backup, no update: the old file goes back.
            exchange(&stage, &entry.path)?;
            Err(discard(&stage, cause.into()))
        }
    }
}

fn leftover(stage: &Path, cause: &std::io::Error) -> Leftover {
    Leftover {
        path: stage.to_path_buf(),
        reason: cause.to_string(),
    }
}

/// Creates the stage file with these permission bits; it must not exist.
fn write_stage(stage: &Path, content: &str, mode: u32) -> Result<(), AgentsError> {
    let with_path = |e: std::io::Error| {
        std::io::Error::new(e.kind(), format!("cannot write {}: {e}", stage.display()))
    };
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(mode)
        .open(stage)
        .map_err(with_path)?;
    let written = file
        .write_all(content.as_bytes())
        .and_then(|()| file.set_permissions(std::fs::Permissions::from_mode(mode)));
    if let Err(e) = written {
        drop(fs_err::remove_file(stage));
        return Err(with_path(e).into());
    }
    Ok(())
}

/// Removes a half-made stage file, and reports the original cause.
fn discard(stage: &Path, cause: AgentsError) -> AgentsError {
    drop(fs_err::remove_file(stage));
    cause
}
