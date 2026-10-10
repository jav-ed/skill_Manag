//! What `agents seed` prints.

use std::path::Path;

use serde::Serialize;

use super::style::{HEADER, MUTED};

#[derive(Debug, Serialize)]
pub(crate) struct AgentsSeedJson<'a> {
    command: &'static str,
    dry_run: bool,
    #[serde(serialize_with = "super::lossy::path")]
    path: &'a Path,
}

impl<'a> AgentsSeedJson<'a> {
    pub(crate) fn new(dry_run: bool, path: &'a Path) -> Self {
        Self {
            command: "agents seed",
            dry_run,
            path,
        }
    }

    pub(crate) fn render(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// The text after the built-in text was written into the vault, or would be (`dry_run`).
pub(crate) fn render_agents_seed(path: &Path, dry_run: bool) -> String {
    let mut out = String::new();
    let tense = if dry_run { "Would write" } else { "Wrote" };
    putln!(
        out,
        "{HEADER}{tense}{HEADER:#} the built-in text to {}",
        path.display()
    );
    if !dry_run {
        putln!(
            out,
            "{MUTED}Edit it there and commit it in the vault. Then `skillmirror agents sync` puts it into the projects: only the managed block of each AGENTS.md changes.{MUTED:#}"
        );
    }
    out
}
