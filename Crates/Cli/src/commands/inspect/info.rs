//! `info`: one skill from every side, read-only.

use skillmirror_core::ops;

use crate::args::{Cli, InfoArgs};
use crate::commands::context::{Context, open};
use crate::exit::Exit;
use crate::output::{self, InfoJson, render_info};
use crate::report::CliError;

pub(in crate::commands) fn run(cli: &Cli, args: &InfoArgs) -> Result<Exit, CliError> {
    let Context { workspace, report } = open(cli, args.json)?;
    let detail = ops::skill_detail(&workspace, &report, &args.skill)?;
    if args.json {
        output::line(&InfoJson::of(&detail).render()?);
    } else {
        output::print(&render_info(&detail));
    }
    Ok(Exit::Clean)
}
