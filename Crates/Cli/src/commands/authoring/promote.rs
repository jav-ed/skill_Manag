//! `promote`: skills changed in a project replace the vault's copies, and can then be synced to the other
//! projects. Writes only the vault, stages it in git, never commits; the old folders go to a backup run.

use skillmirror_core::backup::RunKind;
use skillmirror_core::config::Dirs;
use skillmirror_core::ops::{self, PromotePlan, Scope, Workspace};

use crate::args::{Cli, PromoteArgs};
use crate::commands::backup;
use crate::commands::context::{Context, load_settings, local_project, open};
use crate::commands::pipeline::{Hooks, Run, execute};
use crate::exit::Exit;
use crate::output::{self, PromoteJson, render_promote_plan, render_promoted};
use crate::report::CliError;

pub(in crate::commands) fn run(cli: &Cli, args: &PromoteArgs) -> Result<Exit, CliError> {
    let workspace = Workspace::open(load_settings(cli)?)?;
    let (project, _) = local_project(&workspace, args.project.as_deref())?;
    let changes = ops::local_changes(&workspace, &project)?;
    let names: Vec<String> = if args.all {
        changes.edited().map(|s| s.name.clone()).collect()
    } else {
        args.names.clone()
    };
    if names.is_empty() {
        return nothing_to_promote(args, &project);
    }
    let plan = ops::plan_promote(&workspace, &changes, &names)?;
    if args.dry_run {
        show_plan(args, &plan)?;
        return Ok(Exit::Clean);
    }
    if !args.yes && !confirmed(args, &plan)? {
        output::line("Cancelled, nothing was replaced.");
        return Ok(Exit::Clean);
    }
    let report = ops::promote(&workspace.vault, &plan, &Dirs::from_env()?)?;
    if args.json {
        output::line(&PromoteJson::done(&plan, &report).render()?);
    } else {
        output::print(&render_promoted(&report));
    }
    backup::announce(&report.backup, args.json);
    let promoted: Vec<String> = report
        .skills
        .iter()
        .filter(|s| s.result.is_ok())
        .map(|s| s.name.clone())
        .collect();
    let mut exit = if report.failed() > 0 {
        Exit::Partial
    } else {
        Exit::Clean
    };
    if args.sync && !promoted.is_empty() {
        let synced = spread(cli, &promoted, args.yes)?;
        if synced != Exit::Clean {
            exit = Exit::Partial;
        }
    }
    Ok(exit)
}

fn nothing_to_promote(args: &PromoteArgs, project: &std::path::Path) -> Result<Exit, CliError> {
    if args.json {
        let empty = PromotePlan {
            project: project.to_path_buf(),
            skills: Vec::new(),
        };
        output::line(&PromoteJson::planned(&empty).render()?);
    } else {
        output::line(&format!(
            "Nothing to promote: no skill of {} was changed.",
            project.display()
        ));
    }
    Ok(Exit::Clean)
}

fn show_plan(args: &PromoteArgs, plan: &PromotePlan) -> Result<(), CliError> {
    if args.json {
        output::line(&PromoteJson::planned(plan).render()?);
    } else {
        output::print(&render_promote_plan(plan, true));
        if args.sync {
            output::line("--sync would then update the other projects that have these skills.");
        }
    }
    Ok(())
}

/// Shows what will be replaced and asks. Without a terminal the answer must come from `--yes`.
fn confirmed(args: &PromoteArgs, plan: &PromotePlan) -> Result<bool, CliError> {
    // The plan and the question would land on stdout in front of the document that `--json` promises.
    if args.json {
        return Err(CliError::usage(
            "--json cannot ask for confirmation",
            "pass --yes to replace, or --dry-run to preview",
        ));
    }
    if !output::can_prompt() {
        return Err(CliError::usage(
            "refusing to replace without confirmation",
            "pass --yes to replace, or --dry-run to preview",
        ));
    }
    output::print(&render_promote_plan(plan, false));
    let question = format!(
        "Replace {} skill(s) in the vault? The old ones are kept for `skillmirror undo`.",
        plan.skills.len()
    );
    Ok(output::confirm(&question)?)
}

/// The second step of `--sync`: the vault now has the new copies, so every other project that has these
/// skills is planned again from a fresh read of the vault and goes through the usual sync.
fn spread(cli: &Cli, names: &[String], yes: bool) -> Result<Exit, CliError> {
    let Context { workspace, report } = open(cli, false)?;
    let scope = Scope {
        skills: Some(names.iter().cloned().collect()),
        project: None,
    };
    let plan = ops::plan_sync_scoped(&workspace, &report, &scope)?;
    let run = Run {
        kind: RunKind::Sync,
        dry_run: false,
        check: false,
        yes,
        json: false,
        all: false,
        empty_message: "No project has these skills.",
    };
    execute(plan, &report.issues, &run, &Hooks::NONE).map(|done| done.exit)
}
