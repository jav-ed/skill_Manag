//! `new` and `adopt`: grow the vault. Only ever creates files, stages them, never commits.

use skillmirror_core::ops::{self, Authored, Workspace};

use super::context::load_settings;
use crate::args::{AdoptArgs, Cli, NewArgs};
use crate::exit::Exit;
use crate::output::{self, AuthoredJson, render_authored};
use crate::report::CliError;

pub(super) fn new(cli: &Cli, args: &NewArgs) -> Result<Exit, CliError> {
    let workspace = Workspace::open(load_settings(cli)?)?;
    let vault = &workspace.vault;
    let made = if args.dry_run {
        ops::new_skill_paths(vault, &args.name, &args.group)?
    } else {
        ops::new_skill(vault, &args.name, &args.group, args.description.as_deref())?
    };
    report(
        "new",
        &args.name,
        &made,
        args.dry_run,
        args.json,
        "from a template",
    )
}

pub(super) fn adopt(cli: &Cli, args: &AdoptArgs) -> Result<Exit, CliError> {
    let workspace = Workspace::open(load_settings(cli)?)?;
    let project = std::path::absolute(&args.from)?;
    let plan = ops::plan_adopt(&workspace.vault, &project, &args.name, &args.group)?;
    let made = if args.dry_run {
        Authored {
            dir: plan.dest.clone(),
            files: plan.files.clone(),
            left_out: Vec::new(),
        }
    } else {
        ops::adopt(&workspace.vault, &plan)?
    };
    report(
        "adopt",
        &args.name,
        &made,
        args.dry_run,
        args.json,
        "copied from the project",
    )
}

fn report(
    command: &'static str,
    name: &str,
    made: &Authored,
    dry_run: bool,
    json: bool,
    how: &str,
) -> Result<Exit, CliError> {
    if json {
        let document = AuthoredJson::new(
            command,
            dry_run,
            name,
            &made.dir,
            &made.files,
            &made.left_out,
        );
        output::line(&document.render()?);
    } else {
        output::print(&render_authored(
            how,
            name,
            &made.dir,
            &made.files,
            &made.left_out,
            dry_run,
        ));
    }
    // A file git did not take is a skill that is not whole yet.
    Ok(if made.left_out.is_empty() {
        Exit::Clean
    } else {
        Exit::Partial
    })
}
