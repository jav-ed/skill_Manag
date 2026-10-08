//! XDG directories of the tool and of the legacy `skill_Manag` tool it replaces.

use std::path::{Path, PathBuf};

use etcetera::base_strategy::{BaseStrategy, Xdg};

use super::ConfigError;
use crate::brand;

/// The old tool hard-coded this directory name below `~/.config`.
const LEGACY_CONFIG_DIR: &str = "skill_Manag";

/// Every directory the tool reads or writes outside a vault and the scan root.
#[derive(Debug, Clone)]
pub struct Dirs {
    home: PathBuf,
    config: PathBuf,
    state: PathBuf,
    cache: PathBuf,
    legacy_config: PathBuf,
}

impl Dirs {
    /// Directories from `XDG_*` variables, falling back to the XDG defaults below `$HOME`.
    pub fn from_env() -> Result<Self, ConfigError> {
        let xdg = Xdg::new().map_err(|e| ConfigError::NoHome(e.to_string()))?;
        let state = xdg
            .state_dir()
            .ok_or_else(|| ConfigError::NoHome("no XDG state directory".to_string()))?;
        Ok(Self {
            home: xdg.home_dir().to_path_buf(),
            config: xdg.config_dir().join(brand::CONFIG_DIR),
            state: state.join(brand::CONFIG_DIR),
            cache: xdg.cache_dir().join(brand::CONFIG_DIR),
            legacy_config: xdg.home_dir().join(".config").join(LEGACY_CONFIG_DIR),
        })
    }

    /// Directories below one base directory. Used by tests and by `--config-home`.
    pub fn under(base: &Path) -> Self {
        Self {
            home: base.to_path_buf(),
            config: base.join("config").join(brand::CONFIG_DIR),
            state: base.join("state").join(brand::CONFIG_DIR),
            cache: base.join("cache").join(brand::CONFIG_DIR),
            legacy_config: base.join("config").join(LEGACY_CONFIG_DIR),
        }
    }

    /// The home directory, where file pickers start.
    pub fn home(&self) -> &Path {
        &self.home
    }

    pub fn config(&self) -> &Path {
        &self.config
    }

    pub fn state(&self) -> &Path {
        &self.state
    }

    pub fn cache(&self) -> &Path {
        &self.cache
    }

    /// File that holds the path of the vault.
    pub fn pointer_file(&self) -> PathBuf {
        self.config.join("vault")
    }

    /// The vault pointer file of the old tool.
    pub fn legacy_pointer_file(&self) -> PathBuf {
        self.legacy_config.join("vault")
    }
}
