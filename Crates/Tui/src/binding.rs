//! Key bindings. One table drives both the dispatch and the help text, so they cannot drift apart.

use crate::input::{Code, Key, KeyKind, Mods};

pub(crate) struct Binding {
    pub(crate) keys: &'static [(Code, Mods)],
    /// How the key is written in the help.
    pub(crate) label: &'static str,
    pub(crate) help: &'static str,
}

impl Binding {
    pub(crate) fn matches(&self, key: Key) -> bool {
        // Terminals with the kitty protocol also report releases.
        if key.kind == KeyKind::Release {
            return false;
        }
        self.keys
            .iter()
            .any(|(code, mods)| *code == key.code && *mods == key.mods)
    }
}

const N: Mods = Mods::NONE;

pub(crate) const UP: Binding = Binding {
    keys: &[(Code::Up, N), (Code::Char('k'), N)],
    label: "↑/k",
    help: "up",
};
pub(crate) const DOWN: Binding = Binding {
    keys: &[(Code::Down, N), (Code::Char('j'), N)],
    label: "↓/j",
    help: "down",
};
pub(crate) const PAGE_UP: Binding = Binding {
    keys: &[(Code::PageUp, N)],
    label: "pgup",
    help: "page up",
};
pub(crate) const PAGE_DOWN: Binding = Binding {
    keys: &[(Code::PageDown, N)],
    label: "pgdn",
    help: "page down",
};
pub(crate) const TOGGLE: Binding = Binding {
    keys: &[(Code::Char(' '), N)],
    label: "space",
    help: "toggle",
};
pub(crate) const ALL: Binding = Binding {
    keys: &[(Code::Char('a'), N)],
    label: "a",
    help: "all",
};
pub(crate) const FILTER: Binding = Binding {
    keys: &[(Code::Char('/'), N)],
    label: "/",
    help: "filter",
};
pub(crate) const CONFIRM: Binding = Binding {
    keys: &[(Code::Enter, N)],
    label: "enter",
    help: "confirm",
};
pub(crate) const SYNC: Binding = Binding {
    keys: &[(Code::Char('s'), N)],
    label: "s",
    help: "sync",
};
pub(crate) const DELETE: Binding = Binding {
    keys: &[(Code::Char('d'), N)],
    label: "d",
    help: "delete",
};
pub(crate) const UNDO: Binding = Binding {
    keys: &[(Code::Char('u'), N)],
    label: "u",
    help: "undo",
};
pub(crate) const DETAILS: Binding = Binding {
    keys: &[(Code::Char('d'), N)],
    label: "d",
    help: "details",
};
pub(crate) const ISSUES: Binding = Binding {
    keys: &[(Code::Char('i'), N)],
    label: "i",
    help: "problems",
};
pub(crate) const HELP: Binding = Binding {
    keys: &[(Code::Char('?'), N), (Code::Char('?'), Mods::SHIFT)],
    label: "?",
    help: "help",
};
pub(crate) const BACK: Binding = Binding {
    keys: &[(Code::Char('q'), N), (Code::Left, Mods::ALT)],
    label: "q/alt+←",
    help: "back",
};
pub(crate) const CANCEL: Binding = Binding {
    keys: &[(Code::Esc, N), (Code::Char('n'), N), (Code::Char('N'), N)],
    label: "n/esc",
    help: "cancel",
};
pub(crate) const YES: Binding = Binding {
    keys: &[(Code::Char('y'), N), (Code::Char('Y'), N), (Code::Enter, N)],
    label: "y/enter",
    help: "confirm",
};
pub(crate) const OPEN: Binding = Binding {
    keys: &[(Code::Enter, N)],
    label: "enter",
    help: "open",
};
pub(crate) const MENU_QUIT: Binding = Binding {
    keys: &[(Code::Char('q'), N)],
    label: "q",
    help: "quit",
};
pub(crate) const RUN_SYNC: Binding = Binding {
    keys: &[(Code::Enter, N)],
    label: "enter",
    help: "sync",
};
pub(crate) const RUN_PUSH: Binding = Binding {
    keys: &[(Code::Enter, N)],
    label: "enter",
    help: "push",
};
pub(crate) const RUN_DELETE: Binding = Binding {
    keys: &[(Code::Enter, N)],
    label: "enter",
    help: "delete",
};
pub(crate) const SCROLL: Binding = Binding {
    keys: &[(Code::Up, N), (Code::Down, N)],
    label: "↑/↓",
    help: "scroll",
};
pub(crate) const QUIT: Binding = Binding {
    keys: &[(Code::Char('c'), Mods::CTRL)],
    label: "ctrl+c",
    help: "quit",
};

pub(crate) const MENU_HELP: &[&Binding] = &[&UP, &DOWN, &OPEN, &HELP, &MENU_QUIT, &QUIT];
pub(crate) const SELECT_COMMON: &[&Binding] =
    &[&UP, &DOWN, &PAGE_UP, &PAGE_DOWN, &TOGGLE, &ALL, &FILTER];
pub(crate) const RESULTS_HELP: &[&Binding] = &[&SCROLL, &DETAILS, &BACK, &QUIT];
pub(crate) const CONFIRM_HELP: &[&Binding] = &[&YES, &CANCEL, &QUIT];
pub(crate) const BACK_HELP: &[&Binding] = &[&BACK, &QUIT];
