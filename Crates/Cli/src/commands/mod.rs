//! One module per command.

mod author;
mod backup;
mod bridge;
mod config;
mod context;
mod delete;
mod diff;
mod doctor;
mod info;
mod install;
mod list;
mod mandatory;
mod migrate;
mod mirror;
mod pipeline;
mod report;
mod skills;
mod status;
mod vault;
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
        Command::Sync(args) => mirror::run(cli, args, mirror::Which::Sync),
        Command::Push(args) => mirror::run(cli, args, mirror::Which::Push),
        Command::List(args) => list::run(cli, args),
        Command::Delete(args) => delete::run(cli, args),
        Command::Skills(args) => skills::run(cli, args),
        Command::Info(args) => info::run(cli, args),
        Command::New(args) => author::new(cli, args),
        Command::Adopt(args) => author::adopt(cli, args),
        Command::Vault(command) => vault::run(cli, command),
        Command::Config(command) => config::run(cli, command),
        Command::Mandatory(command) => mandatory::run(cli, command),
        Command::Report(args) => report::run(cli, args),
        Command::Web(args) => web::run(cli, args),
        Command::Bridge(args) => bridge::run(cli, args),
        Command::Status(args) => status::run(cli, args),
        Command::Diff(args) => diff::run(cli, args),
        Command::Doctor(args) => doctor::run(cli, args),
        Command::Add(args) => install::add(cli, args),
        Command::Init(args) => install::init(cli, args),
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
