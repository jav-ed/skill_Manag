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
    /// The scan is under way: how many folders it has looked at and how many projects it has found. Sent
    /// from the scan's threads, once per 256 folders, so a short scan sends none.
    ScanProgress { directories: usize, projects: usize },
    /// The scan found every `.agents/skills` directory.
    ScanFinished { skills_dirs: usize, issues: usize },
    /// One target finished, successfully or not.
    TargetDone { target: Target, status: Status },
}

/// Receives events from any thread. Use [`ignore_events`] when nobody listens.
pub type Observer<'a> = &'a (dyn Fn(Event) + Sync);

/// An observer that drops every event.
pub fn ignore_events(_event: Event) {}
