//! Growing the vault: a new skill from a template. Files are created, never overwritten, and staged with
//! `git add` but never committed. The shared parts (names, groups, rollback, staging) are used by adopt too.

use std::io::Write;
use std::path::PathBuf;

use super::created::{Created, stage};
use crate::Hint;
use crate::vault::{MAX_GROUP_DEPTH, Vault};

#[derive(Debug, thiserror::Error)]
pub enum AuthorError {
    #[error("{name:?} is not a usable skill name")]
    InvalidName { name: String },
    #[error("{group:?} is not a usable group path")]
    InvalidGroup { group: String },
    #[error("skill {name:?} is already in the vault: {existing}")]
    NameTaken { name: String, existing: PathBuf },
    #[error("{name:?} is the name of a group folder of the vault")]
    NameIsGroup { name: String },
    #[error("the group {group:?} would have the same name as the skill {skill}")]
    GroupIsSkill { group: String, skill: PathBuf },
    #[error("a skill may sit at most {max} group folders deep")]
    TooDeep { max: usize },
    #[error("{path} is not a real folder (a file or a link is in the way)")]
    NotARealFolder { path: PathBuf },
    #[error("{path} exists already; nothing was written")]
    Occupied { path: PathBuf },
    #[error("project {project} has no skill folder {name:?}")]
    NotInProject { project: PathBuf, name: String },
    #[error("{path} is a link; skills are never read through links")]
    LinkedParent { path: PathBuf },
    #[error("{path} has no SKILL.md, so it is not a skill")]
    NoSkillFile { path: PathBuf },
    #[error("{path} is a {what}, which a skill cannot hold")]
    UnsupportedEntry { path: PathBuf, what: &'static str },
    #[error("`git add` failed in the vault: {stderr}")]
    GitAdd { stderr: String },
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

impl Hint for AuthorError {
    fn hint(&self) -> Option<String> {
        match self {
            Self::InvalidName { .. } => Some(
                "use lowercase letters, digits, `-` and `_`, as in `doc-start`".to_string(),
            ),
            Self::InvalidGroup { .. } => Some(
                "a group is a vault folder path such as `web` or `web/seo`; `skillmirror skills` shows them"
                    .to_string(),
            ),
            Self::NameTaken { .. } => Some(
                "skill names are unique across all groups; `skillmirror info NAME` shows the existing one"
                    .to_string(),
            ),
            Self::NotInProject { .. } => {
                Some("`skillmirror list` shows what the projects have".to_string())
            }
            Self::GitAdd { .. } => Some(
                "check that the vault is a git repository and does not ignore the new folder".to_string(),
            ),
            Self::TooDeep { .. } => Some("use a shorter group path".to_string()),
            _ => None,
        }
    }
}

/// What a command created in the vault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Authored {
    /// The skill folder.
    pub dir: PathBuf,
    /// The files in it, relative to the folder, sorted. All of them are staged.
    pub files: Vec<PathBuf>,
    /// Files that were written but that git did not take (the vault's `.gitignore` says so); they are not
    /// part of the skill until that is changed.
    pub left_out: Vec<PathBuf>,
}

/// A skill name for a new skill: lowercase letters, digits, `-` and `_`, starting with a letter or a digit.
/// (Adopting accepts any name that a project has; only new names are held to this.)
pub fn check_new_name(name: &str) -> Result<(), AuthorError> {
    let first_ok = name
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
    let rest_ok = name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '_'));
    if first_ok && rest_ok && name.len() <= 64 {
        Ok(())
    } else {
        Err(AuthorError::InvalidName {
            name: name.to_string(),
        })
    }
}

