//! The rows of a selection screen, built from a session.

use std::collections::BTreeMap;

use skillmirror_core::scan::Target;

use crate::num::plural;
use crate::results::short_path;
use crate::session::{Session, State, TargetState};

/// Which selection screen is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    Sync,
    Push,
    Delete,
    List,
}

/// How a note next to a row is coloured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tone {
    /// Something will change.
    Change,
    /// Nothing to do.
    Quiet,
    /// Something is wrong.
    Problem,
}

#[derive(Debug, Clone)]
pub(crate) struct Note {
    pub(crate) text: String,
    pub(crate) tone: Tone,
}

/// One row: a skill with all its projects, or (in the list) one skill in one project.
#[derive(Debug, Clone)]
pub(crate) struct Item {
    pub(crate) name: String,
    pub(crate) detail: String,
    pub(crate) note: Option<Note>,
    pub(crate) targets: Vec<Target>,
    pub(crate) preselected: bool,
}

pub(crate) fn build(mode: Mode, session: &Session) -> Result<Vec<Item>, String> {
    match mode {
        Mode::Sync => Ok(planned(&session.sync)),
        Mode::Push => session
            .push
            .as_ref()
            .map(|s| planned(s))
            .map_err(Clone::clone),
        Mode::Delete => Ok(installed_groups(session)),
        Mode::List => Ok(installed_rows(session)),
    }
}

/// Sync and push rows: grouped by skill, with what applying would do. Rows with changes start selected.
fn planned(states: &[TargetState]) -> Vec<Item> {
    let mut groups: BTreeMap<&str, Vec<&TargetState>> = BTreeMap::new();
    for state in states {
        groups.entry(&state.target.skill).or_default().push(state);
    }
    groups
        .into_iter()
        .map(|(name, members)| {
            let changing = members
                .iter()
                .filter(|m| matches!(m.state, State::Create(_) | State::Update(_)))
                .count();
            let failing = members
                .iter()
                .filter(|m| matches!(m.state, State::Failed(_)))
                .count();
            Item {
                name: name.to_string(),
                detail: plural(members.len(), "project"),
                note: Some(planned_note(changing, failing)),
                targets: members.iter().map(|m| m.target.clone()).collect(),
                preselected: changing > 0,
            }
        })
        .collect()
}

fn planned_note(changing: usize, failing: usize) -> Note {
    if failing > 0 {
        return Note {
            text: format!("{} cannot be applied", plural(failing, "project")),
            tone: Tone::Problem,
        };
    }
    if changing > 0 {
        return Note {
            text: format!("{changing} to update"),
            tone: Tone::Change,
        };
    }
    Note {
        text: "up to date".to_string(),
        tone: Tone::Quiet,
    }
}

/// Delete rows: every installed skill, grouped by name, nothing selected.
fn installed_groups(session: &Session) -> Vec<Item> {
    let mut groups: BTreeMap<&str, Vec<&skillmirror_core::ops::Installed>> = BTreeMap::new();
    for row in &session.installed {
        groups.entry(&row.target.skill).or_default().push(row);
    }
    groups
        .into_iter()
        .map(|(name, members)| Item {
            name: name.to_string(),
            detail: plural(members.len(), "project"),
            note: members
                .iter()
                .any(|m| m.in_vault == Some(false))
                .then(|| Note {
                    text: "not in the vault".to_string(),
                    tone: Tone::Quiet,
                }),
            targets: members.iter().map(|m| m.target.clone()).collect(),
            preselected: false,
        })
        .collect()
}

/// List rows: one per installed folder, in walk order.
fn installed_rows(session: &Session) -> Vec<Item> {
    session
        .installed
        .iter()
        .map(|row| Item {
            name: row.target.skill.clone(),
            detail: short_path(&row.target.project.display().to_string()),
            note: (row.in_vault == Some(false)).then(|| Note {
                text: "not in the vault".to_string(),
                tone: Tone::Quiet,
            }),
            targets: vec![row.target.clone()],
            preselected: false,
        })
        .collect()
}
