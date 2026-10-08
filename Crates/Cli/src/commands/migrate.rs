//! `migrate`: carry the vault pointer over from the old tool.

use skillmirror_core::brand;
use skillmirror_core::config::{ConfigError, Dirs, EnvOverrides, migrate};

use crate::args::MigrateArgs;
use crate::exit::Exit;
use crate::output;
use crate::report::CliError;

pub(super) fn run(args: &MigrateArgs) -> Result<Exit, CliError> {
    let dirs = Dirs::from_env()?;
    let report = migrate(&dirs, args.retire)?;
    if report.already_done {
        output::line(&format!(
            "Already migrated: {} points at {}",
            report.new_pointer.display(),
            report.vault.display()
        ));
    } else {
        output::line(&format!(
            "Copied the vault pointer to {}",
            report.new_pointer.display()
        ));
        output::line(&format!("Vault: {}", report.vault.display()));
    }
    if report.retired {
        output::line(&format!(
            "Removed the old pointer {}",
            report.old_pointer.display()
        ));
    } else {
        output::line(&format!(
            "The old pointer {} is kept; run `{} migrate --retire` to remove it",
            report.old_pointer.display(),
            brand::NAME
        ));
    }
    note_legacy_variables();
    Ok(Exit::Clean)
}

/// Tells the user which old environment variables are still set, since this tool refuses to start with them.
fn note_legacy_variables() {
    if let Err(ConfigError::LegacyEnv { vars }) = EnvOverrides::from_env() {
        output::warn_line(
            "these old environment variables are still set and must be renamed or removed:",
        );
        for var in vars {
            output::warn_line(&format!("  {var}"));
        }
        output::warn_line(&format!(
            "SKILL_MANAG_VAULT is now {}, SKILL_MANAG_ROOT is now {}",
            brand::ENV_VAULT,
            brand::ENV_ROOT
        ));
    }
}
