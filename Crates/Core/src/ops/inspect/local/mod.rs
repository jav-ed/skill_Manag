//! Local changes: how the skills of one project stand against the vault, seen from the project. Which ones
//! the user changed there (and so could replace the vault's copy), which ones only lag behind the vault
//! (a sync fixes those), and which ones the vault does not have at all.

mod error;
mod history;
mod locate;

#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

pub use error::LocalError;
pub use locate::{Found, Located, locate, project_at};

use crate::ops::Workspace;
use crate::plan::inspect::Entry;
use crate::plan::{ChangeKind, Plan, PlanKind, SkillPlan};
use crate::scan::{ScanIssue, SkillsDir, Target, all_targets};

/// What the project did to a file of the skill, compared with the vault.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditKind {
    /// The project has the file, the vault does not.
    Added,
    /// The content or the permissions differ.
    Modified,
    /// The vault has the file, the project does not.
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub rel: PathBuf,
    pub kind: EditKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalState {
    /// The project's folder equals the vault's.
    Same,
    /// The project's folder differs, and is not a version the vault had: someone changed it here.
    Edited(Vec<Edit>),
    /// The project's folder is a version the vault had, and the vault went on since. Sync brings it up.
    /// `commits` is how many changes of the skill the vault has made since (0: the newest one is not
    /// committed yet).
    Behind { commits: usize },
    /// The vault has no skill of this name.
    New,
    /// The skill cannot be compared; the text says why.
    Unusable(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalSkill {
    pub name: String,
    /// The skill folder in the project.
    pub path: PathBuf,
    /// The vault's folders above the skill, when the vault has it.
    pub group: Vec<String>,
    pub state: LocalState,
}

#[derive(Debug)]
pub struct LocalChanges {
    pub project: PathBuf,
    /// Every skill folder of the project, by name.
    pub skills: Vec<LocalSkill>,
    /// What could not be read while listing the skills.
    pub issues: Vec<ScanIssue>,
}

impl LocalChanges {
    pub fn edited(&self) -> impl Iterator<Item = &LocalSkill> {
        self.skills
            .iter()
            .filter(|s| matches!(s.state, LocalState::Edited(_)))
    }
}

/// Compares every skill folder of `project` with the vault. Reads only: nothing is written anywhere.
pub fn local_changes(workspace: &Workspace, project: &Path) -> Result<LocalChanges, LocalError> {
    let dir = SkillsDir {
        project: project.to_path_buf(),
        dir: project.join(".agents").join("skills"),
    };
    let listed = all_targets(std::slice::from_ref(&dir));
    let (known, new): (Vec<Target>, Vec<Target>) = listed
        .targets
        .into_iter()
        .partition(|t| workspace.vault.skills.contains_key(&t.skill));
    let plan = Plan::for_targets(&workspace.vault, &workspace.files, known);
    let mut skills = Vec::new();
    for entry in plan.entries {
        let group = workspace
            .vault
            .skills
            .get(&entry.target.skill)
            .map(|s| s.group.clone())
            .unwrap_or_default();
        let state = match &entry.result {
            Ok(plan) => state_of(workspace, plan)?,
            Err(e) => LocalState::Unusable(e.to_string()),
        };
        skills.push(LocalSkill {
            name: entry.target.skill,
            path: entry.target.path,
            group,
            state,
        });
    }
    skills.extend(new.into_iter().map(|t| LocalSkill {
        name: t.skill,
        path: t.path,
        group: Vec::new(),
        state: LocalState::New,
    }));
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(LocalChanges {
        project: project.to_path_buf(),
        skills,
        issues: listed.issues,
    })
}

fn state_of(workspace: &Workspace, plan: &SkillPlan) -> Result<LocalState, LocalError> {
    match plan.kind {
        PlanKind::Unchanged => return Ok(LocalState::Same),
        // The folder was listed a moment ago; one that is gone by now cannot be compared.
        PlanKind::Create => {
            return Ok(LocalState::Unusable(
                "the folder disappeared while it was read".to_string(),
            ));
        }
        PlanKind::Update => {}
    }
    let Some(existing) = &plan.existing else {
        return Ok(LocalState::Unusable("the folder was not read".to_string()));
    };
    let mut files = Vec::new();
    for (rel, entry) in existing {
        match entry {
            Entry::File { mode, .. } => files.push((rel.clone(), mode & 0o100 != 0)),
            Entry::Dir => {}
            Entry::Other => {
                return Ok(LocalState::Unusable(format!(
                    "{} is a link or a special file, which a skill cannot hold",
                    rel.display()
                )));
            }
        }
    }
    let versions = history::project_versions(&workspace.vault.path, &plan.target.path, &files)?;
    if let Some(commits) =
        history::commits_behind(&workspace.vault.path, &plan.target.skill, &versions)?
    {
        return Ok(LocalState::Behind { commits });
    }
    Ok(LocalState::Edited(edits_of(plan, &files)))
}

/// The plan looks from the vault to the project; the edits look from the project to the vault.
fn edits_of(plan: &SkillPlan, files: &[(PathBuf, bool)]) -> Vec<Edit> {
    let mut edits: Vec<Edit> = plan
        .changes
        .iter()
        .map(|change| Edit {
            rel: change.rel.clone(),
            kind: match change.kind {
                ChangeKind::Added => EditKind::Removed,
                ChangeKind::Modified | ChangeKind::ModeChanged => EditKind::Modified,
            },
        })
        .collect();
    // `removed` also lists folders; only files are edits.
    edits.extend(
        plan.removed
            .iter()
            .filter(|rel| files.iter().any(|(f, _)| f == *rel))
            .map(|rel| Edit {
                rel: rel.clone(),
                kind: EditKind::Added,
            }),
    );
    edits.sort_by(|a, b| a.rel.cmp(&b.rel));
    edits
}
