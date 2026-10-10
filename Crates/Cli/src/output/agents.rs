//! Rows, the JSON documents and the text of the `agents` commands.

use std::path::PathBuf;

use serde::Serialize;
use skillmirror_core::agents::{Action, AgentsPlan, Applied, FileState, Written};

use super::agents_json::{Counts, StatusCounts};
use super::diff::styled_diff;
use super::human::plural;
use super::style::{ERROR, HEADER, MUTED, SUCCESS, WARNING};

/// Plan words (`create`) or done words (`created`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tense {
    Plan,
    Done,
}

/// One project's AGENTS.md, for the table and for `--json`.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct AgentRow {
    #[serde(serialize_with = "super::lossy::path")]
    pub(crate) project: PathBuf,
    /// `missing`, `no_block`, `current`, `outdated`, `edited`, `broken` or `problem`.
    pub(crate) state: &'static str,
    /// What the command does, or did: `create`, `insert`, `update`, `unchanged`, `skipped` or `failed`
    /// (`created`, `inserted`, `updated` once done).
    pub(crate) action: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) detail: Option<String>,
}

fn state_code(state: &FileState) -> &'static str {
    match state {
        FileState::Missing => "missing",
        FileState::NoBlock => "no_block",
        FileState::Current => "current",
        FileState::Outdated => "outdated",
        FileState::Edited => "edited",
        FileState::Broken(_) => "broken",
        FileState::Problem(_) => "problem",
    }
}

impl AgentRow {
    /// A row for `agents status`: the state only.
    pub(crate) fn of_state(project: PathBuf, state: &FileState) -> Self {
        Self {
            project,
            state: state_code(state),
            action: "none",
            detail: state.detail().map(str::to_string),
        }
    }

    /// A row for what a plan intends.
    pub(crate) fn planned(entry: &skillmirror_core::agents::AgentsEntry) -> Self {
        let (action, detail) = match &entry.action {
            Action::Create => ("create", None),
            Action::Insert => ("insert", None),
            Action::Update => ("update", None),
            Action::Unchanged => ("unchanged", None),
            Action::Skipped(why) => ("skipped", Some((*why).to_string())),
            Action::Failed(why) => ("failed", Some(why.clone())),
        };
        Self {
            project: entry.project.clone(),
            state: state_code(&entry.state),
            action,
            detail,
        }
    }

    /// A row for what was done.
    pub(crate) fn done(planned: &Self, applied: &Applied) -> Self {
        let (action, detail) = match &applied.result {
            Ok(Written::Created) => ("created", None),
            Ok(Written::Inserted) => ("inserted", None),
            Ok(Written::Updated) => ("updated", None),
            Ok(Written::Unchanged) => (planned.action, planned.detail.clone()),
            Err(e) => ("failed", Some(e.to_string())),
        };
        Self {
            action,
            detail,
            ..planned.clone()
        }
    }

    pub(super) fn changes(&self) -> bool {
        matches!(
            self.action,
            "create" | "insert" | "update" | "created" | "inserted" | "updated"
        )
    }
}

fn symbol(row: &AgentRow) -> (&'static str, anstyle::Style) {
    match row.state {
        "current" => ("✓", SUCCESS),
        "broken" | "problem" => ("✗", ERROR),
        _ => ("~", WARNING),
    }
}

