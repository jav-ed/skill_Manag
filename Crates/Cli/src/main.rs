mod args;
mod args_more;
mod commands;
mod exit;
mod output;
mod report;

use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    let cli = args::Cli::parse();
    match commands::run(&cli) {
        Ok(exit) => exit.code(),
        Err(error) => {
            report::print_error(&error);
            error.exit().code()
        }
    }
}
