//! State of the selection screens (sync, push, delete, list): phases, selection and filter.

use std::collections::HashSet;
use std::path::PathBuf;

use skillmirror_core::plan::Plan;
use skillmirror_core::scan::Target;
use tui_input::Input as TextInput;

use super::list_view::ListView;
use crate::diffview::DiffLine;
use crate::filter::Fuzzy;
use crate::items::{self, Item, Mode, Project};
use crate::preview::Preview;
use crate::results::{Kind, Results};
use crate::session::{Issue, Session};

/// Where add and init write, and what init makes first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Install {
    pub(crate) project: PathBuf,
    /// The project folder does not exist yet (init): it is made right before the first write.
    pub(crate) create: bool,
    /// Make the new project a git repository.
    pub(crate) git: bool,
}

/// A run waiting for a go-ahead or already started.
#[derive(Debug)]
pub(crate) struct Pending {
    pub(crate) kind: Kind,
    pub(crate) targets: Vec<Target>,
    /// How many skills (rows) the targets belong to.
    pub(crate) skills: usize,
    /// What a sync or push will write. The run applies exactly this plan, so a folder edited after it
    /// was made fails instead of being overwritten. A delete has none.
    pub(crate) plan: Option<Plan>,
    /// The numbers and names the confirmation page shows, worked out with the plan so that drawing
    /// never has to look at the disk.
    pub(crate) preview: Option<Preview>,
    /// Set for add and init.
    pub(crate) install: Option<Install>,
}

// The plan is a large value without equality; it only matters whether there is one.
impl PartialEq for Pending {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.targets == other.targets
            && self.skills == other.skills
            && self.install == other.install
            && self.plan.is_some() == other.plan.is_some()
    }
}

impl Eq for Pending {}

pub(crate) enum Phase {
    Loading,
    Failed(String),
    Select,
    /// Working out what the selected sync or push would write.
    Planning(Kind),
    Confirm(Pending),
    /// Reading the files to show what the plan would change.
    Diffing,
    /// The changes of the plan the question is about.
    Diff(Box<DiffPage>),
    Running {
        kind: Kind,
        done: usize,
        total: usize,
    },
    Done(Box<Results>),
}

/// The plan of a question, and its changes as text.
pub(crate) struct DiffPage {
    pub(crate) pending: Pending,
    pub(crate) lines: Vec<DiffLine>,
    /// First visible line.
    pub(crate) scroll: usize,
}

/// What the app must do after a key or a click.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Action {
    None,
    Back,
    /// Read what the plan of the question would change.
    Diff(Pending),
    /// Work out what the selection would do, then ask.
    Plan(Pending),
    Run(Pending),
}

pub(crate) struct Work {
    pub(crate) mode: Mode,
    pub(crate) phase: Phase,
    pub(crate) items: Vec<Item>,
    pub(crate) view: ListView,
    pub(crate) selected: HashSet<usize>,
    pub(crate) filter: TextInput,
    pub(crate) filtering: bool,
    /// First visible line of the results page.
    pub(crate) scroll: usize,
    /// Results page: one line per project instead of only the failures.
    pub(crate) details: bool,
    /// What the scan could not read, and whether the list of it is open.
    pub(crate) issues: Vec<Issue>,
    pub(crate) issues_open: bool,
    /// The project that add and init write to.
    pub(crate) project: Option<Project>,
    pub(super) fuzzy: Fuzzy,
}

impl Work {
    pub(crate) fn new(mode: Mode) -> Self {
        Self {
            mode,
            phase: Phase::Loading,
            items: Vec::new(),
            view: ListView::default(),
            selected: HashSet::new(),
            filter: TextInput::default(),
            filtering: false,
            scroll: 0,
            details: false,
            issues: Vec::new(),
            issues_open: false,
            project: None,
            fuzzy: Fuzzy::default(),
        }
    }

    /// Builds the rows from a finished scan.
    pub(crate) fn populate(&mut self, session: &Session) {
        self.issues.clone_from(&session.issues);
        let built = match (self.mode, &self.project) {
            (Mode::Add | Mode::Init, Some(project)) => {
                Ok(items::for_install(self.mode, session, project))
            }
            _ => items::build(self.mode, session),
        };
        match built {
            Ok(items) => {
                self.selected = items
                    .iter()
                    .enumerate()
                    .filter(|(_, item)| item.preselected)
                    .map(|(i, _)| i)
                    .collect();
                self.items = items;
                self.phase = Phase::Select;
                self.refilter();
            }
            Err(message) => self.phase = Phase::Failed(message),
        }
    }

    pub(crate) fn refilter(&mut self) {
        let hits = self.fuzzy.run(
            self.filter.value(),
            self.items.iter().map(|i| i.name.as_str()),
        );
        self.view.set(hits);
    }

    pub(crate) fn toggle(&mut self, item: usize) {
        // The skills page only looks; there is nothing to select.
        if self.mode == Mode::Skills {
            return;
        }
        if !self.selected.remove(&item) {
            self.selected.insert(item);
        }
    }

    /// Selects every visible row, or clears them when all of them are already selected.
    pub(crate) fn toggle_all(&mut self) {
        if self.mode == Mode::Skills {
            return;
        }
        let all_on = self
            .view
            .filtered
            .iter()
            .all(|(item, _)| self.selected.contains(item));
        for (item, _) in &self.view.filtered {
            if all_on {
                self.selected.remove(item);
            } else {
                self.selected.insert(*item);
            }
        }
    }

    /// The targets of every selected row that is visible, in row order. A row the filter hides stays
    /// selected for when the filter is cleared, but no key acts on it: the page never lets a delete
    /// reach a skill the user cannot see.
    pub(crate) fn pending(&self, kind: Kind) -> Option<Pending> {
        let mut chosen: Vec<usize> = self
            .view
            .filtered
            .iter()
            .map(|(item, _)| *item)
            .filter(|item| self.selected.contains(item))
            .collect();
        chosen.sort_unstable();
        let rows: Vec<&Item> = chosen.iter().filter_map(|i| self.items.get(*i)).collect();
        let targets: Vec<Target> = rows
            .iter()
            .flat_map(|r| r.targets.iter().cloned())
            .collect();
        let skills = rows.len();
        let install = self.project.as_ref().map(|project| Install {
            project: project.path.clone(),
            create: self.mode == Mode::Init,
            git: project.git,
        });
        (!targets.is_empty()).then_some(Pending {
            kind,
            targets,
            skills,
            plan: None,
            preview: None,
            install,
        })
    }

    /// Selected rows that the filter currently hides.
    pub(crate) fn hidden_selected(&self) -> usize {
        let shown: HashSet<usize> = self.view.filtered.iter().map(|(item, _)| *item).collect();
        self.selected.iter().filter(|i| !shown.contains(i)).count()
    }

    /// Why the list is empty, in the words of the Go tool.
    pub(crate) fn empty_message(&self) -> &'static str {
        match self.mode {
            Mode::Sync => "No matching skills found in any project.",
            Mode::Push => {
                "No mandatory skills configured in vault config, or no opted-in projects found."
            }
            Mode::Delete | Mode::List => "No skills found in any project.",
            Mode::Add | Mode::Init | Mode::Skills => "The vault has no skills.",
        }
    }
}
