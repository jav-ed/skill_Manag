//! `config show`, `config path` and `config root`.

use skillmirror_core::config::{ConfigUpdate, VaultConfig, save_config};

use super::context::load_settings;
use crate::args::{Cli, ConfigCommand, ConfigRootArgs};
use crate::exit::Exit;
use crate::output::{self, ConfigJson, render_config};
use crate::report::CliError;

pub(super) fn run(cli: &Cli, command: &ConfigCommand) -> Result<Exit, CliError> {
    let settings = load_settings(cli)?;
    match command {
        ConfigCommand::Show { json } => {
            if *json {
                output::line(&ConfigJson::of(&settings).render()?);
            } else {
                output::print(&render_config(&settings));
            }
        }
        ConfigCommand::Path => {
            output::line(
                &VaultConfig::path_in(&settings.vault()?.value)
                    .display()
                    .to_string(),
            );
        }
        ConfigCommand::Root(args) => root(&settings, args)?,
    }
    Ok(Exit::Clean)
}

fn root(
    settings: &skillmirror_core::config::Settings,
    args: &ConfigRootArgs,
) -> Result<(), CliError> {
    let dir = std::path::absolute(&args.dir)?;
    match fs_err::metadata(&dir) {
        Ok(meta) if meta.is_dir() => {}
        Ok(_) => {
            return Err(CliError::usage(
                format!("{} is not a folder", dir.display()),
                "the scan root is the folder that holds your projects",
            ));
        }
        Err(e) => {
            return Err(CliError::usage(
                format!("{}: {e}", dir.display()),
                "the scan root is the folder that holds your projects",
            ));
        }
    }
    let vault = &settings.vault()?.value;
    if !args.dry_run {
        save_config(
            vault,
            &ConfigUpdate {
                root: Some(&dir),
                mandatory: None,
            },
        )?;
    }
    output::line(&format!(
        "{} root: {}",
        if args.dry_run { "Would set" } else { "Set" },
        dir.display()
    ));
    if let Ok(now) = settings.root()
        && now.source != skillmirror_core::config::Source::VaultConfig
    {
        output::warn_line(&format!(
            "{} still decides the root in this shell ({})",
            if now.source == skillmirror_core::config::Source::Flag {
                "--root"
            } else {
                "SKILLMIRROR_ROOT"
            },
            now.value.display()
        ));
    }
    Ok(())
}
