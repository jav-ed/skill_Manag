//! Environment overrides. Only the vault and the root can come from the environment.

use std::ffi::OsString;
use std::path::PathBuf;

use super::ConfigError;
use crate::brand;

const LEGACY_PREFIX: &str = "SKILL_MANAG_";

/// Values of `SKILLMIRROR_VAULT` and `SKILLMIRROR_ROOT`. An empty variable counts as unset.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EnvOverrides {
    pub vault: Option<PathBuf>,
    pub root: Option<PathBuf>,
}

impl EnvOverrides {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_pairs(std::env::vars_os())
    }

    /// Reads the overrides from explicit pairs, so tests never touch the real environment.
    /// Any variable of the old tool is a hard error: silently ignoring it could sync the wrong vault.
    pub fn from_pairs(
        pairs: impl IntoIterator<Item = (OsString, OsString)>,
    ) -> Result<Self, ConfigError> {
        let mut found = Self::default();
        let mut legacy = Vec::new();
        for (key, value) in pairs {
            let key = key.to_string_lossy().into_owned();
            if key.starts_with(LEGACY_PREFIX) {
                legacy.push(key);
            } else if key == brand::ENV_VAULT && !value.is_empty() {
                found.vault = Some(PathBuf::from(value));
            } else if key == brand::ENV_ROOT && !value.is_empty() {
                found.root = Some(PathBuf::from(value));
            }
        }
        if legacy.is_empty() {
            return Ok(found);
        }
        legacy.sort();
        Err(ConfigError::LegacyEnv { vars: legacy })
    }
}
