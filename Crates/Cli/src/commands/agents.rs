//! `agents status`, `agents sync` and `agents add`: the AGENTS.md of projects.

use std::path::{Path, PathBuf};

use skillmirror_core::agents::{
    AgentsPlan, Intent, Source, apply_agents, inspect_project, installed_skills, load_source,
    plan_agents,
};
use skillmirror_core::backup::RunKind;
use skillmirror_core::events::ignore_events;
use skillmirror_core::ops;

use super::backup;
use super::context::{Context, load_settings, open};
use crate::args::{
    AgentsAddArgs, AgentsCommand, AgentsFlags, AgentsStatusArgs, AgentsSyncArgs, Cli,
};
use crate::exit::Exit;
use crate::output::{
    self, AgentRow, AgentsRunJson, AgentsStatusJson, AgentsTense, render_agents_plan,
    render_agents_status, render_diffs,
};
use crate::report::CliError;

pub(super) fn run(cli: &Cli, command: &AgentsCommand) -> Result<Exit, CliError> {
    match command {
        AgentsCommand::Status(args) => status(cli, args),
        AgentsCommand::Sync(args) => sync(cli, args),
        AgentsCommand::Add(args) => add(cli, args),
    }
}

/// The text and the projects a command works on: the one named, or every project the scan found.
fn gather(
    cli: &Cli,
    project: Option<&Path>,
    quiet: bool,
) -> Result<(Source, Vec<PathBuf>), CliError> {
    let Some(dir) = project else {
        let Context { workspace, report } = open(cli, quiet)?;
        let source = load_source(&workspace.vault.path)?;
        let projects = report
            .skills_dirs
            .iter()
            .map(|d| d.project.clone())
            .collect();
        return Ok((source, projects));
    };
    let settings = load_settings(cli)?;
    let dir = std::path::absolute(dir)?;
    ops::check_existing(&dir)?;
    Ok((load_source(&settings.vault()?.value)?, vec![dir]))
}

fn status(cli: &Cli, args: &AgentsStatusArgs) -> Result<Exit, CliError> {
    let (source, projects) = gather(cli, args.project.as_deref(), args.json)?;
    let rows: Vec<AgentRow> = projects
        .into_iter()
        .map(|p| {
            let state = inspect_project(&p, &source);
            AgentRow::of_state(p, &state)
        })
        .collect();
    if args.json {
        output::line(&AgentsStatusJson::new(source.describe(), &rows).render()?);
    } else if rows.is_empty() {
        output::line("No projects with a skills directory found.");
    } else {
        output::print(&render_agents_status(&rows, &source.describe(), args.all));
    }
    let counts = crate::output::AgentsStatusCounts::of(&rows);
    // Like `status`: files that cannot be used first, then drift, else clean.
    Ok(if counts.unusable() > 0 {
        Exit::Partial
    } else if counts.drifting() > 0 {
        Exit::Drift
    } else {
        Exit::Clean
    })
}

fn sync(cli: &Cli, args: &AgentsSyncArgs) -> Result<Exit, CliError> {
    let (source, projects) = gather(cli, args.project.as_deref(), args.flags.json)?;
    let intent = Intent {
        force: args.force,
        ..Intent::SYNC
    };
    write(
        "agents sync",
        &plan_agents(&projects, &source, intent),
        &args.flags,
    )
}

fn add(cli: &Cli, args: &AgentsAddArgs) -> Result<Exit, CliError> {
    let project = match &args.project {
        Some(dir) => dir.clone(),
        None => std::env::current_dir()?,
    };
    let (source, projects) = gather(cli, Some(&project), args.flags.json)?;
    let plan = plan_agents(&projects, &source, Intent::ADD);
    if plan.changes() > 0
        && let Some(dir) = projects.first()
    {
        // The file would point at skills the project does not have.
        source.require_skills(dir, &installed_skills(dir))?;
    }
    write("agents add", &plan, &args.flags)
}

/// Shows the plan, asks, writes, shows the result.
fn write(command: &str, plan: &AgentsPlan, flags: &AgentsFlags) -> Result<Exit, CliError> {
    let source = plan.source.describe();
    let rows: Vec<AgentRow> = plan.entries.iter().map(AgentRow::planned).collect();
    if rows.is_empty() {
        output::line("No projects with a skills directory found.");
        return Ok(Exit::Clean);
    }
    if flags.dry_run || plan.changes() == 0 {
        show_plan(command, plan, &rows, &source, flags)?;
        return Ok(if plan.failed() > 0 {
            Exit::Partial
        } else {
            Exit::Clean
        });
    }
    if !flags.yes && !confirm(command, plan, &rows, &source, flags)? {
        output::line("Cancelled, nothing was written.");
        return Ok(Exit::Clean);
    }
    let (backups, run) = backup::begin(RunKind::Agents)?;
    let report = apply_agents(plan, Some(&run), 0, &ignore_events);
    let finished = run.finish(&backups);
    for left in report.applied.iter().filter_map(|a| a.leftover.as_ref()) {
        output::warn_line(&format!(
            "the update is done, but the old file {} could not be removed ({}); delete it by hand",
            left.path.display(),
            left.reason
        ));
    }
    let done: Vec<AgentRow> = rows
        .iter()
        .zip(&report.applied)
        .map(|(planned, applied)| AgentRow::done(planned, applied))
        .collect();
    if flags.json {
        output::line(
            &AgentsRunJson::new(command, "apply", source, &done)
                .with_backup(backup::saved(&finished))
                .render()?,
        );
    } else {
        output::print(&render_agents_plan(
            &done,
            &source,
            AgentsTense::Done,
            flags.all,
        ));
    }
    backup::announce(&finished, flags.json);
    Ok(if report.failed() > 0 {
        Exit::Partial
    } else {
        Exit::Clean
    })
}

fn show_plan(
    command: &str,
    plan: &AgentsPlan,
    rows: &[AgentRow],
    source: &str,
    flags: &AgentsFlags,
) -> Result<(), CliError> {
    if flags.json {
        output::line(&AgentsRunJson::new(command, "dry-run", source.to_string(), rows).render()?);
        return Ok(());
    }
    output::print(&render_agents_plan(
        rows,
        source,
        AgentsTense::Plan,
        flags.all,
    ));
    if flags.diff {
        output::print(&render_diffs(plan));
    }
    Ok(())
}

/// Shows what will be written and asks. Without a terminal the answer must come from `--yes`.
fn confirm(
    command: &str,
    plan: &AgentsPlan,
    rows: &[AgentRow],
    source: &str,
    flags: &AgentsFlags,
) -> Result<bool, CliError> {
    if flags.json {
        return Err(CliError::usage(
            "--json cannot ask for confirmation",
            "pass --yes to apply, or --dry-run to preview",
        ));
    }
    if !output::can_prompt() {
        return Err(CliError::usage(
            "refusing to write without confirmation",
            "pass --yes to apply, or --dry-run to preview",
        ));
    }
    show_plan(command, plan, rows, source, flags)?;
    let question = format!("Write AGENTS.md in {} project(s)?", plan.changes());
    Ok(output::confirm(&question)?)
}
