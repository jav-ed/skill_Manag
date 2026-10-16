//! One module per command.

mod agents;
mod authoring;
mod backup;
mod config;
mod context;
mod inspect;
mod migrate;
mod pipeline;
mod projects;
mod web;

use std::io::IsTerminal;

use clap::CommandFactory;

use crate::args::{Cli, Command};
use crate::exit::Exit;
use crate::output;
use crate::report::CliError;

pub(crate) fn run(cli: &Cli) -> Result<Exit, CliError> {
    match &cli.command {
        None if interactive() => interface(cli),
        None => {
            output::eprint(&Cli::command().render_help().to_string());
            Ok(Exit::Usage)
        }
        Some(Command::Tui) => interface(cli),
        Some(command) => dispatch(cli, command),
    }
}

fn interactive() -> bool {
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

fn interface(cli: &Cli) -> Result<Exit, CliError> {
    skillmirror_tui::run(context::launch(cli)?)?;
    Ok(Exit::Clean)
}

fn dispatch(cli: &Cli, command: &Command) -> Result<Exit, CliError> {
    match command {
        Command::Tui => interface(cli),
        Command::Sync(args) => projects::mirror::run(cli, args, projects::mirror::Which::Sync),
        Command::Push(args) => projects::mirror::run(cli, args, projects::mirror::Which::Push),
        Command::List(args) => inspect::list::run(cli, args),
        Command::Delete(args) => projects::delete::run(cli, args),
        Command::Skills(args) => inspect::skills::run(cli, args),
        Command::Info(args) => inspect::info::run(cli, args),
        Command::New(args) => authoring::author::new(cli, args),
        Command::Adopt(args) => authoring::author::adopt(cli, args),
        Command::Changes(args) => inspect::changes::run(cli, args),
        Command::Promote(args) => authoring::promote::run(cli, args),
        Command::Vault(command) => authoring::vault::run(cli, command),
        Command::Config(command) => config::run(cli, command),
        Command::Mandatory(command) => authoring::mandatory::run(cli, command),
        Command::Report(args) => inspect::report::run(cli, args),
        Command::Web(args) => web::run(cli, args),
        Command::Bridge(args) => projects::bridge::run(cli, args),
        Command::Status(args) => inspect::status::run(cli, args),
        Command::Diff(args) => inspect::diff::run(cli, args),
        Command::Doctor(args) => inspect::doctor::run(cli, args),
        Command::Agents(command) => agents::run(cli, command),
        Command::Add(args) => projects::install::add(cli, args),
        Command::Init(args) => projects::install::init(cli, args),
        Command::Undo(args) => backup::run_undo(args),
        Command::History(args) => backup::run_history(args),
        Command::Migrate(args) => migrate::run(args),
        Command::Completions { shell } => {
            let mut buffer = Vec::new();
            clap_complete::generate(
                *shell,
                &mut Cli::command(),
                skillmirror_core::brand::NAME,
                &mut buffer,
            );
            output::print(&String::from_utf8_lossy(&buffer));
            Ok(Exit::Clean)
        }
    }
}
