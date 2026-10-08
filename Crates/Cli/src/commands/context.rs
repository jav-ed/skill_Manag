//! Shared start-up of the commands that read the vault and scan the root.

use skillmirror_core::config::{Dirs, EnvOverrides, Flags, Settings};
use skillmirror_core::events::Event;
use skillmirror_core::ops::Workspace;
use skillmirror_core::scan::{ScanIssue, ScanReport};
use skillmirror_tui::Launch;

use crate::args::Cli;
use crate::output::{self, ScanLine};
use crate::report::CliError;

pub(super) fn flags(cli: &Cli) -> Flags {
    Flags {
        vault: cli.vault.clone(),
        root: cli.root.clone(),
    }
}

/// Settings from flags, environment and files. Legacy variables and a legacy-only config are hard errors.
pub(super) fn load_settings(cli: &Cli) -> Result<Settings, CliError> {
    Ok(Settings::load(
        &flags(cli),
        &EnvOverrides::from_env()?,
        &Dirs::from_env()?,
    )?)
}

/// What the interactive interface needs, including a way to load the settings again after the setup
/// wizard changed them.
pub(super) fn launch(cli: &Cli) -> Result<Launch, CliError> {
    let dirs = Dirs::from_env()?;
    let env = EnvOverrides::from_env()?;
    let flags = flags(cli);
    let settings = Settings::load(&flags, &env, &dirs)?;
    let reload_dirs = dirs.clone();
    Ok(Launch {
        settings,
        dirs,
        reload: Box::new(move || Settings::load(&flags, &env, &reload_dirs)),
    })
}

/// An opened vault and a finished scan.
pub(super) struct Context {
    pub(super) workspace: Workspace,
    pub(super) report: ScanReport,
}

/// Opens the vault and scans the root. A person at a terminal sees a live line while it runs, unless
/// `quiet` (the command prints JSON).
pub(super) fn open(cli: &Cli, quiet: bool) -> Result<Context, CliError> {
    let workspace = Workspace::open(load_settings(cli)?)?;
    let line = ScanLine::start(quiet);
    let scanned = workspace.scan(&|event| line.hear(&event));
    line.clear();
    let report = scanned?;
    warn_unreadable(&report.issues);
    Ok(Context { workspace, report })
}

/// Tells the user what the scan found that needs a hand: unreadable folders, odd names, leftovers. It never hides them.
pub(super) fn warn_unreadable(issues: &[ScanIssue]) {
    const SHOWN: usize = 3;
    if issues.is_empty() {
        return;
    }
    output::warn_line(&format!("{} problems found while scanning:", issues.len()));
    for issue in issues.iter().take(SHOWN) {
        output::warn_line(&format!("  {}: {}", issue.path.display(), issue.message));
    }
    if issues.len() > SHOWN {
        output::warn_line(&format!("  and {} more", issues.len() - SHOWN));
    }
}

/// Scans the configured root with the live line, for the commands that do not open a whole workspace.
pub(super) fn scan_root(settings: &Settings, quiet: bool) -> Result<ScanReport, CliError> {
    let line = ScanLine::start(quiet);
    let scanned = skillmirror_core::scan::scan_with_progress(
        &settings.root()?.value,
        &settings.scan_options(),
        &|counts| {
            line.hear(&Event::ScanProgress {
                directories: counts.directories,
                projects: counts.projects,
            });
        },
    );
    line.clear();
    Ok(scanned?)
}
