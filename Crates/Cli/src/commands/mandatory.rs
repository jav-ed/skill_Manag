//! `mandatory list`, `mandatory add` and `mandatory remove`.

use skillmirror_core::config::{ConfigUpdate, VaultConfig, save_config};
use skillmirror_core::ops::{MandatoryChange, Workspace, change_mandatory};

use super::context::load_settings;
use crate::args::{Cli, MandatoryCommand, MandatoryEditArgs};
use crate::exit::Exit;
use crate::output::{self, MandatoryJson, render_mandatory};
use crate::report::CliError;

pub(super) fn run(cli: &Cli, command: &MandatoryCommand) -> Result<Exit, CliError> {
    match command {
        MandatoryCommand::List { json } => {
            let settings = load_settings(cli)?;
            let file = VaultConfig::path_in(&settings.vault()?.value);
            show(settings.mandatory(), &file, false, false, *json)?;
        }
        MandatoryCommand::Add(args) => edit(cli, args, true)?,
        MandatoryCommand::Remove(args) => edit(cli, args, false)?,
    }
    Ok(Exit::Clean)
}

fn edit(cli: &Cli, args: &MandatoryEditArgs, add: bool) -> Result<(), CliError> {
    let workspace = Workspace::open(load_settings(cli)?)?;
    let current = workspace.settings.mandatory();
    let change = if add {
        MandatoryChange::Add(&args.names)
    } else {
        MandatoryChange::Remove(&args.names)
    };
    let after = change_mandatory(&workspace.vault, current, change)?;
    let changed = after != current;
    let vault = &workspace.settings.vault()?.value;
    if changed && !args.dry_run {
        save_config(
            vault,
            &ConfigUpdate {
                root: None,
                mandatory: Some(&after),
            },
        )?;
    }
    show(
        &after,
        &VaultConfig::path_in(vault),
        changed,
        args.dry_run,
        args.json,
    )
}

fn show(
    list: &[String],
    file: &std::path::Path,
    changed: bool,
    dry_run: bool,
    json: bool,
) -> Result<(), CliError> {
    if json {
        output::line(&MandatoryJson::new(dry_run, changed, list).render()?);
    } else {
        output::print(&render_mandatory(list, file, changed, dry_run));
    }
    Ok(())
}
