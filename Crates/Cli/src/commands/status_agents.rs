//! The line `status` adds when the AGENTS.md blocks of the projects it lists are behind the text.

use std::path::Path;

use skillmirror_core::agents::{FileState, inspect_project, load_source};

use crate::output;

/// Says how many blocks are out of date or were edited by hand, or nothing when all is well. A text that
/// cannot be used is a warning here: `doctor` and the `agents` commands stop on it.
pub(super) fn hint<'a>(vault: &Path, projects: impl Iterator<Item = &'a Path>) {
    let source = match load_source(vault) {
        Ok(source) => source,
        Err(error) => {
            output::warn_line(&format!(
                "AGENTS.md: {error}; `skillmirror doctor` has the details"
            ));
            return;
        }
    };
    let (mut outdated, mut edited) = (0, 0);
    for project in projects {
        match inspect_project(project, &source) {
            FileState::Outdated => outdated += 1,
            FileState::Edited => edited += 1,
            _ => {}
        }
    }
    let mut parts = Vec::new();
    if outdated > 0 {
        parts.push(format!("{outdated} out of date"));
    }
    if edited > 0 {
        parts.push(format!("{edited} edited by hand"));
    }
    if !parts.is_empty() {
        output::line(&format!(
            "\nAGENTS.md blocks: {}. `skillmirror agents status` has the list, `skillmirror agents sync` brings them up to date.",
            parts.join(", ")
        ));
    }
}
