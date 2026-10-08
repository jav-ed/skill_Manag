//! The text and the JSON of `config show` and the `mandatory` commands.

use std::path::Path;

use serde::Serialize;
use skillmirror_core::config::{Settings, Source, Sourced};

use super::style::{HEADER, MUTED, WARNING};

fn source_name(source: Source) -> &'static str {
    match source {
        Source::Flag => "flag",
        Source::Env => "environment",
        Source::PointerFile => "pointer file",
        Source::VaultConfig => "vault config",
    }
}

#[derive(Debug, Serialize)]
struct Located {
    path: String,
    source: &'static str,
}

fn located(value: Option<&Sourced<std::path::PathBuf>>) -> Option<Located> {
    value.map(|v| Located {
        path: v.value.to_string_lossy().into_owned(),
        source: source_name(v.source),
    })
}

#[derive(Debug, Serialize)]
struct ProfileRow {
    name: String,
    description: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ConfigJson {
    vault: Option<Located>,
    root: Option<Located>,
    config_file: Option<String>,
    mandatory: Vec<String>,
    targets: Vec<String>,
    exclude_dirs: Vec<String>,
    exclude_paths: Vec<String>,
    profiles: Vec<ProfileRow>,
}

impl ConfigJson {
    pub(crate) fn of(settings: &Settings) -> Self {
        let config = settings.config();
        let vault = settings.vault().ok();
        Self {
            vault: located(vault),
            root: located(settings.root().ok()),
            config_file: vault.map(|v| {
                skillmirror_core::config::VaultConfig::path_in(&v.value)
                    .to_string_lossy()
                    .into_owned()
            }),
            mandatory: config.mandatory.clone(),
            targets: config.targets.clone(),
            exclude_dirs: config.exclude_dirs.clone(),
            exclude_paths: config
                .exclude_paths
                .iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect(),
            profiles: config
                .profiles
                .iter()
                .map(|(name, profile)| ProfileRow {
                    name: name.clone(),
                    description: profile.description.clone(),
                })
                .collect(),
        }
    }

    pub(crate) fn render(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

fn label(name: &str) -> String {
    format!("{name:<14}")
}

fn list_or_none(items: &[String]) -> String {
    if items.is_empty() {
        "none".to_string()
    } else {
        items.join(", ")
    }
}

pub(crate) fn render_config(settings: &Settings) -> String {
    let mut out = String::new();
    let config = settings.config();
    if let Ok(vault) = settings.vault() {
        putln!(
            out,
            "{HEADER}{}{HEADER:#}{}  {MUTED}({}){MUTED:#}",
            label("vault"),
            vault.value.display(),
            source_name(vault.source)
        );
    } else {
        putln!(
            out,
            "{WARNING}{}not set{WARNING:#}  {MUTED}(`skillmirror vault init DIR`, or `--vault`, or the Setup screen){MUTED:#}",
            label("vault")
        );
    }
    if let Ok(root) = settings.root() {
        putln!(
            out,
            "{HEADER}{}{HEADER:#}{}  {MUTED}({}){MUTED:#}",
            label("root"),
            root.value.display(),
            source_name(root.source)
        );
    } else {
        putln!(
            out,
            "{WARNING}{}not set{WARNING:#}  {MUTED}(`skillmirror config root DIR`){MUTED:#}",
            label("root")
        );
    }
    if let Ok(vault) = settings.vault() {
        putln!(
            out,
            "{}{}",
            label("config file"),
            skillmirror_core::config::VaultConfig::path_in(&vault.value).display()
        );
    }
    putln!(
        out,
        "{}{}",
        label("mandatory"),
        list_or_none(&config.mandatory)
    );
    putln!(out, "{}{}", label("targets"), list_or_none(&config.targets));
    putln!(
        out,
        "{}{}",
        label("exclude_dirs"),
        list_or_none(&config.exclude_dirs)
    );
    let paths: Vec<String> = config
        .exclude_paths
        .iter()
        .map(|p| p.display().to_string())
        .collect();
    putln!(out, "{}{}", label("exclude_paths"), list_or_none(&paths));
    let profiles: Vec<String> = config.profiles.keys().cloned().collect();
    putln!(out, "{}{}", label("profiles"), list_or_none(&profiles));
    out
}

#[derive(Debug, Serialize)]
pub(crate) struct MandatoryJson<'a> {
    dry_run: bool,
    changed: bool,
    mandatory: &'a [String],
}

impl<'a> MandatoryJson<'a> {
    pub(crate) fn new(dry_run: bool, changed: bool, mandatory: &'a [String]) -> Self {
        Self {
            dry_run,
            changed,
            mandatory,
        }
    }

    pub(crate) fn render(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

pub(crate) fn render_mandatory(
    list: &[String],
    file: &Path,
    changed: bool,
    dry_run: bool,
) -> String {
    let mut out = String::new();
    if list.is_empty() {
        putln!(out, "No mandatory skills.");
    } else {
        for name in list {
            putln!(out, "{name}");
        }
    }
    if dry_run && changed {
        putln!(
            out,
            "{MUTED}(dry run: {} is not changed){MUTED:#}",
            file.display()
        );
    } else if changed {
        putln!(out, "{MUTED}Saved in {}.{MUTED:#}", file.display());
    }
    out
}
