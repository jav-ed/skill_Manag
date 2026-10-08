//! Moving the vault pointer from the old tool's directory to this tool's directory.

use std::path::PathBuf;

use super::{ConfigError, Dirs, read_pointer, write_pointer};

/// What `migrate` found and did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrateReport {
    pub vault: PathBuf,
    pub old_pointer: PathBuf,
    pub new_pointer: PathBuf,
    /// The new pointer already named this vault, so nothing was written.
    pub already_done: bool,
    /// The old pointer file was removed.
    pub retired: bool,
}

/// Copies the old pointer to the new location. Never overwrites a different new pointer.
/// With `retire`, removes the old pointer afterwards so the old tool stops finding the vault.
pub fn migrate(dirs: &Dirs, retire: bool) -> Result<MigrateReport, ConfigError> {
    let old_pointer = dirs.legacy_pointer_file();
    let text = match fs_err::read_to_string(&old_pointer) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(ConfigError::NothingToMigrate { old: old_pointer });
        }
        Err(e) => return Err(e.into()),
    };
    let vault = PathBuf::from(text.trim());
    if vault.as_os_str().is_empty() {
        return Err(ConfigError::PointerEmpty { path: old_pointer });
    }
    if !vault.is_absolute() {
        return Err(ConfigError::NotAbsolute {
            path: vault,
            origin: old_pointer.display().to_string(),
        });
    }
    let already_done = match read_pointer(dirs)? {
        Some(existing) if existing == vault => true,
        Some(existing) => {
            return Err(ConfigError::MigrateConflict {
                old_vault: vault,
                new_vault: existing,
            });
        }
        None => {
            write_pointer(dirs, &vault)?;
            false
        }
    };
    if retire {
        fs_err::remove_file(&old_pointer)?;
        remove_if_empty(old_pointer.parent())?;
    }
    Ok(MigrateReport {
        vault,
        old_pointer,
        new_pointer: dirs.pointer_file(),
        already_done,
        retired: retire,
    })
}

/// Removes the old config directory when the pointer was its only file.
fn remove_if_empty(dir: Option<&std::path::Path>) -> Result<(), ConfigError> {
    let Some(dir) = dir else {
        return Ok(());
    };
    match fs_err::remove_dir(dir) {
        Ok(()) => Ok(()),
        // The directory still holds other files, which is the case to keep.
        Err(e) if e.kind() == std::io::ErrorKind::DirectoryNotEmpty => Ok(()),
        Err(e) => Err(e.into()),
    }
}
