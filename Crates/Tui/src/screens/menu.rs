//! The main menu.

use super::place::Purpose;
use crate::binding::{BACK, CONFIRM, DOWN, UP};
use crate::input::Key;
use crate::items::Mode;
use crate::num::step;

pub(crate) struct Entry {
    pub(crate) label: &'static str,
    pub(crate) blurb: &'static str,
    pub(crate) detail: &'static str,
    pub(crate) dest: Dest,
}

/// Where an entry leads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Dest {
    Work(Mode),
    /// Choose a folder, then the skills to install into it.
    Place(Purpose),
    History,
    Setup,
}

pub(crate) const ENTRIES: [Entry; 8] = [
    Entry {
        label: "Sync",
        blurb: "Refresh the skills each project already has; never adds one",
        detail: "Walks every project under your root and updates skills they already have installed, pulling the latest from your vault. The opt-in rule: if a project doesn't have a skill, it will inshallah never be added. Only what's already there gets refreshed.",
        dest: Dest::Work(Mode::Sync),
    },
    Entry {
        label: "List",
        blurb: "Every installed skill; filter, then sync or delete in place",
        detail: "A searchable table of every skill installed across all your projects. Filter by name with /, select rows with space, then sync or delete the selection directly without leaving the screen.",
        dest: Dest::Work(Mode::List),
    },
    Entry {
        label: "Delete",
        blurb: "Nothing pre-selected; you pick, then confirm",
        detail: "Remove skills from projects. Nothing is pre-selected: you pick explicitly, then confirm before anything is removed. Supports removing from all matching projects at once.",
        dest: Dest::Work(Mode::Delete),
    },
    Entry {
        label: "Push",
        blurb: "Install the mandatory skills into every opted-in project",
        detail: "Reads the mandatory list from your vault config and pushes those skills to every project that already has .agents/skills/, bypassing the opt-in rule. Configure mandatory skills by adding 'mandatory: [skill-name]' to <vault>/config.yaml.",
        dest: Dest::Work(Mode::Push),
    },
    Entry {
        label: "Add",
        blurb: "Install chosen vault skills into one project",
        detail: "Pick a project folder, then the skills to install from your vault. The page says what would be written before anything is, and skills the project already has are compared like in a sync. If the vault config lists targets such as claude, the links to .agents/skills are made for the project too.",
        dest: Dest::Place(Purpose::Add),
    },
    Entry {
        label: "Init",
        blurb: "Make a new project folder with your skills in it",
        detail: "Pick the folder the project goes in and type its name, then pick the skills (the mandatory ones are ticked). Press g to make it a git repository. Nothing is made until you confirm, and if no skill could be installed the new folder is taken away again.",
        dest: Dest::Place(Purpose::Init),
    },
    Entry {
        label: "History",
        blurb: "Undo a sync, push or delete from the backups",
        detail: "Every sync, push, add, init and delete that replaces or removes a skill folder keeps the old copy first. This page lists those runs, newest first. Pick one and confirm to put its folders back (a skill the run created goes again). Undoing is a run of its own, so you can undo the undo.",
        dest: Dest::History,
    },
    Entry {
        label: "Setup",
        blurb: "Choose the vault, the scan root and the mandatory skills",
        detail: "Pick your vault (the git folder holding your master skills) and the root (the folder that contains all your projects) in a folder picker, then tick the mandatory skills. The vault is remembered in ~/.config/skillmirror/vault; root and mandatory go into <vault>/config.yaml, and comments and other keys there are kept.",
        dest: Dest::Setup,
    },
];

#[derive(Debug, Default)]
pub(crate) struct Menu {
    pub(crate) cursor: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum MenuAction {
    None,
    Open(usize),
    Quit,
}

impl Menu {
    pub(crate) fn on_key(&mut self, key: Key) -> MenuAction {
        if UP.matches(key) {
            self.cursor = step(self.cursor, -1, ENTRIES.len());
        } else if DOWN.matches(key) {
            self.cursor = step(self.cursor, 1, ENTRIES.len());
        } else if CONFIRM.matches(key) {
            return MenuAction::Open(self.cursor);
        } else if BACK.matches(key) {
            return MenuAction::Quit;
        }
        MenuAction::None
    }
}
