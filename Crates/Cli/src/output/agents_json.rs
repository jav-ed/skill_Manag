//! The counts and the JSON documents of the `agents` commands.

use serde::Serialize;

use super::agents::AgentRow;

#[derive(Debug, Default, Clone, Copy, Serialize)]
pub(crate) struct Counts {
    pub(super) projects: usize,
    pub(super) changes: usize,
    pub(super) unchanged: usize,
    pub(super) skipped: usize,
    pub(super) failed: usize,
}

impl Counts {
    pub(crate) fn of(rows: &[AgentRow]) -> Self {
        let count = |pick: fn(&AgentRow) -> bool| rows.iter().filter(|r| pick(r)).count();
        Self {
            projects: rows.len(),
            changes: count(AgentRow::changes),
            unchanged: count(|r| r.action == "unchanged"),
            skipped: count(|r| r.action == "skipped"),
            failed: count(|r| r.action == "failed"),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct AgentsStatusJson<'a> {
    source: String,
    projects: &'a [AgentRow],
    summary: StatusCounts,
}

#[derive(Debug, Default, Clone, Copy, Serialize)]
pub(crate) struct StatusCounts {
    pub(super) projects: usize,
    pub(super) current: usize,
    pub(super) outdated: usize,
    pub(super) edited: usize,
    pub(super) missing: usize,
    pub(super) no_block: usize,
    pub(super) broken: usize,
    pub(super) problem: usize,
}

impl StatusCounts {
    pub(crate) fn of(rows: &[AgentRow]) -> Self {
        let count = |state: &str| rows.iter().filter(|r| r.state == state).count();
        Self {
            projects: rows.len(),
            current: count("current"),
            outdated: count("outdated"),
            edited: count("edited"),
            missing: count("missing"),
            no_block: count("no_block"),
            broken: count("broken"),
            problem: count("problem"),
        }
    }

    /// Projects that are neither current nor unusable.
    pub(crate) fn drifting(self) -> usize {
        self.outdated + self.edited + self.missing + self.no_block
    }

    /// Projects whose file cannot be used at all.
    pub(crate) fn unusable(self) -> usize {
        self.broken + self.problem
    }
}

impl<'a> AgentsStatusJson<'a> {
    pub(crate) fn new(source: String, projects: &'a [AgentRow]) -> Self {
        Self {
            source,
            summary: StatusCounts::of(projects),
            projects,
        }
    }

    pub(crate) fn render(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Result of `agents sync` or `agents add`.
#[derive(Serialize)]
pub(crate) struct AgentsRunJson<'a> {
    command: &'a str,
    /// `dry-run` or `apply`.
    mode: &'a str,
    source: String,
    projects: &'a [AgentRow],
    summary: Counts,
    /// The backup run that holds what this run replaced; `skillmirror undo` brings it back.
    backup: Option<String>,
}

impl<'a> AgentsRunJson<'a> {
    pub(crate) fn new(
        command: &'a str,
        mode: &'a str,
        source: String,
        projects: &'a [AgentRow],
    ) -> Self {
        Self {
            command,
            mode,
            source,
            projects,
            summary: Counts::of(projects),
            backup: None,
        }
    }

    pub(crate) fn with_backup(mut self, backup: Option<&str>) -> Self {
        self.backup = backup.map(str::to_string);
        self
    }

    pub(crate) fn render(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
