//! `diff`: the lines a sync would bring in and take away, read-only.

use skillmirror_core::ops::{self, DiffFilter};

use super::context::{Context, open};
use crate::args::{Cli, DiffArgs};
use crate::exit::Exit;
use crate::output::{self, DiffJson};
use crate::report::CliError;

pub(super) fn run(cli: &Cli, args: &DiffArgs) -> Result<Exit, CliError> {
    let Context { workspace, report } = open(cli)?;
    if let Some(skill) = &args.skill {
        // A name the vault does not have is a mistake, not "no differences".
        ops::resolve_names(&workspace.vault, std::slice::from_ref(skill))?;
    }
    let filter = DiffFilter {
        skill: args.skill.clone(),
        project: args
            .project
            .as_deref()
            .map(std::path::absolute)
            .transpose()?,
    };
    let diff = ops::diff(&workspace, &report, &filter);
    if args.json {
        output::line(&DiffJson::of(&diff).render()?);
    } else {
        for (target, error) in &diff.failed {
            output::warn_line(&format!(
                "{} in {} cannot be compared: {error}",
                target.skill,
                target.project.display()
            ));
        }
        if diff.skills.is_empty() {
            output::line("No differences: every matching skill is up to date.");
        } else {
            output::print(&output::render_diff(&diff, args.stat));
        }
    }
    // Like `git diff --exit-code`: 1 when something differs; a comparison that failed comes first.
    Ok(if !diff.failed.is_empty() {
        Exit::Partial
    } else if diff.skills.is_empty() {
        Exit::Clean
    } else {
        Exit::Drift
    })
}
