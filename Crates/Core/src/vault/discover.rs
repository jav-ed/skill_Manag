//! Finding the skills in a vault. A skill is a folder that contains `SKILL.md`; any other folder that holds
//! skills is a group, however deep: only the length of a path (the operating system's limit) ends the nesting.

use std::collections::{BTreeMap, BTreeSet};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use super::VaultError;

/// File that marks a folder as a skill.
pub(super) const SKILL_FILE: &str = "SKILL.md";

/// One skill of the vault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skill {
    /// Folder name; unique across the whole vault and equal to the installed folder name.
    pub name: String,
    /// Folders between the vault and the skill, outermost first. Empty for a top-level skill.
    pub group: Vec<String>,
    /// Absolute path of the skill folder.
    pub dir: PathBuf,
    /// Path of the skill folder relative to the vault.
    pub rel: PathBuf,
}

/// Why a vault entry was not taken as a skill or a group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IgnoredReason {
    Symlink,
    NoSkillInside,
}

/// A directory the discovery looked at and left out, kept so `doctor` can show it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ignored {
    pub path: PathBuf,
    pub reason: IgnoredReason,
}

/// Every skill of a vault.
#[derive(Debug, Clone)]
pub struct Vault {
    pub path: PathBuf,
    pub skills: BTreeMap<String, Skill>,
    /// Group paths relative to the vault, such as `web` and `web/seo`.
    pub groups: BTreeSet<PathBuf>,
    pub ignored: Vec<Ignored>,
}

/// Reads the vault folder tree. Dot-directories, files and symlinks are never skills; descent stops at the first `SKILL.md`.
pub fn discover(vault: &Path) -> Result<Vault, VaultError> {
    match fs_err::metadata(vault) {
        Ok(meta) if meta.is_dir() => {}
        Ok(_) => {
            return Err(VaultError::NotADirectory {
                path: vault.to_path_buf(),
            });
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(VaultError::Missing {
                path: vault.to_path_buf(),
            });
        }
        Err(e) => return Err(e.into()),
    }
    let mut found = Vault {
        path: vault.to_path_buf(),
        skills: BTreeMap::new(),
        groups: BTreeSet::new(),
        ignored: Vec::new(),
    };
    visit(vault, &mut Vec::new(), &mut found)?;
    check_group_names(&found)?;
    Ok(found)
}

/// Returns true when `dir` or anything below it holds a skill.
fn visit(dir: &Path, group: &mut Vec<String>, found: &mut Vault) -> Result<bool, VaultError> {
    let mut entries: Vec<_> = fs_err::read_dir(dir)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(fs_err::DirEntry::file_name);
    let mut any = false;
    for entry in entries {
        let file_name = entry.file_name();
        if file_name.as_bytes().first() == Some(&b'.') {
            continue;
        }
        let kind = entry.file_type()?;
        let path = entry.path();
        if kind.is_symlink() {
            found.ignored.push(Ignored {
                path,
                reason: IgnoredReason::Symlink,
            });
            continue;
        }
        if !kind.is_dir() {
            continue;
        }
        let Some(name) = file_name.to_str().map(str::to_owned) else {
            return Err(VaultError::NonUtf8Name { path });
        };
        if path.join(SKILL_FILE).is_file() {
            add_skill(found, name, group, path)?;
            any = true;
            continue;
        }
        group.push(name);
        let has_skill = visit(&path, group, found)?;
        let group_rel: PathBuf = group.iter().collect();
        group.pop();
        if has_skill {
            found.groups.insert(group_rel);
            any = true;
        } else {
            found.ignored.push(Ignored {
                path,
                reason: IgnoredReason::NoSkillInside,
            });
        }
    }
    Ok(any)
}

fn add_skill(
    found: &mut Vault,
    name: String,
    group: &[String],
    dir: PathBuf,
) -> Result<(), VaultError> {
    let rel = group.iter().collect::<PathBuf>().join(&name);
    if let Some(first) = found.skills.get(&name) {
        return Err(VaultError::DuplicateSkill {
            name,
            first: first.rel.clone(),
            second: rel,
        });
    }
    found.skills.insert(
        name.clone(),
        Skill {
            name,
            group: group.to_vec(),
            dir,
            rel,
        },
    );
    Ok(())
}

/// A group named like a skill would make `--group web` and `web` ambiguous.
fn check_group_names(found: &Vault) -> Result<(), VaultError> {
    for group in &found.groups {
        for part in group.iter().filter_map(|p| p.to_str()) {
            if let Some(skill) = found.skills.get(part) {
                return Err(VaultError::GroupSkillClash {
                    group: group.clone(),
                    skill: skill.rel.clone(),
                });
            }
        }
    }
    Ok(())
}
