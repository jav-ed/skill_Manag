//! Keys and mouse of the selection screens.

use ratatui::layout::Position;

use super::work::{Action, Phase, Work};
use crate::binding::{
    ALL, BACK, CANCEL, CONFIRM, DELETE, DOWN, FILTER, GIT, ISSUES, PAGE_DOWN, PAGE_UP, SYNC,
    TOGGLE, UP, YES,
};
use crate::hit::{HitMap, Target};
use crate::input::{self, Button, Code, Key, KeyKind, Mouse, MouseKind};
use crate::items::Mode;
use crate::results::Kind;

impl Work {
    pub(crate) fn on_key(&mut self, key: Key) -> Action {
        if key.kind == KeyKind::Release {
            return Action::None;
        }
        // The list of scan problems takes the next key, whatever it is.
        if self.issues_open {
            self.issues_open = false;
            return Action::None;
        }
        match self.phase {
            Phase::Loading | Phase::Failed(_) => back_only(key),
            Phase::Planning(_) if BACK.matches(key) || key.code == Code::Esc => Action::Back,
            Phase::Select if self.filtering => self.filter_key(key),
            Phase::Select => self.select_key(key),
            Phase::Confirm(_) => self.confirm_key(key),
            // Nothing else leaves a plan being made or a job that writes: a double tap must not.
            Phase::Planning(_) | Phase::Running { .. } => Action::None,
            Phase::Done(_) => self.done_key(key),
        }
    }

    fn select_key(&mut self, key: Key) -> Action {
        if BACK.matches(key) {
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
        } else if TOGGLE.matches(key) {
            self.toggle_current();
        } else if ALL.matches(key) {
            self.toggle_all();
        } else if FILTER.matches(key) {
            self.filtering = true;
        } else if ISSUES.matches(key) && !self.issues.is_empty() {
            self.issues_open = true;
        } else if key.code == Code::Esc && !self.filter.value().is_empty() {
            self.filter.reset();
            self.refilter();
        } else if CONFIRM.matches(key) {
            return self.confirm_selection();
        } else if self.mode == Mode::Init && GIT.matches(key) {
            if let Some(project) = &mut self.project {
                project.git = !project.git;
            }
        } else if self.mode == Mode::List && SYNC.matches(key) {
            return self.plan(Kind::Sync);
        } else if self.mode == Mode::List && DELETE.matches(key) {
            return self.ask(Kind::Delete);
        }
        Action::None
    }

    /// Enter: sync and push work out what they would write and then ask, delete asks at once, the list
    /// does nothing.
    fn confirm_selection(&mut self) -> Action {
        match self.mode {
            Mode::Sync => self.plan(Kind::Sync),
            Mode::Push => self.plan(Kind::Push),
            Mode::Delete => self.ask(Kind::Delete),
            Mode::Add => self.plan(Kind::Add),
            Mode::Init => self.plan(Kind::Init),
            Mode::List => Action::None,
        }
    }

    fn plan(&mut self, kind: Kind) -> Action {
        self.pending(kind).map_or(Action::None, Action::Plan)
    }

    fn ask(&mut self, kind: Kind) -> Action {
        if let Some(pending) = self.pending(kind) {
            self.phase = Phase::Confirm(pending);
        }
        Action::None
    }

    fn toggle_current(&mut self) {
        if let Some(item) = self.view.current() {
            self.toggle(item);
        }
    }

    fn filter_key(&mut self, key: Key) -> Action {
        match key.code {
            Code::Esc => {
                self.filter.reset();
                self.filtering = false;
                self.refilter();
            }
            Code::Enter => self.filtering = false,
            _ => {
                if let Some(request) = input::to_request(key)
                    && self.filter.handle(request).is_some()
                {
                    self.refilter();
                }
            }
        }
        Action::None
    }

    fn confirm_key(&mut self, key: Key) -> Action {
        if YES.matches(key) {
            return self.confirmed();
        }
        if CANCEL.matches(key) {
            self.phase = Phase::Select;
        }
        Action::None
    }

    /// The user said yes: the pending run, with its plan, leaves the page.
    fn confirmed(&mut self) -> Action {
        match std::mem::replace(&mut self.phase, Phase::Select) {
            Phase::Confirm(pending) => Action::Run(pending),
            other => {
                self.phase = other;
                Action::None
            }
        }
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
        } else if key.code == Code::Char('d') {
            self.details = !self.details;
            self.scroll = 0;
        } else if BACK.matches(key) || key.code == Code::Esc || CONFIRM.matches(key) {
            return Action::Back;
        }
        Action::None
    }

    pub(crate) fn on_mouse(&mut self, mouse: Mouse, hits: &HitMap) -> Action {
        let target = hits.at(Position::new(mouse.column, mouse.row));
        if self.issues_open {
            if matches!(mouse.kind, MouseKind::Down(Button::Left)) {
                self.issues_open = false;
            }
            return Action::None;
        }
        match self.phase {
            Phase::Select => self.select_mouse(mouse, target, hits),
            Phase::Confirm(_) => self.confirm_mouse(mouse, target),
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

    fn select_mouse(&mut self, mouse: Mouse, target: Option<Target>, hits: &HitMap) -> Action {
        let over_list = matches!(
            target,
            Some(Target::Row(_) | Target::ListArea | Target::ScrollTrack)
        );
        match (mouse.kind, target) {
            (MouseKind::Moved, Some(Target::Row(row))) => self.view.cursor = row,
            (MouseKind::Down(Button::Left), Some(Target::Row(row))) => {
                self.view.cursor = row;
                self.toggle_current();
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

    fn confirm_mouse(&mut self, mouse: Mouse, target: Option<Target>) -> Action {
        if !matches!(mouse.kind, MouseKind::Down(Button::Left)) {
            return Action::None;
        }
        match target {
            Some(Target::Button(0)) => self.confirmed(),
            Some(Target::Button(_) | Target::Dismiss) => {
                self.phase = Phase::Select;
                Action::None
            }
            _ => Action::None,
        }
    }
}

/// Loading and error pages only know how to go back.
fn back_only(key: Key) -> Action {
    if BACK.matches(key) || key.code == Code::Esc || CONFIRM.matches(key) {
        Action::Back
    } else {
        Action::None
    }
}
