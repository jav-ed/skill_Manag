//! `delete`: remove one skill folder from one project or from every project that has it.

use skillmirror_core::backup::RunKind;
use skillmirror_core::events::ignore_events;
use skillmirror_core::ops::{self, Deleted};
use skillmirror_core::scan::Target;

use super::backup;
use super::context::{load_settings, scan_root, warn_unreadable};
use crate::args::{Cli, DeleteArgs};
use crate::exit::Exit;
use crate::output::{self, DeleteJson};
use crate::output::{DeleteRow, DeleteStatus};
use crate::report::CliError;

pub(super) fn run(cli: &Cli, args: &DeleteArgs) -> Result<Exit, CliError> {
    let targets = find_targets(cli, args)?;
    if targets.is_empty() {
        let message = format!("{} not found in any project.", args.name);
        if args.json {
            output::line(&DeleteJson::new(args.dry_run, &[]).render()?);
        } else {
            output::line(&message);
        }
        return Ok(Exit::Clean);
    }
    if !args.dry_run && !args.yes && !confirm(&targets, args)? {
        output::line("Cancelled, nothing was removed.");
        return Ok(Exit::Clean);
    }
    let (backups, backup_run) = backup::begin(RunKind::Delete)?;
    let report = ops::delete(targets, args.dry_run, Some(&backup_run), &ignore_events);
    let finished = backup_run.finish(&backups);
    let failed = report.failed();
    let rows: Vec<DeleteRow> = report
        .deleted
        .iter()
        .map(|d| to_row(d, args.dry_run))
        .collect();
    if args.json {
        let document = DeleteJson::new(args.dry_run, &rows);
        output::line(&document.with_backup(backup::saved(&finished)).render()?);
    } else {
        output::print(&output::render_delete(&rows, args.dry_run));
    }
    backup::announce(&finished, args.json);
    Ok(if failed > 0 {
        Exit::Partial
    } else {
        Exit::Clean
    })
}

fn find_targets(cli: &Cli, args: &DeleteArgs) -> Result<Vec<Target>, CliError> {
    if let Some(project) = &args.project {
        let project = std::path::absolute(project)?;
        return Ok(vec![ops::target_in_project(&project, &args.name)?]);
    }
    let settings = load_settings(cli)?;
    let report = scan_root(&settings, args.json)?;
    let set = ops::targets_named(&report, &args.name)?;
    warn_unreadable(&report.issues);
    warn_unreadable(&set.issues);
    Ok(set.targets)
}

fn confirm(targets: &[Target], args: &DeleteArgs) -> Result<bool, CliError> {
    if !output::can_prompt() {
        return Err(CliError::usage(
            "refusing to delete without confirmation",
            "pass --yes to delete, or --dry-run to preview",
        ));
    }
    let question = format!("Delete {} from {} project(s)?", args.name, targets.len());
    Ok(output::confirm(&question)?)
}

fn to_row(deleted: &Deleted, dry_run: bool) -> DeleteRow {
    let status = match &deleted.result {
        Ok(()) if dry_run => DeleteStatus::WouldDelete,
        Ok(()) => DeleteStatus::Deleted,
        Err(e) => DeleteStatus::Failed {
            message: e.to_string(),
        },
    };
    DeleteRow {
        project: deleted.target.project.clone(),
        skill: deleted.target.skill.clone(),
        status,
    }
}
