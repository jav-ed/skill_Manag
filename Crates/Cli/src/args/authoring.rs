//! The arguments of the commands that grow the vault: `new`, `adopt`, `vault init` and `mandatory`.
use std::path::PathBuf;

use clap::{Args, Subcommand};

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
