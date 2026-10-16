//! `changes`: which skills of a project differ from the vault, and how. Read-only.

use skillmirror_core::ops::{self, Workspace};

use crate::args::{ChangesArgs, Cli};
use crate::commands::context::{load_settings, local_project};
use crate::exit::Exit;
use crate::output::{self, ChangesJson, render_changes};
use crate::report::CliError;

pub(in crate::commands) fn run(cli: &Cli, args: &ChangesArgs) -> Result<Exit, CliError> {
    let workspace = Workspace::open(load_settings(cli)?)?;
    let (project, origin) = local_project(&workspace, args.project.as_deref())?;
    let changes = ops::local_changes(&workspace, &project)?;
    if args.json {
        output::line(&ChangesJson::of(&changes, origin).render()?);
    } else {
        let here = std::env::current_dir()?;
        output::print(&render_changes(&changes, origin, &here, args.all));
    }
    // Like `git diff --exit-code`: 1 when there is something to promote.
    Ok(if changes.edited().next().is_some() {
        Exit::Drift
    } else {
        Exit::Clean
    })
}
