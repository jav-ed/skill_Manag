//! `add` and `init`: install a selection of vault skills into one project.

use std::collections::BTreeSet;
use std::path::Path;

use skillmirror_core::ops::{self, SelectError, Selection, Workspace};

use super::context::load_settings;
use super::pipeline::{Run, execute};
use crate::args::{AddArgs, Cli, InitArgs, InstallFlags, SelectArgs};
use crate::exit::Exit;
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
    execute(plan, &[], &run_of("add", &args.flags), &|| Ok(()))
}

pub(super) fn init(cli: &Cli, args: &InitArgs) -> Result<Exit, CliError> {
    let workspace = Workspace::open(load_settings(cli)?)?;
    let dir = std::path::absolute(&args.dir)?;
    ops::check_new(&dir)?;
    let skills = init_skills(&workspace, args)?;
    let plan = ops::plan_install(&workspace, &dir, &skills);
    execute(plan, &[], &run_of("init", &args.flags), &|| {
        create_project(&dir, args.git)
    })
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

fn create_project(dir: &Path, git: bool) -> Result<(), CliError> {
    ops::create_new(dir, git)?;
    Ok(())
}

fn selection(args: &SelectArgs) -> Selection {
    Selection {
        skills: args.skills.clone(),
        groups: args.groups.clone(),
        profiles: args.profiles.clone(),
    }
}

fn run_of<'a>(command: &'a str, flags: &InstallFlags) -> Run<'a> {
    Run {
        command,
        dry_run: flags.dry_run,
        check: false,
        yes: flags.yes,
        json: flags.json,
        all: true,
        empty_message: "Nothing to install.",
    }
}
