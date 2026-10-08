//! Command-line grammar.
// Doc comments here become `--help` text, where environment variable names must stay unquoted.
#![allow(clippy::doc_markdown)]

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use clap_complete::Shell;

pub(crate) use crate::args_more::{
    AdoptArgs, ConfigCommand, ConfigRootArgs, InfoArgs, MandatoryCommand, MandatoryEditArgs,
    NewArgs, ScopeArgs, VaultCommand, VaultInitArgs, WebArgs,
};

#[derive(Debug, Parser)]
#[command(
    name = "skillmirror",
    version,
    about = "Mirror agent skills from one vault into every project that uses them",
    long_about = "skillmirror copies skill folders from a git-tracked vault into every project's .agents/skills/ \
                  directory. Files are copied, never symlinked, so projects stay git-tracked and work over SSH.\n\n\
                  The vault comes from --vault, SKILLMIRROR_VAULT or the pointer file in ~/.config/skillmirror. \
                  The scan root comes from --root, SKILLMIRROR_ROOT or `root:` in <vault>/config.yaml."
)]
pub(crate) struct Cli {
    /// Skill vault directory (overrides SKILLMIRROR_VAULT and the pointer file)
    #[arg(long, global = true, value_name = "DIR")]
    pub(crate) vault: Option<PathBuf>,
    /// Directory to scan for projects (overrides SKILLMIRROR_ROOT and the vault config)
    #[arg(long, global = true, value_name = "DIR")]
    pub(crate) root: Option<PathBuf>,
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Open the interactive interface (the default in a terminal)
    Tui,
    /// Update the skills each project already has from the vault; never adds a skill
    Sync(ApplyArgs),
    /// Install the mandatory skills into every project that has a skills directory
    Push(ApplyArgs),
    /// List every installed skill
    List(ListArgs),
    /// Remove one skill from projects
    Delete(DeleteArgs),
    /// Link the agent folders named in the vault config (`targets:`) to .agents/skills in every project
    Bridge(BridgeArgs),
    /// Show how every project stands against the vault; writes nothing, exit 1 when something differs
    Status(StatusArgs),
    /// Show the lines a sync would bring in and take away; writes nothing, exit 1 when something differs
    Diff(DiffArgs),
    /// Check the vault, the configuration and the machine; writes nothing, exit 1 for warnings, 3 for errors
    Doctor(DoctorArgs),
    /// Write one self-contained HTML file: skills against projects, the vault tree, diffs, a filter
    Report(ReportArgs),
    /// Serve the report live on this computer only; with --allow-write also sync, push and undo from the browser
    Web(WebArgs),
    /// Show the skills in the vault, grouped by folder
    Skills(SkillsArgs),
    /// Show one skill: its place and files in the vault, the profiles that name it, and what it is in each project
    Info(InfoArgs),
    /// Create a skill in the vault from a template (staged in git, never committed)
    New(NewArgs),
    /// Copy a skill folder that a project has into the vault (staged in git, never committed)
    Adopt(AdoptArgs),
    /// Work on the vault itself
    #[command(subcommand)]
    Vault(VaultCommand),
    /// Show or change the configuration
    #[command(subcommand)]
    Config(ConfigCommand),
    /// Show or change the skills that `push` installs everywhere
    #[command(subcommand)]
    Mandatory(MandatoryCommand),
    /// Install skills, groups or profiles into an existing project
    Add(AddArgs),
    /// Create a new project directory and install the mandatory skills plus a selection
    Init(InitArgs),
    /// Bring back what a run replaced or removed (undoing is a run too, so a second undo redoes it)
    Undo(UndoArgs),
    /// List the runs that can be undone, newest first
    History(HistoryArgs),
    /// Copy the vault pointer of the old skill_Manag tool to this tool
    Migrate(MigrateArgs),
    /// Print a shell completion script
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
}

/// Flags shared by `sync` and `push`.
// Independent command-line switches, not a state machine.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Args)]
pub(crate) struct ApplyArgs {
    #[command(flatten)]
    pub(crate) scope: ScopeArgs,
    /// Show what would change and write nothing
    #[arg(long, conflicts_with = "check")]
    pub(crate) dry_run: bool,
    /// Write nothing; exit 1 when any target differs from the vault
    #[arg(long)]
    pub(crate) check: bool,
    /// Apply without asking
    #[arg(short, long)]
    pub(crate) yes: bool,
    /// Print one JSON document instead of text
    #[arg(long)]
    pub(crate) json: bool,
    /// Also list targets that are already up to date
    #[arg(long)]
    pub(crate) all: bool,
}

