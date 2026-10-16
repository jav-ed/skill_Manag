//! Keys and mouse of the selection screens.

use super::{Action, Phase, Work};
use crate::binding::{
    AGENTS_FILE, ALL, BACK, CANCEL, CONFIRM, DELETE, DOWN, FILTER, GIT, ISSUES, PAGE_DOWN, PAGE_UP,
    PANE_DOWN, PANE_FILTER, PANE_PATHS, PANE_UP, PANE_VIEW, SCAFFOLD, SYNC, TOGGLE, UP, VIEW, YES,
};
use crate::input::{self, Code, Key, KeyKind};
use crate::items::Mode;
use crate::results::Kind;

/// Rows one press of the pane keys scrolls by.
const SCROLL_STEP: usize = 8;

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
            Phase::Planning(_) | Phase::Diffing if BACK.matches(key) || key.code == Code::Esc => {
                Action::Back
            }
            Phase::Select if self.filtering => self.filter_key(key),
            Phase::Select if self.pane.filtering => self.pane_filter_key(key),
            Phase::Select => self.select_key(key),
            Phase::Confirm(_) => self.confirm_key(key),
            // Nothing else leaves a plan being made or a job that writes: a double tap must not.
            Phase::Planning(_) | Phase::Diffing | Phase::Running { .. } => Action::None,
            Phase::Diff(_) => self.diff_key(key),
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
        } else if self.mode.has_pane() && PANE_DOWN.matches(key) {
            self.pane.scroll = self.pane.scroll.saturating_add(SCROLL_STEP);
        } else if self.mode.has_pane() && PANE_UP.matches(key) {
            self.pane.scroll = self.pane.scroll.saturating_sub(SCROLL_STEP);
        } else if self.mode.has_project_pane() && PANE_PATHS.matches(key) {
            self.pane.paths = self.pane.paths.toggled();
        } else if self.mode.has_project_pane() && PANE_FILTER.matches(key) {
            self.pane.filtering = true;
        } else if self.mode.has_views() && PANE_VIEW.matches(key) {
            self.pane.view = self.pane.view.next();
        } else if ISSUES.matches(key) && !self.issues.is_empty() {
            self.issues_open = true;
        } else if key.code == Code::Esc && !self.filter.value().is_empty() {
            self.filter.reset();
            self.refilter();
        } else if key.code == Code::Esc && !self.pane.filter.value().is_empty() {
            self.pane.filter.reset();
        } else if CONFIRM.matches(key) {
            return self.confirm_selection();
        } else if self.mode == Mode::Init && GIT.matches(key) {
            if let Some(project) = &mut self.project {
                project.git = !project.git;
            }
        } else if self.mode == Mode::Init && AGENTS_FILE.matches(key) {
            if let Some(project) = &mut self.project {
                project.agents = !project.agents;
            }
        } else if self.mode == Mode::Init && SCAFFOLD.matches(key) {
            if let Some(project) = &mut self.project {
                project.scaffold = !project.scaffold;
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
            Mode::Agents => self.plan(Kind::Agents),
            Mode::List | Mode::Skills => Action::None,
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

    pub(super) fn toggle_current(&mut self) {
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

    /// Typing in the filter of the pane: enter keeps it, esc clears it, like the filter of the list.
    fn pane_filter_key(&mut self, key: Key) -> Action {
        match key.code {
            Code::Esc => {
                self.pane.filter.reset();
                self.pane.filtering = false;
            }
            Code::Enter => self.pane.filtering = false,
            _ => {
                if let Some(request) = input::to_request(key) {
                    self.pane.filter.handle(request);
                }
            }
        }
        Action::None
    }

    fn confirm_key(&mut self, key: Key) -> Action {
        if YES.matches(key) {
            return self.confirmed();
        }
        // Only a sync, push, add, init or Agents write has a plan to show; a delete has none.
        if VIEW.matches(key)
            && matches!(&self.phase, Phase::Confirm(p) if p.plan.is_some() || p.agents.is_some())
            && let Phase::Confirm(pending) = std::mem::replace(&mut self.phase, Phase::Diffing)
        {
            return Action::Diff(pending);
        }
        if CANCEL.matches(key) {
            self.phase = Phase::Select;
        }
        Action::None
    }

    /// The user said yes: the pending run, with its plan, leaves the page.
    pub(super) fn confirmed(&mut self) -> Action {
        match std::mem::replace(&mut self.phase, Phase::Select) {
            Phase::Confirm(pending) => Action::Run(pending),
            other => {
                self.phase = other;
                Action::None
            }
        }
    }

    /// The changes page scrolls; any way back returns to the question.
    fn diff_key(&mut self, key: Key) -> Action {
        let Phase::Diff(page) = &mut self.phase else {
            return Action::None;
        };
        if UP.matches(key) {
            page.scroll = page.scroll.saturating_sub(1);
        } else if DOWN.matches(key) {
            page.scroll = page.scroll.saturating_add(1);
        } else if PAGE_UP.matches(key) {
            page.scroll = page.scroll.saturating_sub(10);
        } else if PAGE_DOWN.matches(key) {
            page.scroll = page.scroll.saturating_add(10);
        } else if BACK.matches(key) || key.code == Code::Esc || VIEW.matches(key) {
            self.return_to_question();
        }
        Action::None
    }

    /// From the changes page back to the question, with the same plan.
    pub(crate) fn return_to_question(&mut self) {
        if let Phase::Diff(page) = std::mem::replace(&mut self.phase, Phase::Select) {
            self.phase = Phase::Confirm(page.pending);
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
}

/// Loading and error pages only know how to go back.
fn back_only(key: Key) -> Action {
    if BACK.matches(key) || key.code == Code::Esc || CONFIRM.matches(key) {
        Action::Back
    } else {
        Action::None
    }
}
