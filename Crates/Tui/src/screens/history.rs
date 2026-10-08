//! The history page: the backup runs, and undoing one of them.

use ratatui::layout::Position;

use super::list_view::ListView;
use crate::binding::{BACK, CANCEL, CONFIRM, DOWN, PAGE_DOWN, PAGE_UP, UNDO, UP, YES};
use crate::hit::{HitMap, Target};
use crate::input::{Button, Code, Key, KeyKind, Mouse, MouseKind};
use crate::undo::{RunRow, UndoView};

pub(crate) enum Phase {
    /// Reading the backup store.
    Loading,
    Failed(String),
    List,
    /// Checking what undoing the run would do.
    Planning,
    Confirm(Box<UndoView>),
    Running {
        done: usize,
        total: usize,
    },
    Done(Box<UndoView>),
}

/// What the app must do after a key or a click.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Action {
    None,
    Back,
    /// Work out what undoing this run would do, then ask.
    Plan(String),
    /// The user said yes to this run.
    Run(String),
}

pub(crate) struct History {
    pub(crate) phase: Phase,
    pub(crate) runs: Vec<RunRow>,
    pub(crate) view: ListView,
    /// First visible line of the results page.
    pub(crate) scroll: usize,
}

impl History {
    pub(crate) fn new() -> Self {
        Self {
            phase: Phase::Loading,
            runs: Vec::new(),
            view: ListView::default(),
            scroll: 0,
        }
    }

    /// The store was read.
    pub(crate) fn loaded(&mut self, runs: Vec<RunRow>) {
        self.view
            .set((0..runs.len()).map(|i| (i, Vec::new())).collect());
        self.runs = runs;
        self.phase = Phase::List;
    }

    fn current(&self) -> Option<&RunRow> {
        self.view.current().and_then(|i| self.runs.get(i))
    }

    /// The run under the cursor, when it can be undone.
    fn plan(&self) -> Action {
        match self.current() {
            Some(run) if run.error.is_none() => Action::Plan(run.id.clone()),
            _ => Action::None,
        }
    }

    pub(crate) fn on_key(&mut self, key: Key) -> Action {
        if key.kind == KeyKind::Release {
            return Action::None;
        }
        match &self.phase {
            Phase::Loading | Phase::Failed(_) | Phase::Planning => back_only(key),
            Phase::List => self.list_key(key),
            Phase::Confirm(view) => {
                let run = view.run.clone();
                self.confirm_key(key, run)
            }
            // Nothing leaves a job that writes: a double tap must not.
            Phase::Running { .. } => Action::None,
            Phase::Done(_) => self.done_key(key),
        }
    }

    fn list_key(&mut self, key: Key) -> Action {
        if BACK.matches(key) || key.code == Code::Esc {
            return Action::Back;
        }
        if UP.matches(key) {
            self.view.move_by(-1);
        } else if DOWN.matches(key) {
            self.view.move_by(1);
        } else if PAGE_UP.matches(key) {
            self.view.page(-1);
        } else if PAGE_DOWN.matches(key) {
            self.view.page(1);
        } else if CONFIRM.matches(key) || UNDO.matches(key) {
            return self.plan();
        }
        Action::None
    }

    fn confirm_key(&mut self, key: Key, run: String) -> Action {
        if YES.matches(key) {
            return Action::Run(run);
        }
        if CANCEL.matches(key) {
            self.phase = Phase::List;
        }
        Action::None
    }

    fn done_key(&mut self, key: Key) -> Action {
        if UP.matches(key) {
            self.scroll = self.scroll.saturating_sub(1);
        } else if DOWN.matches(key) {
            self.scroll = self.scroll.saturating_add(1);
        } else if PAGE_UP.matches(key) {
            self.scroll = self.scroll.saturating_sub(10);
        } else if PAGE_DOWN.matches(key) {
            self.scroll = self.scroll.saturating_add(10);
        } else if BACK.matches(key) || key.code == Code::Esc || CONFIRM.matches(key) {
            return Action::Back;
        }
        Action::None
    }

    pub(crate) fn on_mouse(&mut self, mouse: Mouse, hits: &HitMap) -> Action {
        let target = hits.at(Position::new(mouse.column, mouse.row));
        match &self.phase {
            Phase::List => self.list_mouse(mouse, target, hits),
            Phase::Confirm(view) => {
                let run = view.run.clone();
                if !matches!(mouse.kind, MouseKind::Down(Button::Left)) {
                    return Action::None;
                }
                match target {
                    Some(Target::Button(0)) => Action::Run(run),
                    Some(Target::Button(_) | Target::Dismiss) => {
                        self.phase = Phase::List;
                        Action::None
                    }
                    _ => Action::None,
                }
            }
            Phase::Done(_) => {
                match mouse.kind {
                    MouseKind::ScrollUp => self.scroll = self.scroll.saturating_sub(3),
                    MouseKind::ScrollDown => self.scroll = self.scroll.saturating_add(3),
                    _ => {}
                }
                Action::None
            }
            _ => Action::None,
        }
    }

    fn list_mouse(&mut self, mouse: Mouse, target: Option<Target>, hits: &HitMap) -> Action {
        let over_list = matches!(
            target,
            Some(Target::Row(_) | Target::ListArea | Target::ScrollTrack)
        );
        match (mouse.kind, target) {
            (MouseKind::Moved, Some(Target::Row(row))) => self.view.cursor = row,
            // The first click picks a run, a click on the picked run asks about undoing it.
            (MouseKind::Down(Button::Left), Some(Target::Row(row))) => {
                if self.view.cursor == row {
                    return self.plan();
                }
                self.view.cursor = row;
            }
            (MouseKind::Down(Button::Left), Some(Target::ScrollTrack)) => {
                if let Some(track) = hits.rect_of(Target::ScrollTrack) {
                    let from_top = usize::from(mouse.row.saturating_sub(track.y));
                    self.view.jump(from_top, usize::from(track.height));
                }
            }
            (MouseKind::ScrollUp, _) if over_list => self.view.scroll_by(-3),
            (MouseKind::ScrollDown, _) if over_list => self.view.scroll_by(3),
            _ => {}
        }
        Action::None
    }
}

/// Pages that are waiting only know how to go back.
fn back_only(key: Key) -> Action {
    if BACK.matches(key) || key.code == Code::Esc || CONFIRM.matches(key) {
        Action::Back
    } else {
        Action::None
    }
}
