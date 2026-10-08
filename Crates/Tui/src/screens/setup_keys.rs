//! Keys and mouse of the setup wizard.

use super::setup::{Setup, SetupAction, Step};
use crate::binding::{ALL, BACK, CANCEL, CONFIRM, DOWN, PAGE_DOWN, PAGE_UP, TOGGLE, UP, YES};
use crate::hit::Target;
use crate::input::{Button, Code, Key, KeyKind, Mouse, MouseKind};

impl Setup {
    pub(crate) fn on_key(&mut self, key: Key) -> SetupAction {
        if key.kind == KeyKind::Release {
            return SetupAction::None;
        }
        self.error = None;
        if BACK.matches(key) {
            return SetupAction::Leave;
        }
        if key.code == Code::Esc {
            return self.back();
        }
        match self.step {
            Step::Vault | Step::Root => {
                let outcome = self.picker.on_key(key);
                self.picked(outcome);
                SetupAction::None
            }
            Step::Mandatory => {
                self.mandatory_key(key);
                SetupAction::None
            }
            Step::Save => self.save_key(key),
            Step::Saved(_) => {
                if CONFIRM.matches(key) {
                    SetupAction::Leave
                } else {
                    SetupAction::None
                }
            }
        }
    }

    fn mandatory_key(&mut self, key: Key) {
        if UP.matches(key) {
            self.list.move_by(-1);
        } else if DOWN.matches(key) {
            self.list.move_by(1);
        } else if PAGE_UP.matches(key) {
            self.list.page(-1);
        } else if PAGE_DOWN.matches(key) {
            self.list.page(1);
        } else if TOGGLE.matches(key) {
            self.toggle(self.list.cursor);
        } else if ALL.matches(key) {
            self.toggle_all();
        } else if CONFIRM.matches(key) {
            self.step = Step::Save;
        }
    }

    fn save_key(&mut self, key: Key) -> SetupAction {
        if YES.matches(key) {
            return self.request().map_or(SetupAction::None, SetupAction::Save);
        }
        if CANCEL.matches(key) {
            return SetupAction::Leave;
        }
        SetupAction::None
    }

    pub(crate) fn on_mouse(&mut self, mouse: Mouse, target: Option<Target>) -> SetupAction {
        let left = matches!(mouse.kind, MouseKind::Down(Button::Left));
        match self.step {
            Step::Vault | Step::Root => {
                let outcome = self.picker.on_mouse(mouse, target);
                self.picked(outcome);
            }
            Step::Mandatory => match (mouse.kind, target) {
                (MouseKind::Moved, Some(Target::CheckRow(i))) => self.list.cursor = i,
                (MouseKind::Down(Button::Left), Some(Target::CheckRow(i))) => {
                    self.list.cursor = i;
                    self.toggle(i);
                }
                (MouseKind::ScrollDown, _) => self.list.scroll_by(3),
                (MouseKind::ScrollUp, _) => self.list.scroll_by(-3),
                _ => {}
            },
            Step::Save if left => match target {
                Some(Target::Button(0)) => {
                    return self.request().map_or(SetupAction::None, SetupAction::Save);
                }
                Some(Target::Button(_)) => return SetupAction::Leave,
                _ => {}
            },
            Step::Saved(_) if left && matches!(target, Some(Target::Button(_))) => {
                return SetupAction::Leave;
            }
            _ => {}
        }
        SetupAction::None
    }
}
