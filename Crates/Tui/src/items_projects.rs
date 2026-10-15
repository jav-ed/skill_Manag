//! The panes of the sync, push and delete pages: for the skill under the cursor, the projects it is in.

use std::path::Path;

use skillmirror_core::ops::Installed;

use crate::items::{Note, Tone};
use crate::items_skills::{InProject, in_project};
use crate::num::plural;
use crate::session::{State, TargetState};

/// Which projects the pane of a sync or push row lists; `tab` goes to the next one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum PaneView {
    /// What the run would change, and what it cannot apply.
    #[default]
    Affected,
    /// The projects that are up to date.
    Others,
    /// Every project, in groups.
    All,
}

impl PaneView {
    pub(crate) fn next(self) -> Self {
        match self {
            Self::Affected => Self::Others,
            Self::Others => Self::All,
            Self::All => Self::Affected,
        }
    }
}

/// The pane of one row in each of its views, written when the page is built so that drawing and moving
/// the cursor never have to look at the disk.
#[derive(Debug, Clone)]
pub(crate) struct Views {
    affected: Vec<Note>,
    others: Vec<Note>,
    all: Vec<Note>,
}

impl Views {
    pub(crate) fn of(&self, view: PaneView) -> &[Note] {
        match view {
            PaneView::Affected => &self.affected,
            PaneView::Others => &self.others,
            PaneView::All => &self.all,
        }
    }
}

fn line(text: impl Into<String>, tone: Tone) -> Note {
    Note {
        text: text.into(),
        tone,
    }
}

/// A project as the panes write it: its path below the scan root.
fn below_root(project: &Path, root: &Path) -> String {
    // The walker yields projects below the root, so this only ever cuts; a path outside would be shown whole.
    project
        .strip_prefix(root)
        .unwrap_or(project)
        .to_string_lossy()
        .into_owned()
}

/// What a run does in one project. The order is the order of the groups in the pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Changing,
    Failing,
    Same,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Row {
    kind: Kind,
    path: String,
    text: String,
}

impl Row {
    fn note(&self) -> Note {
        let tone = match self.kind {
            Kind::Changing => Tone::Change,
            Kind::Failing => Tone::Problem,
            Kind::Same => Tone::Quiet,
        };
        line(format!("{}  {}", self.path, self.text).trim_end(), tone)
    }
}

fn row(state: &TargetState, root: &Path) -> Row {
    let InProject { project, text, .. } = in_project(state);
    let (kind, text) = match state.state {
        State::Create(_) | State::Update(_) => (Kind::Changing, text),
        State::Failed(_) => (Kind::Failing, text),
        // "up to date" is what the group says; the line is only the project.
        State::Same => (Kind::Same, String::new()),
    };
    Row {
        kind,
        path: below_root(project, root),
        text,
    }
}

/// The top lines of every view: the skill, what the run does to it, and the three views with the open one
/// in brackets.
fn head(skill: &str, counts: [usize; 3], active: PaneView) -> Vec<Note> {
    let [changing, failing, same] = counts;
    let summary = match (changing, failing) {
        (0, 0) => "all up to date".to_string(),
        (n, 0) => format!("{n} to update"),
        (0, f) => format!("{f} cannot be applied"),
        (n, f) => format!("{n} to update, {f} cannot be applied"),
    };
    let tab = |view: PaneView, name: &str, n: usize| {
        if view == active {
            format!("[{name} {n}]")
        } else {
            format!("{name} {n}")
        }
    };
    vec![
        line(skill, Tone::Heading),
        line(
            format!(
                "{}, {summary}",
                plural(changing + failing + same, "project")
            ),
            Tone::Quiet,
        ),
        line(
            format!(
                "{}  {}  {}",
                tab(PaneView::Affected, "Affected", changing + failing),
                tab(PaneView::Others, "Others", same),
                tab(PaneView::All, "All", changing + failing + same),
            ),
            Tone::Heading,
        ),
    ]
}

/// The pane of a sync or push row. Rows are in path order within their kind, so the projects that
/// change come first.
pub(crate) fn change_pane(skill: &str, states: &[&TargetState], root: &Path) -> Views {
    let mut rows: Vec<Row> = states.iter().map(|state| row(state, root)).collect();
    rows.sort();
    let count = |kind: Kind| rows.iter().filter(|r| r.kind == kind).count();
    let counts = [
        count(Kind::Changing),
        count(Kind::Failing),
        count(Kind::Same),
    ];

    let mut affected = head(skill, counts, PaneView::Affected);
    if counts[0] + counts[1] == 0 {
        affected.push(line("Nothing would change in any project.", Tone::Quiet));
    }
    affected.extend(rows.iter().filter(|r| r.kind != Kind::Same).map(Row::note));

    let mut others = head(skill, counts, PaneView::Others);
    if counts[2] == 0 {
        others.push(line("Every project is affected.", Tone::Quiet));
    }
    others.extend(rows.iter().filter(|r| r.kind == Kind::Same).map(Row::note));

    let mut all = head(skill, counts, PaneView::All);
    for (kind, heading) in [
        (Kind::Changing, "Would change"),
        (Kind::Failing, "Cannot be applied"),
        (Kind::Same, "Up to date"),
    ] {
        let members: Vec<&Row> = rows.iter().filter(|r| r.kind == kind).collect();
        if members.is_empty() {
            continue;
        }
        all.push(line(
            format!("{heading} ({})", members.len()),
            Tone::Heading,
        ));
        all.extend(members.into_iter().map(Row::note));
    }

    Views {
        affected,
        others,
        all,
    }
}

/// The pane of a delete row: every project the skill would be removed from.
pub(crate) fn delete_pane(skill: &str, rows: &[&Installed], root: &Path) -> Vec<Note> {
    let mut projects: Vec<(String, bool)> = rows
        .iter()
        .map(|row| {
            (
                below_root(&row.target.project, root),
                row.in_vault == Some(false),
            )
        })
        .collect();
    projects.sort();
    let mut out = vec![
        line(skill, Tone::Heading),
        line(
            format!(
                "Would be removed from {}",
                plural(projects.len(), "project")
            ),
            Tone::Quiet,
        ),
    ];
    out.extend(projects.into_iter().map(|(path, not_in_vault)| {
        let text = if not_in_vault {
            format!("{path}  not in the vault")
        } else {
            path
        };
        line(text, Tone::Change)
    }));
    out
}
