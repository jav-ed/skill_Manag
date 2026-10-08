//! Turning names, groups and profiles into a set of vault skills.

use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::Hint;
use crate::config::VaultConfig;
use crate::vault::Vault;

#[derive(Debug, thiserror::Error)]
pub enum SelectError {
    #[error("skill {name:?} is not in the vault")]
    UnknownSkill { name: String },
    #[error("group {path:?} is not a folder of the vault")]
    UnknownGroup { path: String },
    #[error("profile {name:?} is not defined in the vault config")]
    UnknownProfile { name: String },
    #[error("profile {name:?} has the same name as a skill or a group")]
    ProfileNameClash { name: String },
    #[error("nothing selected")]
    Empty,
}

impl Hint for SelectError {
    fn hint(&self) -> Option<String> {
        match self {
            Self::UnknownSkill { .. } => Some("`skillmirror skills` lists the vault".to_string()),
            Self::UnknownGroup { .. } => Some("`skillmirror skills` shows the groups".to_string()),
            Self::UnknownProfile { .. } => {
                Some("define it under `profiles:` in <vault>/config.yaml".to_string())
            }
            Self::Empty => Some("name skills, or pass --group or --profile".to_string()),
            Self::ProfileNameClash { .. } => Some("rename the profile".to_string()),
        }
    }
}

/// What the user asked for on the command line.
#[derive(Debug, Default, Clone)]
pub struct Selection {
    pub skills: Vec<String>,
    pub groups: Vec<String>,
    pub profiles: Vec<String>,
}

impl Selection {
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty() && self.groups.is_empty() && self.profiles.is_empty()
    }
}

/// Expands a selection to skill names. Every name must exist; an empty result is an error.
pub fn resolve(
    vault: &Vault,
    config: &VaultConfig,
    selection: &Selection,
) -> Result<BTreeSet<String>, SelectError> {
    let mut chosen = BTreeSet::new();
    for name in &selection.skills {
        add_skill(vault, name, &mut chosen)?;
    }
    for group in &selection.groups {
        add_group(vault, group, &mut chosen)?;
    }
    for profile in &selection.profiles {
        add_profile(vault, config, profile, &mut chosen)?;
    }
    if chosen.is_empty() {
        Err(SelectError::Empty)
    } else {
        Ok(chosen)
    }
}

/// Expands a configured list of skill names, for example `mandatory`, without allowing unknown names.
pub fn resolve_names(vault: &Vault, names: &[String]) -> Result<BTreeSet<String>, SelectError> {
    let mut chosen = BTreeSet::new();
    for name in names {
        add_skill(vault, name, &mut chosen)?;
    }
    Ok(chosen)
}

fn add_skill(vault: &Vault, name: &str, chosen: &mut BTreeSet<String>) -> Result<(), SelectError> {
    if !vault.skills.contains_key(name) {
        return Err(SelectError::UnknownSkill {
            name: name.to_string(),
        });
    }
    chosen.insert(name.to_string());
    Ok(())
}

/// A group is a vault folder path such as `web` or `web/seo`; it brings every skill below it.
fn add_group(vault: &Vault, group: &str, chosen: &mut BTreeSet<String>) -> Result<(), SelectError> {
    let path: PathBuf = group.trim_matches('/').split('/').collect();
    if !vault.groups.contains(&path) {
        return Err(SelectError::UnknownGroup {
            path: group.to_string(),
        });
    }
    chosen.extend(
        vault
            .skills
            .values()
            .filter(|s| s.rel.starts_with(&path))
            .map(|s| s.name.clone()),
    );
    Ok(())
}

fn add_profile(
    vault: &Vault,
    config: &VaultConfig,
    name: &str,
    chosen: &mut BTreeSet<String>,
) -> Result<(), SelectError> {
    let mut members = BTreeSet::new();
    collect_profile(vault, config, name, &mut members)?;
    chosen.extend(members);
    Ok(())
}

/// The config loader already rejected loops, so recursion ends.
fn collect_profile(
    vault: &Vault,
    config: &VaultConfig,
    name: &str,
    out: &mut BTreeSet<String>,
) -> Result<(), SelectError> {
    let profile = config
        .profiles
        .get(name)
        .ok_or_else(|| SelectError::UnknownProfile {
            name: name.to_string(),
        })?;
    if vault.skills.contains_key(name) || vault.groups.iter().any(|g| g.to_str() == Some(name)) {
        return Err(SelectError::ProfileNameClash {
            name: name.to_string(),
        });
    }
    for parent in &profile.extends {
        collect_profile(vault, config, parent, out)?;
    }
    for group in &profile.groups {
        add_group(vault, group, out)?;
    }
    for skill in &profile.skills {
        add_skill(vault, skill, out)?;
    }
    for skill in &profile.exclude {
        add_skill(vault, skill, &mut BTreeSet::new())?;
        out.remove(skill);
    }
    Ok(())
}
