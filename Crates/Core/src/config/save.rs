//! Writing `root` and `mandatory` into `<vault>/config.yaml` by editing the text, so comments, the
//! order of the keys and every other key stay as the user wrote them.

use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use super::ConfigError;
use super::vault_config::VaultConfig;

/// Mode of a config file that did not exist before.
const NEW_FILE_MODE: u32 = 0o644;

/// The keys to change. A key set to `None` stays as it is.
#[derive(Debug, Default, Clone, Copy)]
pub struct ConfigUpdate<'a> {
    pub root: Option<&'a Path>,
    pub mandatory: Option<&'a [String]>,
}

/// Rewrites the named keys in `<vault>/config.yaml`. An existing file that does not parse is an
/// error and stays untouched: the tool never replaces a file it cannot read.
pub fn save_config(vault: &Path, update: &ConfigUpdate<'_>) -> Result<(), ConfigError> {
    let path = VaultConfig::path_in(vault);
    // A config that is a link is edited where it really lives, so the link and the file it points to
    // (often a dotfiles repository) both stay what they are.
    let real = real_path(&path)?;
    let (text, mode) = match fs_err::read_to_string(&real) {
        Ok(text) => {
            VaultConfig::parse(&text, &path)?;
            (text, fs_err::metadata(&real)?.permissions().mode() & 0o777)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (String::new(), NEW_FILE_MODE),
        Err(e) => return Err(e.into()),
    };
    let mut edited = text;
    if let Some(root) = update.root {
        edited = set_key(&edited, "root", &root_line(root)?);
    }
    if let Some(mandatory) = update.mandatory {
        edited = set_key(&edited, "mandatory", &mandatory_block(mandatory));
    }
    let check = |message: &str| ConfigError::InvalidVaultConfig {
        path: path.clone(),
        message: message.to_string(),
    };
    let parsed = VaultConfig::parse(&edited, &path)?;
    if update
        .root
        .is_some_and(|r| parsed.root.as_deref() != Some(r))
    {
        return Err(check("the rewritten file does not hold the new root"));
    }
    if update.mandatory.is_some_and(|m| parsed.mandatory != m) {
        return Err(check(
            "the rewritten file does not hold the new mandatory list",
        ));
    }
    write_atomically(&real, &edited, mode)
}

/// Where the config really is: the path itself, or the target of a link. A link that points nowhere is
/// an error, not a reason to write a new file over it.
fn real_path(path: &Path) -> Result<PathBuf, ConfigError> {
    match fs_err::canonicalize(path) {
        Ok(real) => Ok(real),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            match fs_err::symlink_metadata(path) {
                Ok(_) => Err(ConfigError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("{} is a link to a file that does not exist", path.display()),
                ))),
                Err(_) => Ok(path.to_path_buf()),
            }
        }
        Err(e) => Err(e.into()),
    }
}

fn root_line(root: &Path) -> Result<String, ConfigError> {
    if !root.is_absolute() {
        return Err(ConfigError::NotAbsolute {
            path: root.to_path_buf(),
            origin: "the new scan root".to_string(),
        });
    }
    let text = root.to_str().ok_or_else(|| {
        ConfigError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("the path {} is not valid UTF-8", root.display()),
        ))
    })?;
    Ok(format!("root: {}", quoted(text)))
}

fn mandatory_block(names: &[String]) -> String {
    if names.is_empty() {
        return "mandatory: []".to_string();
    }
    let items: Vec<String> = names.iter().map(|n| format!("  - {}", scalar(n))).collect();
    format!("mandatory:\n{}", items.join("\n"))
}

/// A plain scalar when YAML reads it back as the same string, else a double-quoted one.
fn scalar(text: &str) -> String {
    let plain_chars = text
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
    let first_ok = text.chars().next().is_some_and(|c| c.is_ascii_alphabetic());
    let reserved = matches!(
        text.to_ascii_lowercase().as_str(),
        "y" | "n" | "yes" | "no" | "on" | "off" | "true" | "false" | "null"
    );
    if plain_chars && first_ok && !reserved {
        text.to_string()
    } else {
        quoted(text)
    }
}

fn quoted(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Replaces the top-level `key` (with its indented lines or list items) by `block`, or appends it.
/// A byte order mark and the line endings of the file stay as they are.
fn set_key(text: &str, key: &str, block: &str) -> String {
    let (bom, body) = match text.strip_prefix('\u{feff}') {
        Some(rest) => ("\u{feff}", rest),
        None => ("", text),
    };
    let eol = if body.contains("\r\n") { "\r\n" } else { "\n" };
    let lines: Vec<&str> = body.lines().collect();
    let (before, after) = match lines.iter().position(|l| is_key(l, key)) {
        Some(start) => (start, block_end(&lines, start)),
        None => (lines.len(), lines.len()),
    };
    let mut out: Vec<&str> = lines.iter().take(before).copied().collect();
    out.extend(block.lines());
    out.extend(lines.iter().skip(after).copied());
    format!("{bom}{}{eol}", out.join(eol))
}

/// Whether `line` starts the top-level `key`, written plain or in quotes.
fn is_key(line: &str, key: &str) -> bool {
    [key.to_string(), format!("\"{key}\""), format!("'{key}'")]
        .iter()
        .find_map(|spelled| line.strip_prefix(spelled.as_str()))
        .is_some_and(|rest| rest.trim_start().starts_with(':'))
}

/// The line after the value of the key at `start`: its indented lines and list items, including blank
/// lines and comments between them. Blank lines and comments after the last item belong to what follows.
fn block_end(lines: &[&str], start: usize) -> usize {
    let belongs = |line: &str| {
        line.starts_with(' ') || line.starts_with('\t') || line.starts_with("- ") || line == "-"
    };
    let between = |line: &str| line.trim().is_empty() || line.starts_with('#');
    let mut end = start + 1;
    for (offset, line) in lines.iter().enumerate().skip(start + 1) {
        if belongs(line) {
            end = offset + 1;
        } else if !between(line) {
            break;
        }
    }
    end
}

/// Writes `text` to `target` through a temporary file in the same folder, so a crash leaves the old file.
fn write_atomically(target: &Path, text: &str, mode: u32) -> Result<(), ConfigError> {
    let folder = target.parent().ok_or_else(|| {
        ConfigError::Io(std::io::Error::other(format!(
            "{} has no parent folder",
            target.display()
        )))
    })?;
    let mut tmp = tempfile::NamedTempFile::new_in(folder)?;
    tmp.write_all(text.as_bytes())?;
    tmp.as_file()
        .set_permissions(std::fs::Permissions::from_mode(mode))?;
    tmp.persist(target).map_err(|e| e.error)?;
    Ok(())
}