/// Projects with something to say are listed; the current ones only with `show_all`.
pub(crate) fn render_agents_status(rows: &[AgentRow], source: &str, show_all: bool) -> String {
    let mut out = String::new();
    putln!(out, "{MUTED}text: {source}{MUTED:#}");
    for row in rows.iter().filter(|r| show_all || r.state != "current") {
        let (sym, style) = symbol(row);
        let detail = row
            .detail
            .as_deref()
            .map(|d| format!(": {d}"))
            .unwrap_or_default();
        putln!(
            out,
            "  {style}{sym}{style:#} {} {MUTED}{}{detail}{MUTED:#}",
            row.project.display(),
            row.state.replace('_', " ")
        );
    }
    let counts = StatusCounts::of(rows);
    let mut parts = vec![format!("{} current", counts.current)];
    for (n, word) in [
        (counts.outdated, "outdated"),
        (counts.edited, "edited by hand"),
        (counts.missing, "without AGENTS.md"),
        (counts.no_block, "without a block"),
        (counts.broken, "broken"),
        (counts.problem, "unusable"),
    ] {
        if n > 0 {
            parts.push(format!("{n} {word}"));
        }
    }
    let style = if counts.drifting() + counts.unusable() > 0 {
        WARNING
    } else {
        SUCCESS
    };
    putln!(
        out,
        "\n{style}{}{style:#} in {}",
        parts.join(", "),
        plural(counts.projects, "project")
    );
    if counts.outdated + counts.edited > 0 {
        putln!(
            out,
            "{MUTED}`skillmirror agents sync` updates the blocks that are out of date.{MUTED:#}"
        );
    }
    if counts.missing + counts.no_block > 0 {
        putln!(
            out,
            "{MUTED}`skillmirror agents add --project DIR` makes the file or puts the block in.{MUTED:#}"
        );
    }
    out
}

/// The rows of a plan or a result. Projects where nothing changes are listed only with `show_all`.
pub(crate) fn render_agents_plan(
    rows: &[AgentRow],
    source: &str,
    tense: Tense,
    show_all: bool,
) -> String {
    let mut out = String::new();
    putln!(out, "{MUTED}text: {source}{MUTED:#}");
    for row in rows
        .iter()
        .filter(|r| show_all || r.changes() || r.action == "failed")
    {
        let (sym, style) = match row.action {
            "failed" => ("✗", ERROR),
            "unchanged" | "skipped" => ("=", MUTED),
            _ => ("+", SUCCESS),
        };
        let detail = row
            .detail
            .as_deref()
            .map(|d| format!(": {d}"))
            .unwrap_or_default();
        putln!(
            out,
            "  {style}{sym}{style:#} {} {MUTED}{}{detail}{MUTED:#}",
            row.project.display(),
            row.action
        );
    }
    let counts = Counts::of(rows);
    let verb = if tense == Tense::Plan {
        "to change"
    } else {
        "changed"
    };
    let mut parts = vec![
        format!("{} {verb}", counts.changes),
        format!("{} unchanged", counts.unchanged),
    ];
    if counts.skipped > 0 {
        parts.push(format!("{} not for this command", counts.skipped));
    }
    if counts.failed > 0 {
        parts.push(format!("{} failed", counts.failed));
    }
    putln!(out, "\n{HEADER}{}{HEADER:#}", parts.join(", "));
    out
}

/// The lines that change in each file of the plan.
pub(crate) fn render_diffs(plan: &AgentsPlan) -> String {
    let mut out = String::new();
    for entry in plan.entries.iter().filter(|e| e.writes()) {
        putln!(out, "\n{HEADER}● {}{HEADER:#}", entry.path.display());
        out.push_str(&styled_diff(&entry.diff()));
    }
    out
}

/// The AGENTS.md of a new project, as one section after the skills `init` installs.
pub(crate) fn render_agents_line(row: &AgentRow, source: &str) -> String {
    let mut out = String::new();
    putln!(out, "\n{HEADER}● AGENTS.md{HEADER:#}");
    let (sym, style) = match row.action {
        "failed" => ("✗", ERROR),
        "unchanged" | "skipped" => ("=", MUTED),
        _ => ("+", SUCCESS),
    };
    let detail = row
        .detail
        .as_deref()
        .map(|d| format!(": {d}"))
        .unwrap_or_default();
    putln!(
        out,
        "  {style}{sym}{style:#} {} {MUTED}{}{detail} (from {source}){MUTED:#}",
        row.project.join("AGENTS.md").display(),
        row.action
    );
    out
}
