//! The backup run around every writing command, and the `undo` and `history` commands.

use std::collections::BTreeSet;

use skillmirror_core::Hint;
use skillmirror_core::backup::{
    Backups, Change, Filter, Finished, Run, RunKind, UndoReport, Undone, describe_run, undo,
};
use skillmirror_core::config::Dirs;
use skillmirror_core::events::ignore_events;

use crate::args::{HistoryArgs, UndoArgs};
use crate::exit::Exit;
use crate::output::{
    self, HistoryJson, HistoryRow, UndoJson, UndoRow, UndoStatus, render_history, render_undo,
};
use crate::report::CliError;

fn store() -> Result<Backups, CliError> {
    Ok(Backups::in_dirs(&Dirs::from_env()?))
}

/// Starts the run that keeps what a writing command replaces or removes. Nothing is written until it stores something.
pub(super) fn begin(kind: RunKind) -> Result<(Backups, Run), CliError> {
    let backups = store()?;
    let run = backups.begin(kind)?;
    Ok((backups, run))
}

/// The id of the run, when it kept anything.
pub(super) fn saved(done: &Finished) -> Option<&str> {
    (done.stored > 0).then_some(done.id.as_str())
}

fn warn_prune(done: &Finished) {
    if let Some(error) = &done.prune_error {
        output::warn_line(&format!("old backups could not be removed: {error}"));
    }
}

/// Tells the user where the backup is. With `--json` the id is in the document instead.
pub(super) fn announce(done: &Finished, json: bool) {
    warn_prune(done);
    if let (Some(id), false) = (saved(done), json) {
        output::line(&format!("Backup: run {id} (undo with: skillmirror undo)"));
    }
}

pub(super) fn run_undo(args: &UndoArgs) -> Result<Exit, CliError> {
    let backups = store()?;
    let filter = Filter {
        project: args.project.as_deref().map(absolute).transpose()?,
        skill: args.skill.clone(),
    };
    let id = args.run.as_deref();
    // The dry run checks every entry against the disk, so problems show before anything is touched.
    let preview = undo(&backups, id, &filter, true, &ignore_events)?;
    let rows = rows_of(&preview, true);
    if preview.entries.is_empty() {
        return nothing_matches(args, &preview.from);
    }
    let actionable = preview.entries.iter().filter(|e| e.result.is_ok()).count();
    if args.dry_run || actionable == 0 {
        show(args, &preview.from, true, &rows, None)?;
        return Ok(if preview.failed() > 0 {
            Exit::Partial
        } else {
            Exit::Clean
        });
    }
    if !args.yes {
        if !args.json {
            output::print(&render_undo(&rows, true));
        }
        if !confirm(&backups, &preview)? {
            output::line("Cancelled, nothing was changed.");
            return Ok(Exit::Clean);
        }
    }
    let report = undo(
        &backups,
        Some(&preview.from),
        &filter,
        false,
        &ignore_events,
    )?;
    let saved_as = report.saved.as_ref().map(|s| s.id.as_str());
    show(
        args,
        &report.from,
        false,
        &rows_of(&report, false),
        saved_as,
    )?;
    if let Some(done) = &report.saved {
        warn_prune(done);
        if !args.json {
            output::line(&format!(
                "Saved what was replaced as run {} (undo again to redo this undo).",
                done.id
            ));
        }
    }
    Ok(if report.failed() > 0 {
        Exit::Partial
    } else {
        Exit::Clean
    })
}

fn nothing_matches(args: &UndoArgs, from: &str) -> Result<Exit, CliError> {
    if args.json {
        output::line(&UndoJson::new(from, args.dry_run, &[], None).render()?);
    } else {
        output::line(&format!("Nothing in run {from} matches."));
    }
    Ok(Exit::Clean)
}

fn show(
    args: &UndoArgs,
    from: &str,
    dry_run: bool,
    rows: &[UndoRow],
    saved_as: Option<&str>,
) -> Result<(), CliError> {
    if args.json {
        output::line(&UndoJson::new(from, dry_run, rows, saved_as).render()?);
    } else {
        output::print(&render_undo(rows, dry_run));
    }
    Ok(())
}

fn confirm(backups: &Backups, preview: &UndoReport) -> Result<bool, CliError> {
    if !output::can_prompt() {
        return Err(CliError::usage(
            "refusing to undo without confirmation",
            "pass --yes to undo, or --dry-run to preview",
        ));
    }
    let command = backups
        .load(&preview.from)?
        .kind()
        .map_or("run", RunKind::name);
    let projects: BTreeSet<_> = preview.entries.iter().map(|e| &e.project).collect();
    let question = format!(
        "Undo the {command} of {}: {} folder(s) in {} project(s)?",
        describe_run(&preview.from),
        preview.entries.len(),
        projects.len()
    );
    Ok(output::confirm(&question)?)
}

fn rows_of(report: &UndoReport, dry_run: bool) -> Vec<UndoRow> {
    report
        .entries
        .iter()
        .map(|e| UndoRow {
            project: e.project.clone(),
            skill: e.skill.clone(),
            was: match e.change {
                Change::Created => "created",
                Change::Updated => "updated",
                Change::Deleted => "deleted",
            },
            status: match (&e.result, dry_run) {
                (Ok(Undone::Restored), true) => UndoStatus::WouldRestore,
                (Ok(Undone::Restored), false) => UndoStatus::Restored,
                (Ok(Undone::Removed), true) => UndoStatus::WouldRemove,
                (Ok(Undone::Removed), false) => UndoStatus::Removed,
                (Ok(Undone::AlreadyGone), _) => UndoStatus::AlreadyGone,
                (Err(error), _) => UndoStatus::Failed {
                    message: error.to_string(),
                    hint: error.hint(),
                },
            },
        })
        .collect()
}

pub(super) fn run_history(args: &HistoryArgs) -> Result<Exit, CliError> {
    let backups = store()?;
    let mut rows = Vec::new();
    for id in backups.run_ids()? {
        match backups.load(&id) {
            // A run whose entries were all spent is only an empty folder.
            Ok(run) if run.entries.is_empty() => {}
            Ok(run) => rows.push(HistoryRow {
                date: describe_run(&id),
                command: run.kind().map(RunKind::name),
                skills: run.entries.len(),
                projects: run
                    .entries
                    .iter()
                    .map(|e| &e.entry.project)
                    .collect::<BTreeSet<_>>()
                    .len(),
                error: None,
                id,
            }),
            Err(error) => rows.push(HistoryRow {
                date: describe_run(&id),
                command: None,
                skills: 0,
                projects: 0,
                error: Some(error.to_string()),
                id,
            }),
        }
    }
    if args.json {
        output::line(&HistoryJson::new(&rows).render()?);
    } else {
        output::print(&render_history(&rows));
    }
    Ok(if rows.iter().any(|r| r.error.is_some()) {
        Exit::Partial
    } else {
        Exit::Clean
    })
}

/// Entries store the project path the scan found, so symlinks are resolved when the folder still exists.
fn absolute(path: &std::path::Path) -> Result<std::path::PathBuf, CliError> {
    match path.canonicalize() {
        Ok(resolved) => Ok(resolved),
        Err(_) => Ok(std::path::absolute(path)?),
    }
}
