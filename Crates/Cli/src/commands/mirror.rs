//! `sync` and `push`: plan against the scanned projects, then run the shared pipeline.

use skillmirror_core::backup::RunKind;
use skillmirror_core::ops::{Workspace, plan_push, plan_sync};
use skillmirror_core::plan::Plan;
use skillmirror_core::scan::ScanReport;

use super::context::{Context, open};
use super::pipeline::{Hooks, Run, execute};
use crate::args::{ApplyArgs, Cli};
use crate::exit::Exit;
use crate::report::CliError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Which {
    Sync,
    Push,
}

impl Which {
    fn kind(self) -> RunKind {
        match self {
            Self::Sync => RunKind::Sync,
            Self::Push => RunKind::Push,
        }
    }

    fn plan(self, workspace: &Workspace, report: &ScanReport) -> Result<Plan, CliError> {
        Ok(match self {
            Self::Sync => plan_sync(workspace, report),
            Self::Push => plan_push(workspace, report)?,
        })
    }

    fn empty_message(self) -> &'static str {
        match self {
            Self::Sync => "No matching skills found in any project.",
            Self::Push => "No projects with a skills directory found.",
        }
    }
}

pub(super) fn run(cli: &Cli, args: &ApplyArgs, which: Which) -> Result<Exit, CliError> {
    let Context { workspace, report } = open(cli)?;
    let run = Run {
        kind: which.kind(),
        dry_run: args.dry_run,
        check: args.check,
        yes: args.yes,
        json: args.json,
        all: args.all,
        empty_message: which.empty_message(),
    };
    if let Some(message) = nothing_to_do(&workspace, which) {
        return finish_without_plan(&run, message);
    }
    let plan = which.plan(&workspace, &report)?;
    execute(plan, &report.issues, &run, &Hooks::NONE).map(|done| done.exit)
}

/// Why there is nothing to plan, when the vault side is empty.
fn nothing_to_do(workspace: &Workspace, which: Which) -> Option<&'static str> {
    match which {
        Which::Sync if workspace.vault.skills.is_empty() => Some("No skills found in vault."),
        Which::Push if workspace.settings.mandatory().is_empty() => {
            Some("No mandatory skills configured in the vault config.")
        }
        _ => None,
    }
}

fn finish_without_plan(run: &Run<'_>, message: &str) -> Result<Exit, CliError> {
    let run = Run {
        empty_message: message,
        ..*run
    };
    execute(Plan::default(), &[], &run, &Hooks::NONE).map(|done| done.exit)
}
