//! The arguments of the commands that change the configuration or serve the web view: `config` and `web`.
use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};

#[derive(Debug, Subcommand)]
pub(crate) enum ConfigCommand {
    /// Show the vault, the scan root and the settings in <vault>/config.yaml, and where each came from
    Show {
        /// Print one JSON document instead of text
        #[arg(long)]
        json: bool,
    },
    /// Print the path of <vault>/config.yaml, for `$EDITOR $(skillmirror config path)`
    Path,
    /// Set `root:` in <vault>/config.yaml to a folder that holds your projects
    Root(ConfigRootArgs),
    /// Turn the web view on or off with `web:` in <vault>/config.yaml (it is off until you turn it on)
    Web(ConfigWebArgs),
}

/// On or off, as `config web` takes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum Switch {
    On,
    Off,
}

#[derive(Debug, Args)]
pub(crate) struct ConfigWebArgs {
    /// `on` lets `skillmirror web` start, `off` makes it refuse
    pub(crate) state: Switch,
    /// Show the change and make none
    #[arg(long)]
    pub(crate) dry_run: bool,
}

#[derive(Debug, Args)]
pub(crate) struct ConfigRootArgs {
    /// The folder to scan for projects; it must exist
    pub(crate) dir: PathBuf,
    /// Show the change and make none
    #[arg(long)]
    pub(crate) dry_run: bool,
}

#[derive(Debug, Args)]
pub(crate) struct WebArgs {
    /// Let the pages run sync, push and undo (each shows its plan first and asks)
    #[arg(long)]
    pub(crate) allow_write: bool,
    /// The port on 127.0.0.1 (default: any free one)
    #[arg(long, default_value_t = 0)]
    pub(crate) port: u16,
    /// Open the link in the browser (xdg-open)
    #[arg(long)]
    pub(crate) open: bool,
    /// Stop after this many minutes without a request; 0 never stops
    #[arg(long, value_name = "MINUTES", default_value_t = 120.0, value_parser = minutes)]
    pub(crate) idle_timeout: f64,
}

/// A number of minutes: not negative, not infinite, not too large to count in seconds.
fn minutes(text: &str) -> Result<f64, String> {
    let value: f64 = text
        .parse()
        .map_err(|_| format!("{text:?} is not a number"))?;
    if value.is_finite() && (0.0..=525_600.0).contains(&value) {
        Ok(value)
    } else {
        Err("use a number of minutes between 0 and 525600 (a year)".to_string())
    }
}
