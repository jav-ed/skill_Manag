//! What happens when a background job reports.

use std::sync::Arc;

use super::{App, Screen};
use crate::event::Job;
use crate::jobs;
use crate::results::Kind;
use crate::screens::{Pending, Phase, Work};

impl App {
    pub(super) fn on_job(&mut self, job: Job) {
        match job {
            Job::Loaded(result) => self.on_loaded(*result),
            Job::Progress { done, total } => {
                if let Some(work) = self.work_mut()
                    && let Phase::Running { kind, .. } = work.phase
                {
                    work.phase = Phase::Running { kind, done, total };
                }
            }
            Job::Finished(results) => {
                // The disk changed, so the scan is stale.
                self.session = None;
                if let Some(work) = self.work_mut() {
                    work.scroll = 0;
                    work.phase = Phase::Done(results);
                }
            }
            Job::Failed(message) => {
                self.session = None;
                if let Some(work) = self.work_mut() {
                    work.phase = Phase::Failed(message);
                }
            }
        }
    }

    fn on_loaded(&mut self, result: Result<crate::session::Session, String>) {
        self.loading = false;
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

    pub(super) fn start(&mut self, pending: Pending) {
        let Some(session) = self.session.clone() else {
            return;
        };
        let total = pending.targets.len();
        if let Some(work) = self.work_mut() {
            work.phase = Phase::Running {
                kind: pending.kind,
                done: 0,
                total,
            };
        }
        let tx = self.tx.clone();
        match pending.kind {
            Kind::Delete => jobs::spawn_delete(tx, self.dirs.clone(), pending.targets),
            kind => jobs::spawn_apply(tx, session, self.dirs.clone(), kind, pending.targets),
        }
    }

    pub(super) fn work_mut(&mut self) -> Option<&mut Work> {
        match &mut self.screen {
            Screen::Work(work) => Some(work),
            Screen::Menu(_) | Screen::Setup(_) => None,
        }
    }
}
