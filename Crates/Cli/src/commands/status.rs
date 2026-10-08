//! `status`: how every project stands against the vault, read-only.

use skillmirror_core::ops;

use super::context::{Context, open, scope_of};
use crate::args::{Cli, StatusArgs};
use crate::exit::Exit;
use crate::output::{self, ProjectRow, StatusJson, StatusSummary};
use crate::report::CliError;

pub(super) fn run(cli: &Cli, args: &StatusArgs) -> Result<Exit, CliError> {
    let Context { workspace, report } = open(cli, args.json)?;
    let scope = scope_of(&workspace, &args.scope)?;
    let status = ops::status_scoped(&workspace, &report, &scope)?;
    if args.json {
        let rows: Vec<ProjectRow> = status.projects.iter().map(ProjectRow::of).collect();
        output::line(&StatusJson::new(&rows, StatusSummary::of(&status)).render()?);
    } else if status.projects.is_empty() {
        output::line("No projects with a skills directory found.");
    } else {
        output::print(&output::render_status(&status, args.all));
    }
    // Like `sync --check`: failures first, then drift, else clean.
    Ok(if status.failed() > 0 {
        Exit::Partial
    } else if status.drifting() > 0 {
        Exit::Drift
    } else {
        Exit::Clean
    })
}
