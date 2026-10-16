//! The arguments of the commands that grow the vault: `new`, `adopt`, `promote`, `vault init` and `mandatory`,
//! and of `changes`, which shows what `promote` can take.
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

#[derive(Debug, Args)]
pub(crate) struct ChangesArgs {
    /// The project (a folder with .agents/skills); without it, the one around the current folder
    #[arg(long, value_name = "DIR")]
    pub(crate) project: Option<PathBuf>,
    /// Also list the skills that equal the vault's
    #[arg(long)]
    pub(crate) all: bool,
    /// Print one JSON document instead of text
    #[arg(long)]
    pub(crate) json: bool,
}

// Independent command-line switches, not a state machine.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Args)]
pub(crate) struct PromoteArgs {
    /// The skill folders of the project to put into the vault
    #[arg(value_name = "SKILL", required_unless_present = "all")]
    pub(crate) names: Vec<String>,
    /// Every skill the project changed
    #[arg(long, conflicts_with = "names")]
    pub(crate) all: bool,
    /// The project (a folder with .agents/skills); without it, the one around the current folder
    #[arg(long, value_name = "DIR")]
    pub(crate) project: Option<PathBuf>,
    /// Afterwards update every other project that has these skills (a sync; it asks again)
    #[arg(long, conflicts_with = "json")]
    pub(crate) sync: bool,
    /// Show what would be replaced and replace nothing
    #[arg(long)]
    pub(crate) dry_run: bool,
    /// Replace without asking
    #[arg(short, long)]
    pub(crate) yes: bool,
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
