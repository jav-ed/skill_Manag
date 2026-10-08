//! The history page's data: the runs in the backup store, and what undoing one of them does.

use std::collections::BTreeSet;

use skillmirror_core::Hint;
use skillmirror_core::backup::{
    Backups, Change, Filter, RunKind, UndoReport, Undone, describe_run, undo,
};
use skillmirror_core::config::Dirs;
use skillmirror_core::events::Observer;

use crate::session::describe;

/// One run in the backup store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RunRow {
    pub(crate) id: String,
    /// The id as a date, `2026-10-08 12:34:56 UTC`.
    pub(crate) date: String,
    pub(crate) command: String,
    pub(crate) skills: usize,
    pub(crate) projects: usize,
    /// Set when the notes of the run could not be read; such a run cannot be undone here.
    pub(crate) error: Option<String>,
}

/// What undoing a folder does, or did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Step {
    /// The saved folder comes back.
    Restore,
    /// A skill the run created goes.
    Remove,
    /// A created skill that is not there any more.
    Gone,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UndoLine {
    pub(crate) project: String,
    pub(crate) skill: String,
    /// What the undone run had done to the folder: `created`, `updated` or `deleted`.
    pub(crate) was: &'static str,
    pub(crate) step: Step,
}

/// An undo as planned, or as done once `applied` is set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UndoView {
    pub(crate) run: String,
    pub(crate) date: String,
    pub(crate) command: String,
    pub(crate) lines: Vec<UndoLine>,
    pub(crate) applied: bool,
    /// The run that saved what the undo replaced; undoing that one is a redo.
    pub(crate) saved_as: Option<String>,
    /// Things that went well but need a hand, such as old backups that could not be removed.
    pub(crate) warnings: Vec<String>,
}

impl UndoView {
    /// Folders the undo can put right.
    pub(crate) fn actionable(&self) -> usize {
        self.lines
            .iter()
            .filter(|l| !matches!(l.step, Step::Failed(_)))
            .count()
    }

    pub(crate) fn failed(&self) -> usize {
        self.lines.len() - self.actionable()
    }

    pub(crate) fn projects(&self) -> usize {
        self.lines
            .iter()
            .map(|l| &l.project)
            .collect::<BTreeSet<_>>()
            .len()
    }
}

/// Every run that has anything left to undo, the newest first. A run whose notes cannot be read is
/// listed with the reason.
pub(crate) fn list_runs(dirs: &Dirs) -> Result<Vec<RunRow>, String> {
    let backups = Backups::in_dirs(dirs);
    let ids = backups
        .run_ids()
        .map_err(|e| describe(&skillmirror_core::Error::from(e)))?;
    let mut rows = Vec::new();
    for id in ids {
        let date = describe_run(&id);
        match backups.load(&id) {
            // A run whose entries were all spent is only an empty folder.
            Ok(run) if run.entries.is_empty() => {}
            Ok(run) => rows.push(RunRow {
                command: run.kind().map_or("run", RunKind::name).to_string(),
                skills: run.entries.len(),
                projects: run
                    .entries
                    .iter()
                    .map(|e| &e.entry.project)
                    .collect::<BTreeSet<_>>()
                    .len(),
                error: None,
                date,
                id,
            }),
            Err(error) => rows.push(RunRow {
                command: "run".to_string(),
                skills: 0,
                projects: 0,
                error: Some(error.to_string()),
                date,
                id,
            }),
        }
    }
    Ok(rows)
}

/// Checks every folder of the run against the disk and changes nothing.
pub(crate) fn plan_undo(dirs: &Dirs, id: &str) -> Result<UndoView, String> {
    run(dirs, id, true, &skillmirror_core::events::ignore_events)
}

/// Brings the run back. What it replaces is saved again as a run of its own.
pub(crate) fn do_undo(dirs: &Dirs, id: &str, observer: Observer<'_>) -> Result<UndoView, String> {
    run(dirs, id, false, observer)
}

fn run(dirs: &Dirs, id: &str, dry_run: bool, observer: Observer<'_>) -> Result<UndoView, String> {
    let backups = Backups::in_dirs(dirs);
    let command = backups
        .load(id)
        .map_err(|e| describe(&skillmirror_core::Error::from(e)))?
        .kind()
        .map_or("run", RunKind::name)
        .to_string();
    let report = undo(&backups, Some(id), &Filter::default(), dry_run, observer)
        .map_err(|e| describe(&skillmirror_core::Error::from(e)))?;
    Ok(view_of(&report, command, !dry_run))
}

fn view_of(report: &UndoReport, command: String, applied: bool) -> UndoView {
    let lines = report
        .entries
        .iter()
        .map(|e| UndoLine {
            project: e.project.display().to_string(),
            skill: e.skill.clone(),
            was: match e.change {
                Change::Created => "created",
                Change::Updated => "updated",
                Change::Deleted => "deleted",
            },
            step: match &e.result {
                Ok(Undone::Restored) => Step::Restore,
                Ok(Undone::Removed) => Step::Remove,
                Ok(Undone::AlreadyGone) => Step::Gone,
                Err(error) => Step::Failed(match error.hint() {
                    Some(hint) => format!("{error} ({hint})"),
                    None => error.to_string(),
                }),
            },
        })
        .collect();
    UndoView {
        run: report.from.clone(),
        date: describe_run(&report.from),
        command,
        lines,
        applied,
        saved_as: report.saved.as_ref().map(|s| s.id.clone()),
        warnings: report
            .saved
            .as_ref()
            .and_then(|s| s.prune_error.as_ref())
            .map(|e| format!("Old backups could not be removed: {e}"))
            .into_iter()
            .collect(),
    }
}
