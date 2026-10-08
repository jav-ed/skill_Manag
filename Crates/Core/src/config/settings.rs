//! Resolution of vault and root: flag, then environment, then files.

use std::path::{Path, PathBuf};

use super::{ConfigError, Dirs, EnvOverrides, VaultConfig, read_pointer};
use crate::scan::ScanOptions;

/// Where a resolved value came from, shown by `doctor` and in error messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Flag,
    Env,
    PointerFile,
    VaultConfig,
}

/// A value together with the layer that supplied it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sourced<T> {
    pub value: T,
    pub source: Source,
}

/// Command-line overrides.
#[derive(Debug, Default, Clone)]
pub struct Flags {
    pub vault: Option<PathBuf>,
    pub root: Option<PathBuf>,
}

/// Fully resolved configuration. Vault and root stay optional so a front end can start the setup wizard.
#[derive(Debug, Clone)]
pub struct Settings {
    vault: Option<Sourced<PathBuf>>,
    root: Option<Sourced<PathBuf>>,
    config: VaultConfig,
}

impl Settings {
    /// Precedence per key: flag, then environment, then (vault) the pointer file or (root) the vault config.
    pub fn load(flags: &Flags, env: &EnvOverrides, dirs: &Dirs) -> Result<Self, ConfigError> {
        let vault = resolve_vault(flags, env, dirs)?;
        let config = match &vault {
            Some(v) => {
                if fs_err::metadata(&v.value).is_ok_and(|m| !m.is_dir()) {
                    return Err(ConfigError::VaultNotADirectory {
                        path: v.value.clone(),
                    });
                }
                VaultConfig::load(&v.value)?
            }
            None => VaultConfig::default(),
        };
        let root = resolve_root(flags, env, &config)?;
        Ok(Self {
            vault,
            root,
            config,
        })
    }

    pub fn vault(&self) -> Result<&Sourced<PathBuf>, ConfigError> {
        self.vault.as_ref().ok_or(ConfigError::VaultMissing)
    }

    pub fn root(&self) -> Result<&Sourced<PathBuf>, ConfigError> {
        self.root.as_ref().ok_or(ConfigError::RootMissing)
    }

    pub fn mandatory(&self) -> &[String] {
        &self.config.mandatory
    }

    pub fn config(&self) -> &VaultConfig {
        &self.config
    }

    pub fn scan_options(&self) -> ScanOptions {
        ScanOptions {
            exclude_dirs: self.config.exclude_dirs.clone(),
            exclude_paths: self.config.exclude_paths.clone(),
        }
    }
}

fn resolve_vault(
    flags: &Flags,
    env: &EnvOverrides,
    dirs: &Dirs,
) -> Result<Option<Sourced<PathBuf>>, ConfigError> {
    if let Some(path) = &flags.vault {
        return Ok(Some(Sourced {
            value: absolute(path)?,
            source: Source::Flag,
        }));
    }
    if let Some(path) = &env.vault {
        return Ok(Some(Sourced {
            value: absolute(path)?,
            source: Source::Env,
        }));
    }
    if let Some(path) = read_pointer(dirs)? {
        return Ok(Some(Sourced {
            value: path,
            source: Source::PointerFile,
        }));
    }
    let old = dirs.legacy_pointer_file();
    if old.exists() {
        return Err(ConfigError::LegacyConfig {
            old,
            new: dirs.pointer_file(),
        });
    }
    Ok(None)
}

fn resolve_root(
    flags: &Flags,
    env: &EnvOverrides,
    config: &VaultConfig,
) -> Result<Option<Sourced<PathBuf>>, ConfigError> {
    if let Some(path) = &flags.root {
        return Ok(Some(Sourced {
            value: absolute(path)?,
            source: Source::Flag,
        }));
    }
    if let Some(path) = &env.root {
        return Ok(Some(Sourced {
            value: absolute(path)?,
            source: Source::Env,
        }));
    }
    match &config.root {
        Some(path) => Ok(Some(Sourced {
            value: absolute(path)?,
            source: Source::VaultConfig,
        })),
        None => Ok(None),
    }
}

/// Makes a path absolute against the current directory without touching symlinks.
fn absolute(path: &Path) -> Result<PathBuf, ConfigError> {
    Ok(std::path::absolute(path)?)
}
