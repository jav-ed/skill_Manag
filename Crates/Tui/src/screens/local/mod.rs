//! The Override page: the skills of the project around the folder skillmirror was started in, against the
//! vault, and replacing the vault's copy of the ones changed there. Nothing is read before the page is
//! opened, and nothing is written before the user has seen what would be.

mod keys;

use std::collections::HashSet;
use std::path::PathBuf;

use skillmirror_core::ops::{Found, LocalSkill, LocalState, PromotePlan};

use super::list_view::ListView;

/// The compare of one project, as the job reports it.
pub(crate) struct Compared {
    pub(crate) project: PathBuf,
    /// Where the project was found; `None` when the user picked it from several.
    pub(crate) found: Option<Found>,
    pub(crate) skills: Vec<LocalSkill>,
    /// What the listing of the skills could not read.
    pub(crate) problems: usize,
}

/// What looking around the folder found.
pub(crate) enum Located {
    One(Compared),
    /// Several projects are below the folder; the user picks one.
    Several(Vec<PathBuf>),
}

/// What the user is asked to confirm.
pub(crate) struct Pending {
    pub(crate) plan: PromotePlan,
    /// When the other projects are updated afterwards: how many of them have one of the skills.
    pub(crate) spread: Option<usize>,
}

/// What one skill came to.
pub(crate) struct Outcome {
    pub(crate) name: String,
    /// The number of files now in the vault, or why the skill was not replaced.
    pub(crate) result: Result<usize, String>,
    /// Files git did not take.
    pub(crate) left_out: usize,
}

/// What updating the other projects came to.
pub(crate) struct Spread {
    pub(crate) projects: usize,
    pub(crate) written: usize,
    pub(crate) failed: Vec<String>,
}

pub(crate) struct Done {
    pub(crate) outcomes: Vec<Outcome>,
    /// The backup run that kept the vault's old folders, when it kept any.
    pub(crate) backup: Option<String>,
    pub(crate) spread: Option<Result<Spread, String>>,
}

pub(crate) enum Phase {
    /// Looking for the project and comparing its skills.
    Loading,
    Failed(String),
    /// Several projects below the folder: pick one.
    Pick,
    List,
    /// Reading the project again and checking the selection against the vault.
    Planning,
    Confirm(Box<Pending>),
    Running {
        done: usize,
        total: usize,
    },
    Done(Box<Done>),
}

/// What the app must do after a key or a click.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Action {
    None,
    Back,
    /// Compare this project.
    Load(PathBuf),
    /// Read the project again and check these skills, then ask. `spread` updates the other projects too.
    Plan {
        names: Vec<String>,
        spread: bool,
    },
    /// The user said yes.
    Run,
}

pub(crate) struct Local {
    pub(crate) phase: Phase,
    pub(crate) project: Option<PathBuf>,
    pub(crate) found: Option<Found>,
    pub(crate) skills: Vec<LocalSkill>,
    pub(crate) problems: usize,
    /// The projects to pick from.
    pub(crate) projects: Vec<PathBuf>,
    pub(crate) view: ListView,
    /// Ticked rows: indexes into `skills`, edited skills only.
    pub(crate) selected: HashSet<usize>,
    /// Update the other projects after the vault has the new copies.
    pub(crate) spread: bool,
    /// First visible line of the results page.
    pub(crate) scroll: usize,
}

impl Local {
    pub(crate) fn new() -> Self {
        Self {
            phase: Phase::Loading,
            project: None,
            found: None,
            skills: Vec::new(),
            problems: 0,
            projects: Vec::new(),
            view: ListView::default(),
            selected: HashSet::new(),
            spread: false,
            scroll: 0,
        }
    }

    /// The job found one project, or several.
    pub(crate) fn located(&mut self, located: Located) {
        match located {
            Located::One(compared) => self.compared(compared),
            Located::Several(projects) => {
                self.view
                    .set((0..projects.len()).map(|i| (i, Vec::new())).collect());
                self.projects = projects;
                self.phase = Phase::Pick;
            }
        }
    }

    fn compared(&mut self, compared: Compared) {
        self.view.set(
            (0..compared.skills.len())
                .map(|i| (i, Vec::new()))
                .collect(),
        );
        // Start on the first skill that can be replaced: that is what the page is for.
        self.view.cursor = compared
            .skills
            .iter()
            .position(|s| matches!(s.state, LocalState::Edited(_)))
            .unwrap_or(0);
        self.view.ensure_visible();
        self.project = Some(compared.project);
        self.found = compared.found;
        self.skills = compared.skills;
        self.problems = compared.problems;
        self.selected.clear();
        self.phase = Phase::List;
    }

    pub(crate) fn edited(&self, index: usize) -> bool {
        self.skills
            .get(index)
            .is_some_and(|s| matches!(s.state, LocalState::Edited(_)))
    }

    /// The names the next step acts on: the ticked skills, or the one under the cursor when none is ticked.
    pub(crate) fn chosen(&self) -> Vec<String> {
        let mut rows: Vec<usize> = self.selected.iter().copied().collect();
        if rows.is_empty()
            && let Some(row) = self.view.current().filter(|row| self.edited(*row))
        {
            rows.push(row);
        }
        rows.sort_unstable();
        rows.iter()
            .filter_map(|i| self.skills.get(*i))
            .map(|s| s.name.clone())
            .collect()
    }

    pub(crate) fn toggle(&mut self, row: usize) {
        if self.edited(row) && !self.selected.remove(&row) {
            self.selected.insert(row);
        }
    }

    /// Ticks every edited skill, or clears them when all of them are ticked already.
    pub(crate) fn toggle_all(&mut self) {
        let edited: Vec<usize> = (0..self.skills.len()).filter(|i| self.edited(*i)).collect();
        if edited.iter().all(|i| self.selected.contains(i)) {
            self.selected.clear();
        } else {
            self.selected.extend(edited);
        }
    }

    /// How many skills can be replaced.
    pub(crate) fn edited_count(&self) -> usize {
        (0..self.skills.len()).filter(|i| self.edited(*i)).count()
    }
}
