//! Writing `root` and `mandatory` into `<vault>/config.yaml` by editing the text, so comments, the
//! order of the keys and every other key stay as the user wrote them.

use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use super::ConfigError;
use super::vault_config::{FILE_NAME, VaultConfig};

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
    let (text, mode) = match fs_err::read_to_string(&path) {
        Ok(text) => {
            VaultConfig::parse(&text, &path)?;
            (text, fs_err::metadata(&path)?.permissions().mode() & 0o777)
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
    write_atomically(vault, &edited, mode)
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
fn set_key(text: &str, key: &str, block: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let is_key = |line: &str| {
        line.strip_prefix(key)
            .is_some_and(|rest| rest.starts_with(':'))
    };
    let Some(start) = lines.iter().position(|l| is_key(l)) else {
        let mut out = text.to_string();
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(block);
        out.push('\n');
        return out;
    };
    let belongs = |line: &str| {
        line.starts_with(' ') || line.starts_with('\t') || line.starts_with("- ") || line == "-"
    };
    let following = lines
        .iter()
        .skip(start + 1)
        .take_while(|l| belongs(l))
        .count();
    let end = start + 1 + following;
    let mut out: Vec<&str> = lines.iter().take(start).copied().collect();
    out.push(block);
    out.extend(lines.iter().skip(end).copied());
    let mut joined = out.join("\n");
    joined.push('\n');
    joined
}

fn write_atomically(vault: &Path, text: &str, mode: u32) -> Result<(), ConfigError> {
    let mut tmp = tempfile::NamedTempFile::new_in(vault)?;
    tmp.write_all(text.as_bytes())?;
    tmp.as_file()
        .set_permissions(std::fs::Permissions::from_mode(mode))?;
    tmp.persist(vault.join(FILE_NAME)).map_err(|e| e.error)?;
    Ok(())
}
