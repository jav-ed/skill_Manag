//! What `new`, `adopt` and `vault init` print.

use std::path::{Path, PathBuf};

use serde::Serialize;

use super::human::plural;
use super::style::{HEADER, MUTED, NAME, WARNING};

/// What happened to the default-vault pointer when a vault was created.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Pointer {
    /// Written: the new vault is now the default.
    Written,
    /// It already pointed here.
    AlreadyHere,
    /// It points to another vault and was left alone.
    KeptOther,
}

#[derive(Debug, Serialize)]
pub(crate) struct AuthoredJson<'a> {
    command: &'static str,
    dry_run: bool,
    skill: &'a str,
    #[serde(serialize_with = "super::lossy::path")]
    dir: &'a Path,
    files: Vec<String>,
    /// Files that git did not take; they are not part of the skill.
    left_out: Vec<String>,
}

impl<'a> AuthoredJson<'a> {
    pub(crate) fn new(
        command: &'static str,
        dry_run: bool,
        skill: &'a str,
        dir: &'a Path,
        files: &[PathBuf],
        left_out: &[PathBuf],
    ) -> Self {
        let names = |paths: &[PathBuf]| paths.iter().map(|p| p.display().to_string()).collect();
        Self {
            command,
            dry_run,
            skill,
            dir,
            files: names(files),
            left_out: names(left_out),
        }
    }

    pub(crate) fn render(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// The text after a skill was created or adopted, or would be (`dry_run`).
pub(crate) fn render_authored(
    verb: &str,
    skill: &str,
    dir: &Path,
    files: &[PathBuf],
    left_out: &[PathBuf],
    dry_run: bool,
) -> String {
    let mut out = String::new();
    let tense = if dry_run { "Would create" } else { "Created" };
    putln!(
        out,
        "{HEADER}{tense}{HEADER:#} {NAME}{skill}{NAME:#} in the vault ({verb}): {}",
        dir.display()
    );
    for file in files {
        putln!(out, "  {}", file.display());
    }
    if !left_out.is_empty() {
        putln!(
            out,
            "{WARNING}{} not taken by git (the vault's .gitignore?): {}{WARNING:#}",
            plural(left_out.len(), "file"),
            left_out
                .iter()
                .map(|f| f.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if !dry_run {
        putln!(
            out,
            "{MUTED}Staged in git, not committed. Commit it in the vault when it is ready; sync and add copy what git tracks.{MUTED:#}"
        );
    }
    out
}

#[derive(Debug, Serialize)]
pub(crate) struct VaultInitJson<'a> {
    dry_run: bool,
    #[serde(serialize_with = "super::lossy::path")]
    vault: &'a Path,
    root: Option<String>,
    /// `null` on a dry run.
    pointer: Option<Pointer>,
}

impl<'a> VaultInitJson<'a> {
    pub(crate) fn new(
        dry_run: bool,
        vault: &'a Path,
        root: Option<&'a Path>,
        pointer: Option<Pointer>,
    ) -> Self {
        Self {
            dry_run,
            vault,
            root: root.map(|r| r.to_string_lossy().into_owned()),
            pointer,
        }
    }

    pub(crate) fn render(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

pub(crate) fn render_vault_init(
    vault: &Path,
    root: Option<&Path>,
    pointer: Option<Pointer>,
    other: Option<&Path>,
) -> String {
    let mut out = String::new();
    let dry = pointer.is_none();
    putln!(
        out,
        "{HEADER}{}{HEADER:#} the vault {}",
        if dry { "Would create" } else { "Created" },
        vault.display()
    );
    putln!(
        out,
        "  a git repository with a config.yaml{}",
        if dry { "" } else { " (staged, not committed)" }
    );
    if let Some(root) = root {
        putln!(out, "  scan root   {}", root.display());
    } else {
        putln!(
            out,
            "  {WARNING}no scan root yet{WARNING:#}  set it in {}/config.yaml (`root: /path/to/projects`) or pass --root",
            vault.display()
        );
    }
    match pointer {
        Some(Pointer::Written) => putln!(out, "  default     this is now your default vault"),
        Some(Pointer::AlreadyHere) => putln!(out, "  default     already your default vault"),
        Some(Pointer::KeptOther) => putln!(
            out,
            "  {WARNING}default     still {}{WARNING:#}  (pass --use to switch to the new vault)",
            other.map(|p| p.display().to_string()).unwrap_or_default()
        ),
        None => {}
    }
    if !dry {
        putln!(
            out,
            "{MUTED}Next: `skillmirror new NAME` makes a skill, `skillmirror adopt NAME --from PROJECT` takes one a project already has.{MUTED:#}"
        );
    }
    out
}
