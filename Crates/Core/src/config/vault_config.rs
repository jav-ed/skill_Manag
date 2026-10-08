//! `<vault>/config.yaml`: scan root, mandatory skills and scan exclusions.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::ConfigError;

/// Name of the config file inside the vault.
pub(super) const FILE_NAME: &str = "config.yaml";

/// Contents of the vault config. Unknown keys are a hard error, a missing file is an empty config.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct VaultConfig {
    pub root: Option<PathBuf>,
    pub mandatory: Vec<String>,
    pub exclude_dirs: Vec<String>,
    pub exclude_paths: Vec<PathBuf>,
    /// Named selections of skills for `init` and `add`.
    pub profiles: BTreeMap<String, Profile>,
}

/// A named selection: groups (vault folder paths), skills (names) and other profiles, minus `exclude`.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Profile {
    pub description: Option<String>,
    pub extends: Vec<String>,
    pub groups: Vec<String>,
    pub skills: Vec<String>,
    pub exclude: Vec<String>,
}

impl VaultConfig {
    pub fn path_in(vault: &Path) -> PathBuf {
        vault.join(FILE_NAME)
    }

    /// Reads and validates the config of `vault`. A missing file gives the default config.
    pub fn load(vault: &Path) -> Result<Self, ConfigError> {
        let path = Self::path_in(vault);
        let text = match fs_err::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(e.into()),
        };
        Self::parse(&text, &path)
    }

    pub(crate) fn parse(text: &str, path: &Path) -> Result<Self, ConfigError> {
        let invalid = |message: String| ConfigError::InvalidVaultConfig {
            path: path.to_path_buf(),
            message,
        };
        // A file with only comments is an empty document, not a null that the struct would reject.
        let parsed: Option<Self> =
            serde_saphyr::from_str(text).map_err(|e| invalid(e.to_string()))?;
        let config = parsed.unwrap_or_default();
        config.validate().map_err(invalid)?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), String> {
        let mut seen = std::collections::BTreeSet::new();
        for name in &self.mandatory {
            if name.is_empty() || name.contains('/') || name.starts_with('.') {
                return Err(format!("mandatory entry {name:?} is not a skill name"));
            }
            if !seen.insert(name) {
                return Err(format!("mandatory lists {name:?} twice"));
            }
        }
        if let Some(bad) = self
            .exclude_dirs
            .iter()
            .find(|d| d.is_empty() || d.contains('/'))
        {
            return Err(format!(
                "exclude_dirs entry {bad:?} must be a bare directory name"
            ));
        }
        if self.root.as_ref().is_some_and(|r| r.as_os_str().is_empty()) {
            return Err("root must not be empty".to_string());
        }
        if self.exclude_paths.iter().any(|p| p.as_os_str().is_empty()) {
            return Err("exclude_paths has an empty entry".to_string());
        }
        self.validate_profiles()
    }

    /// Structure only: names are plain, `extends` points at existing profiles and never loops.
    /// Whether skills and groups exist is checked against the vault when a profile is resolved.
    fn validate_profiles(&self) -> Result<(), String> {
        for (name, profile) in &self.profiles {
            if name.is_empty() || name.contains('/') || name.starts_with('.') {
                return Err(format!("profile name {name:?} is not a plain name"));
            }
            if let Some(unknown) = profile
                .extends
                .iter()
                .find(|e| !self.profiles.contains_key(*e))
            {
                return Err(format!(
                    "profile {name:?} extends {unknown:?}, which is not defined"
                ));
            }
            self.check_acyclic(name, &mut vec![name.as_str()])?;
        }
        Ok(())
    }

    fn check_acyclic<'a>(&'a self, name: &str, path: &mut Vec<&'a str>) -> Result<(), String> {
        let Some(profile) = self.profiles.get(name) else {
            return Ok(());
        };
        for parent in &profile.extends {
            if path.contains(&parent.as_str()) {
                return Err(format!(
                    "profiles extend each other in a loop: {} -> {parent}",
                    path.join(" -> ")
                ));
            }
            path.push(parent);
            self.check_acyclic(parent, path)?;
            path.pop();
        }
        Ok(())
    }
}
