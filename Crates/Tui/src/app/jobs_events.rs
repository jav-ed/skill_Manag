//! What happens when a background job reports.

use std::sync::Arc;

use super::{App, Screen};
use crate::event::{Job, JobId};
use crate::jobs;
use crate::preview::Preview;
use crate::results::Kind;
use crate::screens::{History, HistoryPhase, Pending, Phase, Work};

impl App {
    /// A report counts only while the screen still waits for that job. Anything else is the late word of
    /// a job the user has left behind, and it must not write into whatever page is open now.
    pub(super) fn on_job(&mut self, id: JobId, job: Job) {
        match job {
            Job::Loaded(result) if self.loading == Some(id) => self.on_loaded(*result),
            Job::Checked(result) if self.checking == Some(id) => {
                self.checking = None;
                if let Screen::Setup(setup) = &mut self.screen {
                    setup.checked(*result);
                }
            }
            Job::Planned(pending) if self.running == Some(id) => self.on_planned(*pending),
            Job::Runs(result) if self.listing == Some(id) => {
                self.listing = None;
                if let Some(history) = self.history_mut() {
                    match *result {
                        Ok(runs) => history.loaded(runs),
                        Err(message) => history.phase = HistoryPhase::Failed(message),
                    }
                }
            }
            Job::UndoPlanned(result) if self.running == Some(id) => {
                if let Some(history) = self.history_mut() {
                    history.phase = match *result {
                        // Nothing that could be put right: the page says why, and there is nothing to ask.
                        Ok(view) if view.actionable() == 0 => HistoryPhase::Done(Box::new(view)),
                        Ok(view) => HistoryPhase::Confirm(Box::new(view)),
                        Err(message) => HistoryPhase::Failed(message),
                    };
                }
            }
            Job::Undone(result) if self.running == Some(id) => {
                self.job_ended();
                if let Some(history) = self.history_mut() {
                    history.scroll = 0;
                    history.phase = match *result {
                        Ok(view) => HistoryPhase::Done(Box::new(view)),
                        Err(message) => HistoryPhase::Failed(message),
                    };
                }
            }
            Job::Progress { done, total } if self.running == Some(id) => {
                if let Some(work) = self.work_mut()
                    && let Phase::Running { kind, .. } = work.phase
                {
                    work.phase = Phase::Running { kind, done, total };
                }
                if let Some(history) = self.history_mut()
                    && matches!(history.phase, HistoryPhase::Running { .. })
                {
                    history.phase = HistoryPhase::Running { done, total };
                }
            }
            Job::Finished(results) if self.running == Some(id) => {
                self.job_ended();
                if let Some(work) = self.work_mut() {
                    work.scroll = 0;
                    work.phase = Phase::Done(results);
                }
            }
            Job::Failed(message) if self.running == Some(id) => {
                self.job_ended();
                if let Some(work) = self.work_mut() {
                    work.phase = Phase::Failed(message);
                } else if let Some(history) = self.history_mut() {
                    history.phase = HistoryPhase::Failed(message);
                }
            }
            _ => {}
        }
    }

    /// The disk changed or the job failed, so the scan is stale and the next page needs a new one.
    fn job_ended(&mut self) {
        self.running = None;
        self.quit_warned = false;
        self.session = None;
    }

    fn on_loaded(&mut self, result: Result<crate::session::Session, String>) {
        self.loading = None;
        match result {
            Ok(session) => {
                let session = Arc::new(session);
                self.session = Some(Arc::clone(&session));
                if let Some(work) = self.work_mut()
                    && matches!(work.phase, Phase::Loading)
                {
                    work.populate(&session);
                }
            }
            Err(message) => {
                if let Some(work) = self.work_mut()
                    && matches!(work.phase, Phase::Loading)
                {
                    work.phase = Phase::Failed(message);
                }
            }
        }
    }

    /// Starts working out what the selection would do. Nothing is written until the user has seen it.
    pub(super) fn plan(&mut self, pending: Pending) {
        let Some(session) = self.session.clone() else {
            return;
        };
        let id = self.new_job();
        self.running = Some(id);
        if let Some(work) = self.work_mut() {
            work.phase = Phase::Planning(pending.kind);
        }
        jobs::spawn_plan(self.tx.clone(), id, session, pending);
    }

    /// The plan is ready: a run that would write something is shown first, one that would not goes on.
    fn on_planned(&mut self, pending: Pending) {
        let writes = pending.preview.as_ref().is_some_and(Preview::writes);
        if !writes {
            self.start(pending);
        } else if let Some(work) = self.work_mut() {
            work.phase = Phase::Confirm(pending);
        }
    }

    pub(super) fn start(&mut self, pending: Pending) {
        let id = self.new_job();
        let total = pending.targets.len();
        let kind = pending.kind;
        let Some(work) = self.work_mut() else {
            return;
        };
        work.phase = Phase::Running {
            kind,
            done: 0,
            total,
        };
        self.running = Some(id);
        let tx = self.tx.clone();
        let dirs = self.dirs.clone();
        match pending.plan {
            Some(plan) if kind != Kind::Delete => jobs::spawn_apply(tx, id, dirs, kind, plan),
            _ if kind == Kind::Delete => jobs::spawn_delete(tx, id, dirs, pending),
            _ => {
                self.running = None;
                if let Some(work) = self.work_mut() {
                    work.phase = Phase::Failed("internal error: no plan for the run".to_string());
                }
            }
        }
    }

    pub(super) fn work_mut(&mut self) -> Option<&mut Work> {
        match &mut self.screen {
            Screen::Work(work) => Some(work),
            Screen::Menu(_) | Screen::Setup(_) | Screen::History(_) => None,
        }
    }

    fn history_mut(&mut self) -> Option<&mut History> {
        match &mut self.screen {
            Screen::History(history) => Some(history),
            Screen::Menu(_) | Screen::Setup(_) | Screen::Work(_) => None,
        }
    }

    /// Checks what undoing a run would do. Nothing is changed until the user has seen it.
    pub(super) fn plan_undo(&mut self, run: String) {
        let id = self.new_job();
        self.running = Some(id);
        if let Some(history) = self.history_mut() {
            history.phase = HistoryPhase::Planning;
        }
        jobs::spawn_undo_plan(self.tx.clone(), id, self.dirs.clone(), run);
    }

    /// The user said yes: bring the run back.
    pub(super) fn start_undo(&mut self, run: String) {
        let id = self.new_job();
        let Some(history) = self.history_mut() else {
            return;
        };
        let HistoryPhase::Confirm(view) = &history.phase else {
            return;
        };
        let total = view.lines.len();
        history.phase = HistoryPhase::Running { done: 0, total };
        self.running = Some(id);
        jobs::spawn_undo(self.tx.clone(), id, self.dirs.clone(), run, total);
    }
}
