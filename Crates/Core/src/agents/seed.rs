//! Writing the built-in text into the vault, so that it can be edited there.

use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use super::AgentsError;
use super::text::{Source, VAULT_FOLDER, vault_text_path};

/// Where [`seed_vault_text`] writes, or the error it would stop with. Writes nothing.
pub fn plan_seed(vault: &Path) -> Result<PathBuf, AgentsError> {
    let path = vault_text_path(vault);
    match fs_err::symlink_metadata(&path) {
        Ok(_) => Err(AgentsError::Exists { path }),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(path),
        Err(e) => Err(e.into()),
    }
}

/// Writes the built-in text to `<vault>/project-files/AGENTS.md`. A file that is there is never touched,
/// and a failure half way leaves the vault as it was. Nothing is staged or committed.
pub fn seed_vault_text(vault: &Path) -> Result<PathBuf, AgentsError> {
    let path = plan_seed(vault)?;
    let folder = vault.join(VAULT_FOLDER);
    let made_folder = match fs_err::create_dir(&folder) {
        Ok(()) => true,
        Err(e) if e.kind() == ErrorKind::AlreadyExists => false,
        Err(e) => return Err(e.into()),
    };
    let take_back_folder = || {
        if made_folder {
            drop(fs_err::remove_dir(&folder));
        }
    };
    let mut file = match fs_err::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(file) => file,
        Err(e) => {
            take_back_folder();
            // Somebody made it between the check and now: still not ours to touch.
            return Err(if e.kind() == ErrorKind::AlreadyExists {
                AgentsError::Exists { path }
            } else {
                e.into()
            });
        }
    };
    let text = format!("{}\n", Source::builtin().text());
    if let Err(e) = file.write_all(text.as_bytes()) {
        drop(file);
        drop(fs_err::remove_file(&path));
        take_back_folder();
        return Err(e.into());
    }
    Ok(path)
}
