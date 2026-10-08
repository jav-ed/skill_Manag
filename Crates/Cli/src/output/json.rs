//! JSON documents for `--json`. One document per run on stdout, nothing else.

use serde::Serialize;

use super::view::{DeleteRow, InstalledRow, Row, SkillRow, Summary};

#[derive(Serialize)]
struct Issue {
    path: String,
    message: String,
}

/// Result of `sync` or `push`.
#[derive(Serialize)]
pub(crate) struct RunJson<'a> {
    command: &'a str,
    /// `dry-run`, `check` or `apply`.
    mode: &'a str,
    summary: Summary,
    entries: &'a [Row],
    scan_issues: Vec<Issue>,
}

impl<'a> RunJson<'a> {
    pub(crate) fn new(
        command: &'a str,
        mode: &'a str,
        entries: &'a [Row],
        issues: &[skillmirror_core::scan::ScanIssue],
    ) -> Self {
        Self {
            command,
            mode,
            summary: Summary::of(entries),
            entries,
            scan_issues: issues
                .iter()
                .map(|i| Issue {
                    path: i.path.display().to_string(),
                    message: i.message.clone(),
                })
                .collect(),
        }
    }

    pub(crate) fn render(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[derive(Serialize)]
pub(crate) struct ListJson<'a> {
    installed: &'a [InstalledRow],
}

impl<'a> ListJson<'a> {
    pub(crate) fn new(installed: &'a [InstalledRow]) -> Self {
        Self { installed }
    }

    pub(crate) fn render(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[derive(Serialize)]
pub(crate) struct DeleteJson<'a> {
    dry_run: bool,
    deleted: &'a [DeleteRow],
}

impl<'a> DeleteJson<'a> {
    pub(crate) fn new(dry_run: bool, deleted: &'a [DeleteRow]) -> Self {
        Self { dry_run, deleted }
    }

    pub(crate) fn render(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[derive(Serialize)]
pub(crate) struct VaultJson<'a> {
    skills: &'a [SkillRow],
}

impl<'a> VaultJson<'a> {
    pub(crate) fn new(skills: &'a [SkillRow]) -> Self {
        Self { skills }
    }

    pub(crate) fn render(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
