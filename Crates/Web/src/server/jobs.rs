//! The write that is running or has just finished. One at a time: two writers over the same folders
//! would only get in each other's way.

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde::Serialize;

use super::state::lock;

/// Results kept for polling after a job ends.
const KEPT: usize = 8;

/// One line of a finished write.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct Line {
    pub(crate) project: String,
    pub(crate) skill: String,
    /// `created`, `updated`, `unchanged`, `failed`, `restored`, `removed`, `gone`.
    pub(crate) outcome: &'static str,
    pub(crate) detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Finished {
    pub(crate) title: String,
    pub(crate) lines: Vec<Line>,
    pub(crate) failed: usize,
    /// The backup run that kept what was replaced; undoing it brings it back.
    pub(crate) backup: Option<String>,
    pub(crate) warnings: Vec<String>,
}

#[derive(Default)]
pub(crate) struct Progress {
    pub(crate) done: AtomicUsize,
    pub(crate) total: AtomicUsize,
}

enum Outcome {
    Running,
    Done(Finished),
    Failed(String),
}

struct Job {
    id: String,
    progress: Arc<Progress>,
    outcome: Outcome,
}

#[derive(Default)]
pub(crate) struct Jobs {
    items: VecDeque<Job>,
}

/// What a poll sees.
#[derive(Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub(crate) enum View {
    Running { done: usize, total: usize },
    Done(Finished),
    Failed { message: String },
}

impl Jobs {
    pub(crate) fn busy(&self) -> bool {
        self.items
            .iter()
            .any(|job| matches!(job.outcome, Outcome::Running))
    }

    /// Registers a job that is about to run; `None` when another one is running.
    pub(crate) fn start(&mut self, id: String, total: usize) -> Option<Arc<Progress>> {
        if self.busy() {
            return None;
        }
        let progress = Arc::new(Progress::default());
        progress.total.store(total, Ordering::Relaxed);
        while self.items.len() >= KEPT {
            self.items.pop_front();
        }
        self.items.push_back(Job {
            id,
            progress: Arc::clone(&progress),
            outcome: Outcome::Running,
        });
        Some(progress)
    }

    pub(crate) fn finish(&mut self, id: &str, result: Result<Finished, String>) {
        if let Some(job) = self.items.iter_mut().find(|job| job.id == id) {
            job.outcome = match result {
                Ok(finished) => Outcome::Done(finished),
                Err(message) => Outcome::Failed(message),
            };
        }
    }

    pub(crate) fn view(&self, id: &str) -> Option<View> {
        let job = self.items.iter().find(|job| job.id == id)?;
        Some(match &job.outcome {
            Outcome::Running => View::Running {
                done: job.progress.done.load(Ordering::Relaxed),
                total: job.progress.total.load(Ordering::Relaxed),
            },
            Outcome::Done(finished) => View::Done(finished.clone()),
            Outcome::Failed(message) => View::Failed {
                message: message.clone(),
            },
        })
    }
}

/// Registers a job in the shared state.
pub(crate) fn start(
    jobs: &std::sync::Mutex<Jobs>,
    id: String,
    total: usize,
) -> Option<Arc<Progress>> {
    lock(jobs).start(id, total)
}
