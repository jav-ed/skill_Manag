//! State of the selection screens (sync, push, delete, list): phases, selection and filter.

use std::collections::HashSet;

use skillmirror_core::scan::Target;
use tui_input::Input as TextInput;

use super::list_view::ListView;
use crate::filter::Fuzzy;
use crate::items::{self, Item, Mode};
use crate::results::{Kind, Results};
use crate::session::Session;

/// A run waiting for a go-ahead or already started.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Pending {
    pub(crate) kind: Kind,
    pub(crate) targets: Vec<Target>,
    /// How many skills (rows) the targets belong to.
    pub(crate) skills: usize,
}

pub(crate) enum Phase {
    Loading,
    Failed(String),
    Select,
    Confirm(Pending),
    Running {
        kind: Kind,
        done: usize,
        total: usize,
    },
    Done(Box<Results>),
}

/// What the app must do after a key or a click.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Action {
    None,
    Back,
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
            fuzzy: Fuzzy::default(),
        }
    }

    /// Builds the rows from a finished scan.
    pub(crate) fn populate(&mut self, session: &Session) {
        match items::build(self.mode, session) {
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
        if !self.selected.remove(&item) {
            self.selected.insert(item);
        }
    }

    /// Selects every visible row, or clears them when all of them are already selected.
    pub(crate) fn toggle_all(&mut self) {
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

    /// The targets of every selected row, in row order. Rows hidden by the filter count too.
    pub(crate) fn pending(&self, kind: Kind) -> Option<Pending> {
        let mut chosen: Vec<usize> = self.selected.iter().copied().collect();
        chosen.sort_unstable();
        let rows: Vec<&Item> = chosen.iter().filter_map(|i| self.items.get(*i)).collect();
        let targets: Vec<Target> = rows
            .iter()
            .flat_map(|r| r.targets.iter().cloned())
            .collect();
        let skills = rows.len();
        (!targets.is_empty()).then_some(Pending {
            kind,
            targets,
            skills,
        })
    }

    /// Why the list is empty, in the words of the Go tool.
    pub(crate) fn empty_message(&self) -> &'static str {
        match self.mode {
            Mode::Sync => "No matching skills found in any project.",
            Mode::Push => {
                "No mandatory skills configured in vault config, or no opted-in projects found."
            }
            Mode::Delete | Mode::List => "No skills found in any project.",
        }
    }
}
