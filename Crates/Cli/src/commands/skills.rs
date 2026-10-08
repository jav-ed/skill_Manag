//! `skills`: what the vault holds, grouped by folder.

use std::path::PathBuf;

use skillmirror_core::vault::discover;

use super::context::load_settings;
use crate::args::{Cli, SkillsArgs};
use crate::exit::Exit;
use crate::output::{self, SkillRow, VaultJson};
use crate::report::CliError;

pub(super) fn run(cli: &Cli, args: &SkillsArgs) -> Result<Exit, CliError> {
    let settings = load_settings(cli)?;
    let vault = discover(&settings.vault()?.value).map_err(skillmirror_core::Error::from)?;
    let filter: Option<PathBuf> = args
        .group
        .as_deref()
        .map(|g| g.trim_matches('/').split('/').collect());
    if let Some(group) = &filter
        && !vault.groups.contains(group)
    {
        return Err(skillmirror_core::ops::SelectError::UnknownGroup {
            path: group.display().to_string(),
        }
        .into());
    }
    let mut rows: Vec<SkillRow> = vault
        .skills
        .values()
        .filter(|s| filter.as_ref().is_none_or(|g| s.rel.starts_with(g)))
        .map(|s| SkillRow {
            name: s.name.clone(),
            group: s.group.clone(),
            mandatory: settings.mandatory().contains(&s.name),
        })
        .collect();
    rows.sort_by(|a, b| a.group.cmp(&b.group).then_with(|| a.name.cmp(&b.name)));
    if args.json {
        output::line(&VaultJson::new(&rows).render()?);
    } else {
        output::print(&output::render_vault(&rows));
    }
    Ok(Exit::Clean)
}
