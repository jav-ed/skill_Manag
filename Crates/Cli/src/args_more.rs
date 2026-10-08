//! The arguments of the commands that grow the vault, change the configuration, or serve the web view.
// Doc comments here become `--help` text, where environment variable names must stay unquoted.
#![allow(clippy::doc_markdown)]

use std::path::PathBuf;

use clap::{Args, Subcommand};

/// Limits a command to some skills, to a vault folder or profile, or to one project.
#[derive(Debug, Args)]
pub(crate) struct ScopeArgs {
    /// Only these skills (default: all of them)
    #[arg(value_name = "SKILL")]
    pub(crate) skills: Vec<String>,
    /// Only the skills below this vault folder, such as `web` or `web/seo`
    #[arg(long = "group", value_name = "PATH")]
    pub(crate) groups: Vec<String>,
    /// Only the skills of a profile defined under `profiles:` in <vault>/config.yaml
    #[arg(long = "profile", value_name = "NAME")]
    pub(crate) profiles: Vec<String>,
    /// Only this project (default: every project under the root)
    #[arg(long, value_name = "DIR")]
    pub(crate) project: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub(crate) struct InfoArgs {
    /// The skill folder name
    #[arg(value_name = "SKILL")]
    pub(crate) skill: String,
    /// Print one JSON document instead of text
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct NewArgs {
    /// Name of the new skill: lowercase letters, digits, `-` and `_`
    #[arg(value_name = "NAME")]
    pub(crate) name: String,
    /// The vault folder to put it in, such as `web` or `web/seo` (made when it does not exist)
    #[arg(long, value_name = "PATH", default_value = "")]
    pub(crate) group: String,
    /// The line agents read to decide when to use the skill
    #[arg(long, value_name = "TEXT")]
    pub(crate) description: Option<String>,
    /// Show what would be created and create nothing
    #[arg(long)]
    pub(crate) dry_run: bool,
    /// Print one JSON document instead of text
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct AdoptArgs {
    /// The skill folder name in the project
    #[arg(value_name = "NAME")]
    pub(crate) name: String,
    /// The project that has it
    #[arg(long, value_name = "DIR", required = true)]
    pub(crate) from: PathBuf,
    /// The vault folder to put it in, such as `web` or `web/seo` (made when it does not exist)
    #[arg(long, value_name = "PATH", default_value = "")]
    pub(crate) group: String,
    /// Show what would be copied and copy nothing
    #[arg(long)]
    pub(crate) dry_run: bool,
    /// Print one JSON document instead of text
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Subcommand)]
pub(crate) enum VaultCommand {
    /// Make a new vault: a git repository with a config.yaml
    Init(VaultInitArgs),
}

#[derive(Debug, Args)]
pub(crate) struct VaultInitArgs {
    /// The new vault folder; it must not exist or must be empty
    pub(crate) dir: PathBuf,
    /// The folder that holds your projects, written to config.yaml as `root:`
    #[arg(long, value_name = "DIR")]
    pub(crate) root: Option<PathBuf>,
    /// Make it the default vault even when another one is set
    #[arg(long = "use")]
    pub(crate) use_it: bool,
    /// Show what would be created and create nothing
    #[arg(long)]
    pub(crate) dry_run: bool,
    /// Print one JSON document instead of text
    #[arg(long)]
    pub(crate) json: bool,
}

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
}

#[derive(Debug, Args)]
pub(crate) struct ConfigRootArgs {
    /// The folder to scan for projects; it must exist
    pub(crate) dir: PathBuf,
    /// Show the change and make none
    #[arg(long)]
    pub(crate) dry_run: bool,
}

#[derive(Debug, Subcommand)]
pub(crate) enum MandatoryCommand {
    /// List the mandatory skills
    List {
        /// Print one JSON document instead of text
        #[arg(long)]
        json: bool,
    },
    /// Add skills of the vault to the list
    Add(MandatoryEditArgs),
    /// Take skills off the list
    Remove(MandatoryEditArgs),
}

#[derive(Debug, Args)]
pub(crate) struct MandatoryEditArgs {
    /// Skill names
    #[arg(value_name = "SKILL", required = true)]
    pub(crate) names: Vec<String>,
    /// Show the new list and change nothing
    #[arg(long)]
    pub(crate) dry_run: bool,
    /// Print one JSON document instead of text
    #[arg(long)]
    pub(crate) json: bool,
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
