//! Writing a confirmed plan with a backup run around it, the way every front end does it.

use crate::apply::{ApplyOptions, ApplyReport, apply};
use crate::backup::{Backups, Finished, RunKind};
use crate::config::Dirs;
use crate::events::Observer;
use crate::plan::Plan;

/// What a written plan came to.
#[derive(Debug)]
pub struct RunOutcome {
    pub report: ApplyReport,
    /// The backup run that kept what the plan replaced. It saved nothing (and left no folder) when the
    /// plan only created skills.
    pub backup: Finished,
}

/// Applies `plan` and keeps the old copies in a backup run of the given kind. A folder that cannot be
/// written fails alone and is in the report; the others are written.
pub fn run_plan(
    dirs: &Dirs,
    kind: RunKind,
    plan: Plan,
    observer: Observer<'_>,
) -> Result<RunOutcome, crate::Error> {
    let backups = Backups::in_dirs(dirs);
    let run = backups.begin(kind)?;
    let options = ApplyOptions {
        backup: Some(&run),
        ..ApplyOptions::default()
    };
    let report = apply(plan, options, observer)?;
    let backup = run.finish(&backups);
    Ok(RunOutcome { report, backup })
}
