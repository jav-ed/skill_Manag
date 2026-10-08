//! `list`: every skill folder installed in any project.

use skillmirror_core::ops::installed;
use skillmirror_core::vault::{Vault, discover};

use super::context::{load_settings, scan_root, warn_unreadable};
use crate::args::{Cli, ListArgs};
use crate::exit::Exit;
use crate::output::InstalledRow;
use crate::output::{self, ListJson};
use crate::report::CliError;

pub(super) fn run(cli: &Cli, args: &ListArgs) -> Result<Exit, CliError> {
    let settings = load_settings(cli)?;
    let report = scan_root(&settings, args.json)?;
    let vault = open_vault(&settings)?;
    let set = installed(&report, vault.as_ref());
    warn_unreadable(&report.issues);
    warn_unreadable(&set.issues);
    let rows: Vec<InstalledRow> = set
        .rows
        .iter()
        .map(|r| InstalledRow {
            project: r.target.project.clone(),
            skill: r.target.skill.clone(),
            in_vault: r.in_vault,
        })
        .collect();
    if args.json {
        output::line(&ListJson::new(&rows).render()?);
    } else {
        output::print(&output::render_installed(&rows));
    }
    Ok(Exit::Clean)
}

/// The vault is optional for `list`: without one, nothing is marked as missing from it.
fn open_vault(settings: &skillmirror_core::config::Settings) -> Result<Option<Vault>, CliError> {
    match settings.vault() {
        Ok(vault) => Ok(Some(
            discover(&vault.value).map_err(skillmirror_core::Error::from)?,
        )),
        Err(skillmirror_core::config::ConfigError::VaultMissing) => Ok(None),
        Err(other) => Err(other.into()),
    }
}
