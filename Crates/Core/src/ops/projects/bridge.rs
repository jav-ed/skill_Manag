//! Bridges: a link from another agent's folder (such as `.claude/skills`) to `.agents/skills`, so that
//! agent sees the same skills. A bridge is created only where nothing is. A real folder, a link that points
//! elsewhere and a link above the place are reported and left alone.

use std::path::{Path, PathBuf};

use crate::config::KNOWN_TARGETS;
use crate::scan::SkillsDir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeState {
    /// Nothing is there; `create` makes the link.
    Missing,
    InPlace,
    /// A link that points somewhere else.
    Elsewhere(PathBuf),
    /// A real folder or file: what it holds is not ours to replace.
    Occupied,
    /// A folder above the place (for example `.claude`) is itself a link, and nothing is written through it.
    ParentIsLink(PathBuf),
}

impl BridgeState {
    /// Whether the user has something to do about it.
    pub fn is_problem(&self) -> bool {
        !matches!(self, Self::Missing | Self::InPlace)
    }
}

#[derive(Debug, Clone)]
pub struct Bridge {
    pub project: PathBuf,
    /// The name used in `targets:`, such as `claude`.
    pub name: String,
    /// The link, `<project>/.claude/skills`.
    pub link: PathBuf,
    /// What the link points at, relative to the folder it is in, such as `../.agents/skills`.
    pub points_to: PathBuf,
    pub state: BridgeState,
}

impl Bridge {
    /// A path below the project as it is written inside the project, such as `.claude/skills`.
    pub fn short(&self, path: &Path) -> PathBuf {
        path.strip_prefix(&self.project)
            .unwrap_or(path)
            .to_path_buf()
    }

    /// What is in the way and what to do about it, for a state that needs the user.
    pub fn problem(&self) -> Option<(String, String)> {
        let link = self.short(&self.link).display().to_string();
        match &self.state {
            BridgeState::Missing | BridgeState::InPlace => None,
            BridgeState::Elsewhere(found) => Some((
                format!(
                    "{link} is a link to {}, not to {}",
                    found.display(),
                    self.points_to.display()
                ),
                "remove the link or point it at ../.agents/skills yourself".to_string(),
            )),
            BridgeState::Occupied => Some((
                format!("{link} is a real folder or file, so no bridge is made there"),
                "move what it holds into .agents/skills, then remove it".to_string(),
            )),
            BridgeState::ParentIsLink(parent) => Some((
                format!(
                    "{} is a link, so nothing is written through it",
                    self.short(parent).display()
                ),
                "make it a real folder, or drop the target from the vault config".to_string(),
            )),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("{link} is not free ({why}), so no bridge is made there")]
    NotFree { link: PathBuf, why: String },
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

/// One bridge per project and per named target, with its state right now.
pub fn plan(targets: &[String], dirs: &[SkillsDir]) -> Vec<Bridge> {
    dirs.iter()
        .flat_map(|dir| plan_project(targets, &dir.project))
        .collect()
}

/// The bridges of one project.
pub fn plan_project(targets: &[String], project: &Path) -> Vec<Bridge> {
    targets
        .iter()
        .filter_map(|name| {
            let (_, folder) = KNOWN_TARGETS.iter().find(|(known, _)| known == name)?;
            let link = project.join(folder);
            let depth = Path::new(folder).components().count().saturating_sub(1);
            // Each folder between the project and the link is one step up: `.claude/skills` is one deep.
            let mut points_to = PathBuf::new();
            for _ in 0..depth {
                points_to.push("..");
            }
            points_to.push(".agents");
            points_to.push("skills");
            let state = state_of(project, &link, &points_to);
            Some(Bridge {
                project: project.to_path_buf(),
                name: name.clone(),
                link,
                points_to,
                state,
            })
        })
        .collect()
}

fn state_of(project: &Path, link: &Path, points_to: &Path) -> BridgeState {
    if let Some(parent) = link.parent().filter(|p| *p != project)
        && std::fs::symlink_metadata(parent).is_ok_and(|m| m.file_type().is_symlink())
    {
        return BridgeState::ParentIsLink(parent.to_path_buf());
    }
    match std::fs::symlink_metadata(link) {
        Err(_) => BridgeState::Missing,
        Ok(meta) if meta.file_type().is_symlink() => match std::fs::read_link(link) {
            Ok(found) if found == points_to => BridgeState::InPlace,
            Ok(found) => {
                // A link written another way (absolute, or through another route) may still lead to the same folder.
                let same = match (
                    link.canonicalize(),
                    project.join(".agents/skills").canonicalize(),
                ) {
                    (Ok(through_link), Ok(skills)) => through_link == skills,
                    _ => false,
                };
                if same {
                    BridgeState::InPlace
                } else {
                    BridgeState::Elsewhere(found)
                }
            }
            Err(_) => BridgeState::Occupied,
        },
        Ok(_) => BridgeState::Occupied,
    }
}

/// Makes the link, but only if the place is still free: the state is looked at again right before.
pub fn create(bridge: &Bridge) -> Result<(), BridgeError> {
    let project_skills = bridge.project.join(".agents").join("skills");
    if !project_skills.is_dir() {
        return Err(BridgeError::NotFree {
            link: bridge.link.clone(),
            why: format!("{} does not exist", project_skills.display()),
        });
    }
    let now = state_of(&bridge.project, &bridge.link, &bridge.points_to);
    if now != BridgeState::Missing {
        return Err(BridgeError::NotFree {
            link: bridge.link.clone(),
            why: format!("{now:?}"),
        });
    }
    if let Some(parent) = bridge.link.parent() {
        fs_err::create_dir_all(parent)?;
    }
    // `symlink` never replaces anything: it fails if something appeared since the look above.
    std::os::unix::fs::symlink(&bridge.points_to, &bridge.link)?;
    Ok(())
}
