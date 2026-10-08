//! What one scan produces: the opened vault, the installed skills and what each target needs.

use skillmirror_core::Result;
use skillmirror_core::config::Settings;
use skillmirror_core::events::ignore_events;
use skillmirror_core::ops::{self, Installed, Workspace};
use skillmirror_core::plan::{Plan, PlanKind};
use skillmirror_core::scan::Target;

/// What applying the vault copy to one target would do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum State {
    /// The skill folder does not exist yet; this many files would be written.
    Create(usize),
    /// The folder differs from the vault in this many files or leftovers.
    Update(usize),
    Same,
    Failed(String),
}

#[derive(Debug, Clone)]
pub(crate) struct TargetState {
    pub(crate) target: Target,
    pub(crate) state: State,
}

/// Something the scan could not read, for the list behind `i`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Issue {
    pub(crate) path: String,
    pub(crate) message: String,
}

#[derive(Debug)]
pub(crate) struct Session {
    pub(crate) workspace: Workspace,
    /// Installed folders that the vault also has (what `sync` would refresh).
    pub(crate) sync: Vec<TargetState>,
    /// Mandatory skills in every project with a skills directory, or why they cannot be planned.
    pub(crate) push: std::result::Result<Vec<TargetState>, String>,
    /// Every installed skill folder, in walk order.
    pub(crate) installed: Vec<Installed>,
    /// What the scan and the list of installed folders could not read.
    pub(crate) issues: Vec<Issue>,
}

pub(crate) use skillmirror_core::describe;

/// Opens the vault, scans the root and plans sync and push, all read-only.
pub(crate) fn load(settings: Settings) -> std::result::Result<Session, String> {
    build(settings).map_err(|e| describe(&e))
}

fn build(settings: Settings) -> Result<Session> {
    let workspace = Workspace::open(settings)?;
    let report = workspace.scan(&ignore_events)?;
    let sync = states(ops::plan_sync(&workspace, &report));
    let push = ops::plan_push(&workspace, &report)
        .map(states)
        .map_err(|e| describe(&e));
    let set = ops::installed(&report, Some(&workspace.vault));
    let issues = report
        .issues
        .iter()
        .chain(&set.issues)
        .map(|i| Issue {
            path: i.path.display().to_string(),
            message: i.message.clone(),
        })
        .collect();
    Ok(Session {
        workspace,
        sync,
        push,
        installed: set.rows,
        issues,
    })
}

fn states(plan: Plan) -> Vec<TargetState> {
    plan.entries
        .into_iter()
        .map(|entry| {
            let state = match entry.result {
                Ok(plan) => match plan.kind {
                    PlanKind::Create => State::Create(plan.file_count()),
                    PlanKind::Update => State::Update(plan.changes.len() + plan.removed.len()),
                    PlanKind::Unchanged => State::Same,
                },
                Err(e) => State::Failed(describe(&e)),
            };
            TargetState {
                target: entry.target,
                state,
            }
        })
        .collect()
}