/// The folders of a group path, checked against the vault: plain names, not deeper than allowed, none of
/// them the name of a skill, and none of the existing ones a file or a link.
pub(super) fn group_parts(vault: &Vault, group: &str) -> Result<Vec<String>, AuthorError> {
    let invalid = || AuthorError::InvalidGroup {
        group: group.to_string(),
    };
    let trimmed = group.trim_matches('/');
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let parts: Vec<String> = trimmed.split('/').map(str::to_string).collect();
    if parts
        .iter()
        .any(|p| p.is_empty() || p.starts_with('.') || p.contains(['\\', '\0']))
    {
        return Err(invalid());
    }
    if parts.len() > MAX_GROUP_DEPTH {
        return Err(AuthorError::TooDeep {
            max: MAX_GROUP_DEPTH,
        });
    }
    if let Some(skill) = parts.iter().find_map(|p| vault.skills.get(p)) {
        return Err(AuthorError::GroupIsSkill {
            group: group.to_string(),
            skill: skill.rel.clone(),
        });
    }
    let mut so_far = vault.path.clone();
    for part in &parts {
        so_far.push(part);
        match fs_err::symlink_metadata(&so_far) {
            Ok(meta) if meta.is_dir() => {}
            Ok(_) => return Err(AuthorError::NotARealFolder { path: so_far }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => break,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(parts)
}

/// A name that no skill and no group of the vault has yet.
pub(super) fn check_free(vault: &Vault, name: &str) -> Result<(), AuthorError> {
    if let Some(existing) = vault.skills.get(name) {
        return Err(AuthorError::NameTaken {
            name: name.to_string(),
            existing: existing.rel.clone(),
        });
    }
    if vault
        .groups
        .iter()
        .any(|g| g.iter().any(|part| part.to_str() == Some(name)))
    {
        return Err(AuthorError::NameIsGroup {
            name: name.to_string(),
        });
    }
    Ok(())
}

/// The text of a new `SKILL.md`. The description is always quoted, so any text is a valid header.
fn template(name: &str, description: &str) -> String {
    let mut quoted = String::from("\"");
    for c in description.chars() {
        match c {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '\n' | '\r' => quoted.push(' '),
            c => quoted.push(c),
        }
    }
    quoted.push('"');
    format!(
        "---\nname: {name}\ndescription: {quoted}\n---\n\n# {name}\n\nSay what the agent should do, step by step. Keep this file short; put long reference\nmaterial in separate files next to it.\n"
    )
}

/// What a new skill would get when created: where, and which files.
pub fn new_skill_paths(vault: &Vault, name: &str, group: &str) -> Result<Authored, AuthorError> {
    check_new_name(name)?;
    check_free(vault, name)?;
    let parts = group_parts(vault, group)?;
    let dir: PathBuf = std::iter::once(vault.path.clone())
        .chain(parts.iter().map(PathBuf::from))
        .chain(std::iter::once(PathBuf::from(name)))
        .collect();
    if fs_err::symlink_metadata(&dir).is_ok() {
        return Err(AuthorError::Occupied { path: dir });
    }
    Ok(Authored {
        dir,
        files: vec![PathBuf::from("SKILL.md")],
        left_out: Vec::new(),
    })
}

/// Creates `<vault>/<group>/<name>/SKILL.md` from the template and stages it. Nothing is overwritten and
/// nothing is committed; a failure leaves the vault as it was.
pub fn new_skill(
    vault: &Vault,
    name: &str,
    group: &str,
    description: Option<&str>,
) -> Result<Authored, AuthorError> {
    let planned = new_skill_paths(vault, name, group)?;
    let parts = group_parts(vault, group)?;
    let mut created = Created::default();
    let outcome = (|| {
        created.make_dirs(&vault.path, &parts, name)?;
        let mut file = created.create_file(&planned.dir.join("SKILL.md"))?;
        let text = template(
            name,
            description.unwrap_or("TODO: say what this skill does and when an agent should use it"),
        );
        file.write_all(text.as_bytes())?;
        stage(vault, &planned.dir, &planned.files)
    })();
    match outcome {
        Ok(left_out) => Ok(Authored {
            left_out,
            ..planned
        }),
        Err(e) => {
            created.undo();
            Err(e)
        }
    }
}
