//! The vault pointer file: one line holding the absolute path of the vault.

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use super::{ConfigError, Dirs};

/// Reads the pointer file. `None` means the file does not exist; an empty or relative value is an error.
pub fn read_pointer(dirs: &Dirs) -> Result<Option<PathBuf>, ConfigError> {
    let file = dirs.pointer_file();
    let bytes = match fs_err::read(&file) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    // Bytes, not text: a vault path need not be valid UTF-8.
    let value = bytes.trim_ascii();
    if value.is_empty() {
        return Err(ConfigError::PointerEmpty { path: file });
    }
    let path = PathBuf::from(OsStr::from_bytes(value));
    if !path.is_absolute() {
        return Err(ConfigError::NotAbsolute {
            path,
            origin: file.display().to_string(),
        });
    }
    Ok(Some(path))
}

/// Writes the pointer file atomically, creating the config directory when needed.
pub fn write_pointer(dirs: &Dirs, vault: &Path) -> Result<(), ConfigError> {
    use std::io::Write;

    if !vault.is_absolute() {
        return Err(ConfigError::NotAbsolute {
            path: vault.to_path_buf(),
            origin: "the new vault path".to_string(),
        });
    }
    fs_err::create_dir_all(dirs.config())?;
    let mut tmp = tempfile::NamedTempFile::new_in(dirs.config())?;
    tmp.write_all(vault.as_os_str().as_bytes())?;
    tmp.write_all(b"\n")?;
    tmp.persist(dirs.pointer_file()).map_err(|e| e.error)?;
    Ok(())
}
