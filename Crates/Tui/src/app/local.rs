//! The Override page as the app drives it: looking for the project, checking the selection, replacing, and
//! what each job reports.

use super::{App, Screen};
use crate::jobs::local::{self, Look};
use crate::screens::local::{Done, Located, Pending};
use crate::screens::{Local, LocalAction, LocalPhase};

impl App {
    /// Opens the page. Nothing is read before this: the look at the project starts now, on a thread.
    pub(crate) fn open_local(&mut self) {
        self.screen = Screen::Local(Box::new(Local::new()));
        self.locate(Look::Around(self.cwd.clone()));
    }

    fn locate(&mut self, look: Look) {
        let id = self.new_job();
        self.comparing = Some(id);
        local::spawn_locate(self.tx.clone(), id, self.settings.clone(), look);
    }

    pub(super) fn local_act(&mut self, action: LocalAction) {
        // One job at a time: nothing starts while another one is writing.
        if self.writing() && !matches!(action, LocalAction::None) {
            return;
        }
        match action {
            LocalAction::None => {}
            LocalAction::Back => {
                // Leaving drops a look or a check that is still on its way.
                self.running = None;
                self.comparing = None;
                self.back_to_menu();
            }
            LocalAction::Load(project) => {
                if let Some(page) = self.local_mut() {
                    page.phase = LocalPhase::Loading;
                }
                self.locate(Look::At(project));
            }
            LocalAction::Plan { names, spread } => self.plan_promote(names, spread),
            LocalAction::Run => self.start_promote(),
        }
    }

    /// Reads the project again and checks the selection. Nothing is written until the user has seen it.
    fn plan_promote(&mut self, names: Vec<String>, spread: bool) {
        let id = self.new_job();
        let settings = self.settings.clone();
        let Some(page) = self.local_mut() else {
            return;
        };
        let Some(project) = page.project.clone() else {
            return;
        };
        page.phase = LocalPhase::Planning;
        self.running = Some(id);
        local::spawn_plan(self.tx.clone(), id, settings, project, names, spread);
    }

    /// The user said yes: replace the vault's copies.
    fn start_promote(&mut self) {
        let id = self.new_job();
        let (settings, dirs, tx) = (self.settings.clone(), self.dirs.clone(), self.tx.clone());
        let Some(page) = self.local_mut() else {
            return;
        };
        let phase = std::mem::replace(&mut page.phase, LocalPhase::List);
        let LocalPhase::Confirm(pending) = phase else {
            page.phase = phase;
            return;
        };
        let Pending { plan, spread } = *pending;
        page.phase = LocalPhase::Running {
            done: 0,
            total: plan.skills.len(),
        };
        self.running = Some(id);
        local::spawn_promote(tx, id, settings, dirs, plan, spread.is_some());
    }

    pub(super) fn on_located(&mut self, result: Result<Located, String>) {
        self.comparing = None;
        if let Some(page) = self.local_mut() {
            match result {
                Ok(located) => page.located(located),
                Err(message) => page.phase = LocalPhase::Failed(message),
            }
        }
    }

    pub(super) fn on_promote_planned(&mut self, result: Result<Pending, String>) {
        self.running = None;
        if let Some(page) = self.local_mut() {
            page.phase = match result {
                Ok(pending) => LocalPhase::Confirm(Box::new(pending)),
                Err(message) => LocalPhase::Failed(message),
            };
        }
    }

    pub(super) fn on_promoted(&mut self, done: Done) {
        self.job_ended();
        if let Some(page) = self.local_mut() {
            page.scroll = 0;
            page.phase = LocalPhase::Done(Box::new(done));
        }
    }

    pub(super) fn local_progress(&mut self, done: usize, total: usize) {
        if let Some(page) = self.local_mut()
            && matches!(page.phase, LocalPhase::Running { .. })
        {
            page.phase = LocalPhase::Running { done, total };
        }
    }

    pub(super) fn local_mut(&mut self) -> Option<&mut Local> {
        match &mut self.screen {
            Screen::Local(page) => Some(page),
            Screen::Menu(_)
            | Screen::Setup(_)
            | Screen::Work(_)
            | Screen::History(_)
            | Screen::Place(_) => None,
        }
    }
}
