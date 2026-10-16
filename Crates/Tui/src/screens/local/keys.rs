//! Keys and clicks of the Override page.

use ratatui::layout::Position;

use super::{Action, Local, Phase};
use crate::binding::{
    ALL, BACK, CANCEL, CONFIRM, DOWN, PAGE_DOWN, PAGE_UP, SPREAD, TOGGLE, UP, YES,
};
use crate::hit::{HitMap, Target};
use crate::input::{Button, Code, Key, KeyKind, Mouse, MouseKind};

impl Local {
    pub(crate) fn on_key(&mut self, key: Key) -> Action {
        if key.kind == KeyKind::Release {
            return Action::None;
        }
        match &self.phase {
            Phase::Loading | Phase::Failed(_) | Phase::Planning => back_only(key),
            Phase::Pick => self.pick_key(key),
            Phase::List => self.list_key(key),
            Phase::Confirm(_) => self.confirm_key(key),
            // Nothing leaves a job that writes: a double tap must not.
            Phase::Running { .. } => Action::None,
            Phase::Done(_) => self.done_key(key),
        }
    }

    fn move_cursor(&mut self, key: Key) {
        if UP.matches(key) {
            self.view.move_by(-1);
        } else if DOWN.matches(key) {
            self.view.move_by(1);
        } else if PAGE_UP.matches(key) {
            self.view.page(-1);
        } else if PAGE_DOWN.matches(key) {
            self.view.page(1);
        }
    }

    fn pick_key(&mut self, key: Key) -> Action {
        if BACK.matches(key) || key.code == Code::Esc {
            return Action::Back;
        }
        if CONFIRM.matches(key) {
            return self
                .view
                .current()
                .and_then(|i| self.projects.get(i))
                .map_or(Action::None, |p| Action::Load(p.clone()));
        }
        self.move_cursor(key);
        Action::None
    }

    fn list_key(&mut self, key: Key) -> Action {
        if BACK.matches(key) || key.code == Code::Esc {
            return Action::Back;
        }
        if TOGGLE.matches(key) {
            if let Some(row) = self.view.current() {
                self.toggle(row);
            }
        } else if ALL.matches(key) {
            self.toggle_all();
        } else if SPREAD.matches(key) {
            self.spread = !self.spread;
        } else if CONFIRM.matches(key) {
            return self.plan();
        } else {
            self.move_cursor(key);
        }
        Action::None
    }

    fn plan(&self) -> Action {
        let names = self.chosen();
        if names.is_empty() {
            return Action::None;
        }
        Action::Plan {
            names,
            spread: self.spread,
        }
    }

    fn confirm_key(&mut self, key: Key) -> Action {
        if YES.matches(key) {
            return Action::Run;
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
            Phase::Pick | Phase::List => self.list_mouse(mouse, target, hits),
            Phase::Confirm(_) => {
                if !matches!(mouse.kind, MouseKind::Down(Button::Left)) {
                    return Action::None;
                }
                match target {
                    Some(Target::Button(0)) => Action::Run,
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
        let picking = matches!(self.phase, Phase::Pick);
        match (mouse.kind, target) {
            (MouseKind::Moved, Some(Target::Row(row))) => self.view.cursor = row,
            // A click on a project to pick opens it; a click on a skill ticks it when it can be replaced.
            (MouseKind::Down(Button::Left), Some(Target::Row(row))) => {
                self.view.cursor = row;
                if picking {
                    return self
                        .view
                        .current()
                        .and_then(|i| self.projects.get(i))
                        .map_or(Action::None, |p| Action::Load(p.clone()));
                }
                if let Some(index) = self.view.current() {
                    self.toggle(index);
                }
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
