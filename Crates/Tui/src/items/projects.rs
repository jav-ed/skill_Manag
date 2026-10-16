//! The data of the pane of the sync, push and delete pages: for the skill under the cursor, its projects.

use std::path::Path;

use skillmirror_core::ops::Installed;

use super::Tone;
use super::skills::{InProject, in_project};
use crate::num::plural;
use crate::results::short_path;
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

/// How the pane writes a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum PathStyle {
    /// `parent/project`: enough to know the project, one row each.
    #[default]
    Short,
    /// The whole path below the scan root.
    Long,
}

impl PathStyle {
    pub(crate) fn toggled(self) -> Self {
        match self {
            Self::Short => Self::Long,
            Self::Long => Self::Short,
        }
    }
}

/// What a run does in one project. The order is the order of the groups in the pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Kind {
    Changing,
    Failing,
    Same,
}

/// One project of the pane.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ProjectRow {
    pub(crate) kind: Kind,
    /// The path below the scan root.
    pub(crate) long: String,
    /// The last two folders of it.
    pub(crate) short: String,
    /// What happens there: the file count, the reason of a failure; empty for a project that is up to date.
    pub(crate) text: String,
}

impl ProjectRow {
    pub(crate) fn path(&self, style: PathStyle) -> &str {
        match style {
            PathStyle::Short => &self.short,
            PathStyle::Long => &self.long,
        }
    }

    pub(crate) fn tone(&self) -> Tone {
        match self.kind {
            Kind::Changing => Tone::Change,
            Kind::Failing => Tone::Problem,
            Kind::Same => Tone::Quiet,
        }
    }
}

/// The pane of one row, written when the page is built so that drawing and moving the cursor never have to
/// look at the disk.
#[derive(Debug, Clone)]
pub(crate) struct ProjectPane {
    pub(crate) skill: String,
    /// The line under the name: what the run does to the skill.
    pub(crate) summary: String,
    /// In pane order: by kind, then by path.
    pub(crate) rows: Vec<ProjectRow>,
    /// Whether the three views exist; a delete only lists the projects.
    pub(crate) views: bool,
}

impl ProjectPane {
    /// The rows of `view` whose path below the root has every word of `filter`, in pane order.
    pub(crate) fn visible(&self, view: PaneView, filter: &str) -> Vec<&ProjectRow> {
        let words: Vec<String> = filter.split_whitespace().map(str::to_lowercase).collect();
        self.rows
            .iter()
            .filter(|row| match (self.views, view) {
                (true, PaneView::Affected) => row.kind != Kind::Same,
                (true, PaneView::Others) => row.kind == Kind::Same,
                _ => true,
            })
            .filter(|row| {
                let path = row.long.to_lowercase();
                words.iter().all(|word| path.contains(word.as_str()))
            })
            .collect()
    }

    /// How many rows each view shows with `filter`: affected, others, all.
    pub(crate) fn counts(&self, filter: &str) -> [usize; 3] {
        [PaneView::Affected, PaneView::Others, PaneView::All]
            .map(|view| self.visible(view, filter).len())
    }
}

/// A project as the pane writes it: its path below the scan root.
fn below_root(project: &Path, root: &Path) -> String {
    // The walker yields projects below the root, so this only ever cuts; a path outside would be shown whole.
    project
        .strip_prefix(root)
        .unwrap_or(project)
        .to_string_lossy()
        .into_owned()
}

fn project_row(kind: Kind, long: String, text: String) -> ProjectRow {
    ProjectRow {
        kind,
        short: short_path(&long),
        long,
        text,
    }
}

/// The pane of a sync or push row.
pub(crate) fn change_pane(skill: &str, states: &[&TargetState], root: &Path) -> ProjectPane {
    let mut rows: Vec<ProjectRow> = states
        .iter()
        .map(|state| {
            let InProject { project, text, .. } = in_project(state);
            let long = below_root(project, root);
            match state.state {
                State::Create(_) | State::Update(_) => project_row(Kind::Changing, long, text),
                State::Failed(_) => project_row(Kind::Failing, long, text),
                // "up to date" is what the Others view says; the line is only the project.
                State::Same => project_row(Kind::Same, long, String::new()),
            }
        })
        .collect();
    rows.sort();
    let count = |kind: Kind| rows.iter().filter(|r| r.kind == kind).count();
    let (changing, failing) = (count(Kind::Changing), count(Kind::Failing));
    let what = match (changing, failing) {
        (0, 0) => "all up to date".to_string(),
        (n, 0) => format!("{n} to update"),
        (0, f) => format!("{f} cannot be applied"),
        (n, f) => format!("{n} to update, {f} cannot be applied"),
    };
    ProjectPane {
        skill: skill.to_string(),
        summary: format!("{}, {what}", plural(rows.len(), "project")),
        rows,
        views: true,
    }
}

/// The pane of a delete row: every project the skill would be removed from.
pub(crate) fn delete_pane(skill: &str, installed: &[&Installed], root: &Path) -> ProjectPane {
    let mut rows: Vec<ProjectRow> = installed
        .iter()
        .map(|row| {
            let text = if row.in_vault == Some(false) {
                "not in the vault"
            } else {
                ""
            };
            project_row(
                Kind::Changing,
                below_root(&row.target.project, root),
                text.to_string(),
            )
        })
        .collect();
    rows.sort();
    ProjectPane {
        skill: skill.to_string(),
        summary: format!("Would be removed from {}", plural(rows.len(), "project")),
        rows,
        views: false,
    }
}
