//! Progress events. Operations report through an observer, so the CLI, the TUI and the web view share one stream.

use crate::scan::Target;

/// What happened to one target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Unchanged,
    Created,
    Updated,
    Deleted,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// The scan found every `.agents/skills` directory.
    ScanFinished { skills_dirs: usize, issues: usize },
    /// One target finished, successfully or not.
    TargetDone { target: Target, status: Status },
}

/// Receives events from any thread. Use [`ignore_events`] when nobody listens.
pub type Observer<'a> = &'a (dyn Fn(Event) + Sync);

/// An observer that drops every event.
pub fn ignore_events(_event: Event) {}
