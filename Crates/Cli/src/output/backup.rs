//! Rows, JSON documents and text of `undo` and `history`.

use std::path::PathBuf;

use serde::Serialize;

use super::human::{pad, plural};
use super::style::{ERROR, MUTED, NAME, SUCCESS, WARNING};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum UndoStatus {
    WouldRestore,
    WouldRemove,
    Restored,
    Removed,
    AlreadyGone,
    Failed {
        message: String,
        hint: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct UndoRow {
    pub(crate) project: PathBuf,
    pub(crate) skill: String,
    /// What the undone run had done to the folder: `created`, `updated` or `deleted`.
    pub(crate) was: &'static str,
    pub(crate) status: UndoStatus,
}

#[derive(Serialize)]
pub(crate) struct UndoJson<'a> {
    run: &'a str,
    dry_run: bool,
    entries: &'a [UndoRow],
    /// The run that saved what the undo replaced. Undoing that run is a redo.
    saved_as: Option<&'a str>,
}

impl<'a> UndoJson<'a> {
    pub(crate) fn new(
        run: &'a str,
        dry_run: bool,
        entries: &'a [UndoRow],
        saved_as: Option<&'a str>,
    ) -> Self {
        Self {
            run,
            dry_run,
            entries,
            saved_as,
        }
    }

    pub(crate) fn render(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// One run in the backup store.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoryRow {
    pub(crate) id: String,
    /// The id as a date, `2026-10-08 12:34:56 UTC`.
    pub(crate) date: String,
    pub(crate) command: Option<&'static str>,
    pub(crate) skills: usize,
    pub(crate) projects: usize,
    /// Set when the run's notes could not be read.
    pub(crate) error: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct HistoryJson<'a> {
    runs: &'a [HistoryRow],
}

impl<'a> HistoryJson<'a> {
    pub(crate) fn new(runs: &'a [HistoryRow]) -> Self {
        Self { runs }
    }

    pub(crate) fn render(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

pub(crate) fn render_undo(rows: &[UndoRow], dry_run: bool) -> String {
    let mut out = String::new();
    let (mut restored, mut removed, mut failed) = (0, 0, 0);
    for row in rows {
        let name = pad(&row.skill);
        let project = row.project.display();
        match &row.status {
            UndoStatus::WouldRestore | UndoStatus::Restored => {
                restored += 1;
                let verb = if dry_run {
                    "would restore in"
                } else {
                    "restored in"
                };
                putln!(
                    out,
                    "  {SUCCESS}↺{SUCCESS:#} {NAME}{name}{NAME:#} {MUTED}{verb} {project}{MUTED:#}"
                );
            }
            UndoStatus::WouldRemove | UndoStatus::Removed => {
                removed += 1;
                let verb = if dry_run {
                    "would remove from"
                } else {
                    "removed from"
                };
                putln!(
                    out,
                    "  {WARNING}-{WARNING:#} {NAME}{name}{NAME:#} {MUTED}{verb} {project}{MUTED:#}"
                );
            }
            UndoStatus::AlreadyGone => {
                putln!(
                    out,
                    "  {MUTED}· {name} already gone from {project}{MUTED:#}"
                );
            }
            UndoStatus::Failed { message, hint } => {
                failed += 1;
                putln!(
                    out,
                    "  {ERROR}✗{ERROR:#} {NAME}{name}{NAME:#} {project}: {message}"
                );
                if let Some(hint) = hint {
                    putln!(out, "      {MUTED}hint: {hint}{MUTED:#}");
                }
            }
        }
    }
    let mut parts = Vec::new();
    let (back, gone) = if dry_run {
        ("to restore", "to remove")
    } else {
        ("restored", "removed")
    };
    for (count, label) in [(restored, back), (removed, gone), (failed, "failed")] {
        if count > 0 {
            parts.push(format!("{} {label}", plural(count, "folder")));
        }
    }
    let note = if dry_run { " (dry run)" } else { "" };
    let detail = if parts.is_empty() {
        "nothing to do".to_string()
    } else {
        parts.join(", ")
    };
    let style = if failed > 0 { WARNING } else { SUCCESS };
    put!(out, "\n{style}{detail}{note}{style:#}\n");
    out
}

pub(crate) fn render_history(rows: &[HistoryRow]) -> String {
    if rows.is_empty() {
        return "No backups yet. A sync, push, add, init or delete that changes something leaves one.\n"
            .to_string();
    }
    let mut out = String::new();
    for row in rows {
        if let Some(error) = &row.error {
            putln!(out, "{ERROR}✗{ERROR:#} {} {MUTED}{error}{MUTED:#}", row.id);
            continue;
        }
        putln!(
            out,
            "{NAME}{}{NAME:#}  {:<7} {} in {}  {MUTED}{}{MUTED:#}",
            row.date,
            row.command.unwrap_or("?"),
            plural(row.skills, "folder"),
            plural(row.projects, "project"),
            row.id
        );
    }
    put!(
        out,
        "\n{MUTED}Newest first. `skillmirror undo <RUN>` brings a run back; the newest is the default.{MUTED:#}\n"
    );
    out
}
