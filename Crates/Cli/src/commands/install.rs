//! `add` and `init`: install a selection of vault skills into one project.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::path::Path;

use skillmirror_core::agents::{Intent, Source, apply_agents, load_source, plan_agents};
use skillmirror_core::backup::RunKind;
use skillmirror_core::events::ignore_events;
use skillmirror_core::ops::{self, SelectError, Selection, Workspace};

use super::bridge::link_project;
use super::context::load_settings;
use super::pipeline::{AgentsHook, Hooks, Run, execute};
use crate::args::{AddArgs, Cli, InitArgs, InstallFlags, SelectArgs};
use crate::exit::Exit;
use crate::output::{self, AgentRow, BridgeRow};
use crate::report::CliError;

pub(super) fn add(cli: &Cli, args: &AddArgs) -> Result<Exit, CliError> {
    let workspace = Workspace::open(load_settings(cli)?)?;
    let project = match &args.project {
        Some(dir) => std::path::absolute(dir)?,
        None => std::env::current_dir()?,
    };
    ops::check_existing(&project)?;
    let skills = ops::resolve(
        &workspace.vault,
        workspace.settings.config(),
        &selection(&args.select),
    )?;
    let plan = ops::plan_install(&workspace, &project, &skills);
    let hooks = Hooks {
        before_write: &|| Ok(()),
        after_write: &link_after(&workspace, &project),
        agents: None,
    };
    execute(plan, &[], &run_of(RunKind::Add, &args.flags), &hooks).map(|done| done.exit)
}

pub(super) fn init(cli: &Cli, args: &InitArgs) -> Result<Exit, CliError> {
    let workspace = Workspace::open(load_settings(cli)?)?;
    let dir = std::path::absolute(&args.dir)?;
    ops::check_new(&dir)?;
    let skills = init_skills(&workspace, args)?;
    // Read and checked before anything is made: a text that cannot be used, or skills it names that the
    // project would not get, stop the command here.
    let source = if args.no_agents_md {
        None
    } else {
        let source = load_source(&workspace.vault.path)?;
        source.require_skills(&dir, &skills)?;
        Some(source)
    };
    let plan = ops::plan_install(&workspace, &dir, &skills);
    let made = RefCell::new(None);
    let agents = source.as_ref().and_then(|source| agents_plan(&dir, source));
    let write_agents = |run: &skillmirror_core::backup::Run, slots: usize| {
        agents
            .as_ref()
            .map(|(plan, planned)| written_row(plan, planned, run, slots))
    };
    let hooks = Hooks {
        before_write: &|| {
            *made.borrow_mut() = Some(ops::create_new(&dir, args.git)?);
            Ok(())
        },
        after_write: &link_after(&workspace, &dir),
        agents: agents
            .as_ref()
            .zip(source.as_ref())
            .map(|((_, planned), source)| AgentsHook {
                planned: planned.clone(),
                source: source.describe(),
                write: &write_agents,
            }),
    };
    let done = execute(plan, &[], &run_of(RunKind::Init, &args.flags), &hooks)?;
    // Nothing got installed: the project and repository this command made go, so it can be run again.
    if let (0, Some(project)) = (done.wrote, made.into_inner())
        && let Err(e) = project.take_back()
    {
        output::warn_line(&format!("could not remove the empty project: {e}"));
    }
    Ok(done.exit)
}

/// The links the vault config asks for, made in a project right after skills were written there. A link
/// that is blocked is reported in the output and does not turn the finished install into a failure.
fn link_after<'a>(
    workspace: &'a Workspace,
    project: &'a Path,
) -> impl Fn(usize) -> Vec<BridgeRow> + 'a {
    move |_| link_project(&workspace.settings.config().targets, project)
}

/// The mandatory skills (unless switched off) plus whatever was selected. At least one skill is required.
fn init_skills(workspace: &Workspace, args: &InitArgs) -> Result<BTreeSet<String>, CliError> {
    let mut skills = if args.no_mandatory {
        BTreeSet::new()
    } else {
        ops::resolve_names(&workspace.vault, workspace.settings.mandatory())?
    };
    let chosen = selection(&args.select);
    if !chosen.is_empty() {
        skills.extend(ops::resolve(
            &workspace.vault,
            workspace.settings.config(),
            &chosen,
        )?);
    }
    if skills.is_empty() {
        return Err(SelectError::Empty.into());
    }
    Ok(skills)
}

fn selection(args: &SelectArgs) -> Selection {
    Selection {
        skills: args.skills.clone(),
        groups: args.groups.clone(),
        profiles: args.profiles.clone(),
    }
}

fn run_of(kind: RunKind, flags: &InstallFlags) -> Run<'static> {
    Run {
        kind,
        dry_run: flags.dry_run,
        check: false,
        yes: flags.yes,
        json: flags.json,
        all: true,
        empty_message: "Nothing to install.",
    }
}

/// The plan for the AGENTS.md of the new project, and its row.
fn agents_plan(
    dir: &Path,
    source: &Source,
) -> Option<(skillmirror_core::agents::AgentsPlan, AgentRow)> {
    let plan = plan_agents(
        std::slice::from_ref(&dir.to_path_buf()),
        source,
        Intent::ADD,
    );
    let row = plan.entries.first().map(AgentRow::planned)?;
    Some((plan, row))
}

/// Writes the planned file inside the run the skills used, and says how it went.
fn written_row(
    plan: &skillmirror_core::agents::AgentsPlan,
    planned: &AgentRow,
    run: &skillmirror_core::backup::Run,
    slots: usize,
) -> AgentRow {
    let report = apply_agents(plan, Some(run), slots, &ignore_events);
    match report.applied.first() {
        Some(applied) => AgentRow::done(planned, applied),
        None => planned.clone(),
    }
}
