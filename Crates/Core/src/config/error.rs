//! Failures while locating or reading configuration.

use std::path::PathBuf;

use crate::Hint;
use crate::brand;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("cannot determine the home directory: {0}")]
    NoHome(String),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("the vault pointer file {path} is empty")]
    PointerEmpty { path: PathBuf },
    #[error("the vault path {path} must be absolute (found in {origin})")]
    NotAbsolute { path: PathBuf, origin: String },
    #[error("invalid vault config {path}: {message}")]
    InvalidVaultConfig { path: PathBuf, message: String },
    #[error("legacy environment variables are set: {}", .vars.join(", "))]
    LegacyEnv { vars: Vec<String> },
    #[error("old configuration found at {old} and none at {new}")]
    LegacyConfig { old: PathBuf, new: PathBuf },
    #[error("nothing to migrate: {old} does not exist")]
    NothingToMigrate { old: PathBuf },
    #[error("the old tool points at {old_vault} but this tool already points at {new_vault}")]
    MigrateConflict {
        old_vault: PathBuf,
        new_vault: PathBuf,
    },
    #[error("the vault path {path} is not a directory")]
    VaultNotADirectory { path: PathBuf },
    #[error("no vault configured")]
    VaultMissing,
    #[error("no scan root configured")]
    RootMissing,
}

impl Hint for ConfigError {
    fn hint(&self) -> Option<String> {
        match self {
            Self::LegacyEnv { vars } => Some(format!(
                "rename each variable: {}",
                vars.iter()
                    .map(|v| replacement_for(v))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
            Self::LegacyConfig { .. } => Some(format!("run `{} migrate`", brand::NAME)),
            Self::MigrateConflict { .. } => Some(format!(
                "decide which vault is right and fix the pointer in {} by hand",
                brand::CONFIG_DIR
            )),
            Self::VaultMissing => Some(format!(
                "pass --vault, set {}, or run the setup wizard",
                brand::ENV_VAULT
            )),
            Self::RootMissing => Some(format!(
                "pass --root, set {}, or add `root:` to <vault>/config.yaml",
                brand::ENV_ROOT
            )),
            _ => None,
        }
    }
}

fn replacement_for(var: &str) -> String {
    let name = var.split('=').next().unwrap_or(var);
    match name {
        "SKILL_MANAG_VAULT" => format!("{name} -> {}", brand::ENV_VAULT),
        "SKILL_MANAG_ROOT" => format!("{name} -> {}", brand::ENV_ROOT),
        other => format!("{other} has no replacement, set the key in <vault>/config.yaml"),
    }
}