#[derive(Debug, Args)]
pub(crate) struct ListArgs {
    /// Print one JSON document instead of text
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct ReportArgs {
    /// Where to write the report; `-` prints it. An existing file is replaced only when it is a report
    /// this tool wrote.
    #[arg(
        short,
        long,
        value_name = "FILE",
        default_value = "skillmirror-report.html"
    )]
    pub(crate) output: PathBuf,
    /// Open the report in the browser afterwards (xdg-open)
    #[arg(long)]
    pub(crate) open: bool,
}

// The flags mirror independent command-line switches.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Args)]
pub(crate) struct BridgeArgs {
    /// Show what would be linked and write nothing
    #[arg(long)]
    pub(crate) dry_run: bool,
    /// Link without asking
    #[arg(short, long)]
    pub(crate) yes: bool,
    /// Also list the bridges that are already in place
    #[arg(long)]
    pub(crate) all: bool,
    /// Print one JSON document instead of text
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct StatusArgs {
    #[command(flatten)]
    pub(crate) scope: ScopeArgs,
    /// Also list the projects that are fully up to date, with their skills
    #[arg(long)]
    pub(crate) all: bool,
    /// Print one JSON document instead of text
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct DiffArgs {
    /// Only this skill
    #[arg(value_name = "SKILL")]
    pub(crate) skill: Option<String>,
    /// Only this project
    #[arg(long, value_name = "DIR")]
    pub(crate) project: Option<PathBuf>,
    /// Only the lines added and removed per file, not the lines themselves
    #[arg(long)]
    pub(crate) stat: bool,
    /// Print one JSON document instead of text
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct DoctorArgs {
    /// Print one JSON document instead of text
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct DeleteArgs {
    /// Name of the skill folder to remove
    pub(crate) name: String,
    /// Remove it from this one project instead of scanning the root
    #[arg(long, value_name = "DIR")]
    pub(crate) project: Option<PathBuf>,
    /// Show what would be removed and remove nothing
    #[arg(long)]
    pub(crate) dry_run: bool,
    /// Remove without asking
    #[arg(short, long)]
    pub(crate) yes: bool,
    /// Print one JSON document instead of text
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct UndoArgs {
    /// The run to undo (default: the newest one); `history` lists them
    #[arg(value_name = "RUN")]
    pub(crate) run: Option<String>,
    /// Only undo what the run did in this project
    #[arg(long, value_name = "DIR")]
    pub(crate) project: Option<PathBuf>,
    /// Only undo what the run did to this skill
    #[arg(long, value_name = "NAME")]
    pub(crate) skill: Option<String>,
    /// Show what would be brought back and change nothing
    #[arg(long)]
    pub(crate) dry_run: bool,
    /// Undo without asking
    #[arg(short, long)]
    pub(crate) yes: bool,
    /// Print one JSON document instead of text
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct HistoryArgs {
    /// Print one JSON document instead of text
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct MigrateArgs {
    /// Remove the old pointer file afterwards, so the old tool stops finding the vault
    #[arg(long)]
    pub(crate) retire: bool,
}

/// Which skills to install: names, vault group folders and profiles from the vault config.
#[derive(Debug, Args)]
pub(crate) struct SelectArgs {
    /// Skill names
    #[arg(value_name = "SKILL")]
    pub(crate) skills: Vec<String>,
    /// A vault folder such as `web` or `web/seo`; installs every skill below it
    #[arg(long = "group", value_name = "PATH")]
    pub(crate) groups: Vec<String>,
    /// A profile defined under `profiles:` in <vault>/config.yaml
    #[arg(long = "profile", value_name = "NAME")]
    pub(crate) profiles: Vec<String>,
}

/// Flags shared by `add` and `init`.
#[derive(Debug, Args)]
pub(crate) struct InstallFlags {
    /// Show what would be installed and write nothing
    #[arg(long)]
    pub(crate) dry_run: bool,
    /// Install without asking
    #[arg(short, long)]
    pub(crate) yes: bool,
    /// Print one JSON document instead of text
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct SkillsArgs {
    /// Only the skills below this vault folder
    #[arg(long, value_name = "PATH")]
    pub(crate) group: Option<String>,
    /// Print one JSON document instead of text
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct AddArgs {
    #[command(flatten)]
    pub(crate) select: SelectArgs,
    /// The project to install into (default: the current directory)
    #[arg(long, value_name = "DIR")]
    pub(crate) project: Option<PathBuf>,
    #[command(flatten)]
    pub(crate) flags: InstallFlags,
}

#[derive(Debug, Args)]
pub(crate) struct InitArgs {
    /// The new project directory; it must not exist or must be empty
    pub(crate) dir: PathBuf,
    #[command(flatten)]
    pub(crate) select: SelectArgs,
    /// Also run `git init` in the new directory
    #[arg(long)]
    pub(crate) git: bool,
    /// Do not install the vault's mandatory skills
    #[arg(long)]
    pub(crate) no_mandatory: bool,
    #[command(flatten)]
    pub(crate) flags: InstallFlags,
}
