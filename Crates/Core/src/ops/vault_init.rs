//! A new vault: an empty folder that is a git repository and holds a `config.yaml`.

use std::path::{Path, PathBuf};

use super::{NewProject, ProjectError, check_new, create_new};
use crate::config::{ConfigError, ConfigUpdate, VaultConfig, save_config};

#[derive(Debug, thiserror::Error)]
pub enum VaultInitError {
    #[error(transparent)]
    Folder(#[from] ProjectError),
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error("`git add` failed in {path}: {stderr}")]
    GitAdd { path: PathBuf, stderr: String },
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

impl crate::Hint for VaultInitError {
    fn hint(&self) -> Option<String> {
        match self {
            Self::Folder(e) => e.hint(),
            Self::Config(e) => e.hint(),
            _ => None,
        }
    }
}

/// What [`init_vault`] made.
#[derive(Debug)]
pub struct NewVault {
    pub dir: PathBuf,
    /// The configuration file, staged but not committed.
    pub config: PathBuf,
    folder: NewProject,
}

impl NewVault {
    /// Removes what the call made.
    pub fn take_back(&self) -> Result<(), ProjectError> {
        drop(fs_err::remove_file(&self.config));
        self.folder.take_back()
    }
}

/// Checks that `dir` can become a vault: absent, or an empty folder.
pub fn check_new_vault(dir: &Path) -> Result<(), ProjectError> {
    check_new(dir)
}

/// Makes `dir` a vault: creates the folder, `git init`, writes `config.yaml` with the scan root (when
/// given) and an empty mandatory list, and stages the file. Nothing is committed. A failure takes back
/// what was made.
pub fn init_vault(dir: &Path, root: Option<&Path>) -> Result<NewVault, VaultInitError> {
    let folder = create_new(dir, true)?;
    let config = VaultConfig::path_in(dir);
    let vault = NewVault {
        dir: dir.to_path_buf(),
        config,
        folder,
    };
    let update = ConfigUpdate {
        root,
        mandatory: Some(&[]),
    };
    let staged = save_config(dir, &update)
        .map_err(VaultInitError::from)
        .and_then(|()| stage_config(dir, &vault.config));
    match staged {
        Ok(()) => Ok(vault),
        Err(e) => {
            drop(vault.take_back());
            Err(e)
        }
    }
}

fn stage_config(dir: &Path, config: &Path) -> Result<(), VaultInitError> {
    let out = crate::git::command()
        .arg("-C")
        .arg(dir)
        .args(["add", "--"])
        .arg(config.file_name().unwrap_or_default())
        .output()?;
    if out.status.success() {
        Ok(())
    } else {
        Err(VaultInitError::GitAdd {
            path: dir.to_path_buf(),
            stderr: String::from_utf8_lossy(&out.stderr).trim().to_string(),
        })
    }
}
