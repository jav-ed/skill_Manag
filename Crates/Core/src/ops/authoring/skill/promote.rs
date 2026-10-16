//! Promoting: the project's copy of a skill replaces the vault's. The vault's old folder goes to the backup
//! store (`undo` brings it back), the new one is staged in git and never committed, like adopt and new.
//! Only a skill the project changed (`LocalState::Edited`) is promoted: one that is merely behind the vault
//! would take the vault back in time, and one the vault does not have is adopted.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::adopt::collect;
use super::author::AuthorError;
use super::created::stage;
use crate::Hint;
use crate::apply::{ApplyError, discard, exchange};
use crate::backup::{BackupError, Backups, Change, Finished, Run, RunKind};
use crate::config::Dirs;
use crate::ops::{Edit, LocalChanges, LocalSkill, LocalState, Workspace};
use crate::plan::inspect::{Snapshot, walk, walk_settled};
use crate::runid::run_id;
use crate::vault::Vault;

#[derive(Debug, thiserror::Error)]
pub enum PromoteError {
    #[error("project {project} has no skill folder {name:?}")]
    NotInProject { project: PathBuf, name: String },
    #[error("{name:?} is not promoted: {why}")]
    NotEdited { name: String, why: String },
    #[error(
        "{name:?} has changes in the vault that are not staged ({files}), a promote would replace them"
    )]
    VaultDirty { name: String, files: String },
    #[error("{name:?} changed in the project after it was looked at; nothing was replaced")]
    SourceChanged { name: String },
    #[error("{name:?} changed in the vault after it was looked at; nothing was replaced")]
    VaultChanged { name: String },
    #[error("promoted {name:?}, but `git add` failed in the vault: {stderr}")]
    NotStaged { name: String, stderr: String },
    #[error("`git {command}` failed in the vault: {stderr}")]
    Git { command: String, stderr: String },
    #[error(transparent)]
    Author(#[from] AuthorError),
    #[error(transparent)]
    Apply(#[from] ApplyError),
    #[error(transparent)]
    Backup(#[from] BackupError),
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

impl Hint for PromoteError {
    fn hint(&self) -> Option<String> {
        match self {
            Self::NotEdited { .. } => Some(
                "`skillmirror changes` shows the state of every skill of the project".to_string(),
            ),
            Self::VaultDirty { .. } => Some(
                "commit or discard those changes in the vault first, or work from them".to_string(),
            ),
            Self::SourceChanged { .. } | Self::VaultChanged { .. } => {
                Some("look at the changes again and confirm the new state".to_string())
            }
            Self::NotStaged { .. } => Some(
                "the new copy is in the vault; `skillmirror undo` takes it back, or stage it with `git add`"
                    .to_string(),
            ),
            Self::Author(e) => e.hint(),
            Self::Apply(e) => e.hint(),
            Self::Backup(e) => e.hint(),
            _ => None,
        }
    }
}

/// One skill that will replace the vault's copy.
#[derive(Debug, Clone)]
pub struct SkillPromotion {
    pub name: String,
    /// The project's folder.
    pub source: PathBuf,
    /// The vault's folder, which it replaces.
    pub dest: PathBuf,
    /// The files the vault's folder will hold, relative to it, sorted.
    pub files: Vec<PathBuf>,
    pub edits: Vec<Edit>,
    /// Both folders as they were when planned. Applying looks again, so an edit made while the plan waited
    /// for a confirmation is never replaced (project side) or overwritten (vault side) unseen.
    source_seen: Snapshot,
    dest_seen: Snapshot,
}

#[derive(Debug, Clone)]
pub struct PromotePlan {
    pub project: PathBuf,
    pub skills: Vec<SkillPromotion>,
}

/// Checks that every named skill can be promoted and reads both sides; writes nothing. Pass a comparison
/// made just before, so that what the user was shown is what is checked.
pub fn plan_promote(
    workspace: &Workspace,
    changes: &LocalChanges,
    names: &[String],
) -> Result<PromotePlan, PromoteError> {
    let mut skills = Vec::new();
    let mut seen = BTreeSet::new();
    for name in names.iter().filter(|n| seen.insert(n.as_str())) {
        let local = changes
            .skills
            .iter()
            .find(|s| &s.name == name)
            .ok_or_else(|| PromoteError::NotInProject {
                project: changes.project.clone(),
                name: name.clone(),
            })?;
        skills.push(plan_one(workspace, local)?);
    }
    Ok(PromotePlan {
        project: changes.project.clone(),
        skills,
    })
}

fn plan_one(workspace: &Workspace, local: &LocalSkill) -> Result<SkillPromotion, PromoteError> {
    let LocalState::Edited(edits) = &local.state else {
        return Err(PromoteError::NotEdited {
            name: local.name.clone(),
            why: why_not(&local.state),
        });
    };
    let Some(skill) = workspace.vault.skills.get(&local.name) else {
        return Err(PromoteError::NotEdited {
            name: local.name.clone(),
            why: why_not(&LocalState::New),
        });
    };
    let dirty = unstaged_changes(&workspace.vault, &skill.rel)?;
    if !dirty.is_empty() {
        return Err(PromoteError::VaultDirty {
            name: local.name.clone(),
            files: dirty.join(", "),
        });
    }
    let mut files = Vec::new();
    collect(&local.path, Path::new(""), &mut files)?;
    files.sort();
    if !files.iter().any(|f| f == Path::new("SKILL.md")) {
        return Err(AuthorError::NoSkillFile {
            path: local.path.clone(),
        }
        .into());
    }
    Ok(SkillPromotion {
        name: local.name.clone(),
        source: local.path.clone(),
        dest: skill.dir.clone(),
        files,
        edits: edits.clone(),
        source_seen: walk_settled(&local.path)?,
        dest_seen: walk_settled(&skill.dir)?,
    })
}

fn why_not(state: &LocalState) -> String {
    match state {
        LocalState::Same => "it equals the vault's copy".to_string(),
        LocalState::Behind { .. } => {
            "it is an older version of the vault's, `sync` brings the project up to date"
                .to_string()
        }
        LocalState::New => "the vault has no skill of this name, `adopt` adds it".to_string(),
        LocalState::Unusable(reason) => reason.clone(),
        LocalState::Edited(_) => "it can be promoted".to_string(),
    }
}

/// Files of the skill folder whose working tree differs from git's index, or that git does not know.
/// What is staged is no problem: that is what an earlier promote leaves.
fn unstaged_changes(vault: &Vault, rel: &Path) -> Result<Vec<String>, PromoteError> {
    let out = crate::git::command()
        .env("GIT_LITERAL_PATHSPECS", "1")
        .arg("-C")
        .arg(&vault.path)
        .args([
            "status",
            "--porcelain=v1",
            "-z",
            "--no-renames",
            "--untracked-files=all",
            "--",
        ])
        .arg(rel)
        .output()?;
    if !out.status.success() {
        return Err(PromoteError::Git {
            command: "status".to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).trim().to_string(),
        });
    }
    // Each record is `XY path`; Y is the folder against the index, `?` for a file git does not know.
    Ok(out
        .stdout
        .split(|b| *b == 0)
        .filter_map(|record| record.split_at_checked(3))
        .filter(|(status, _)| status.get(1).is_some_and(|y| *y != b' '))
        .map(|(_, path)| String::from_utf8_lossy(path).into_owned())
        .collect())
}

#[derive(Debug)]
pub struct Promoted {
    /// The skill folder in the vault.
    pub dir: PathBuf,
    pub files: usize,
    /// Files that were copied but that git did not take (the vault's `.gitignore` says so).
    pub left_out: Vec<PathBuf>,
}

#[derive(Debug)]
pub struct PromoteOutcome {
    pub name: String,
    pub result: Result<Promoted, PromoteError>,
}

#[derive(Debug)]
pub struct PromoteReport {
    pub skills: Vec<PromoteOutcome>,
    /// The backup run that kept the vault's old folders.
    pub backup: Finished,
}

impl PromoteReport {
    pub fn failed(&self) -> usize {
        self.skills.iter().filter(|s| s.result.is_err()).count()
    }
}

/// Replaces the vault's folders and keeps the old ones in a backup run. A skill that cannot be replaced
/// fails alone and is in the report; the others are replaced.
pub fn promote(
    vault: &Vault,
    plan: &PromotePlan,
    dirs: &Dirs,
) -> Result<PromoteReport, BackupError> {
    let backups = Backups::in_dirs(dirs);
    let run = backups.begin(RunKind::Promote)?;
    // Names of the temporary folders, in the form that scans can tell from a leftover of a dead run.
    let token = run_id()?;
    let skills = plan
        .skills
        .iter()
        .enumerate()
        .map(|(index, skill)| PromoteOutcome {
            name: skill.name.clone(),
            result: promote_one(vault, skill, &run, index, &token),
        })
        .collect();
    let backup = run.finish(&backups);
    Ok(PromoteReport { skills, backup })
}

/// Builds the new folder beside the old one, swaps them, and files the old one in the backup run. At every
/// failure the vault is as it was.
fn promote_one(
    vault: &Vault,
    skill: &SkillPromotion,
    run: &Run,
    index: usize,
    token: &str,
) -> Result<Promoted, PromoteError> {
    let name = &skill.name;
    let group = skill.dest.parent().ok_or_else(|| {
        std::io::Error::other(format!("{} has no parent folder", skill.dest.display()))
    })?;
    let stage_dir = group.join(format!(".stage-{token}-{index}"));
    if let Err(cause) = build(&skill.source, &stage_dir, &skill.files) {
        return Err(discard(&stage_dir, cause.into()).into());
    }
    if !matches!(walk(&skill.source), Ok(now) if now == skill.source_seen) {
        let cause = PromoteError::SourceChanged { name: name.clone() };
        drop(fs_err::remove_dir_all(&stage_dir));
        return Err(cause);
    }
    exchange(&stage_dir, &skill.dest).map_err(|cause| discard(&stage_dir, cause))?;
    // After the exchange the stage path holds the vault's old folder.
    if !matches!(walk(&stage_dir), Ok(old) if old == skill.dest_seen) {
        exchange(&stage_dir, &skill.dest)?;
        drop(fs_err::remove_dir_all(&stage_dir));
        return Err(PromoteError::VaultChanged { name: name.clone() });
    }
    let leftover = match run.keep_vault_skill(index, group, name, Change::Updated, &stage_dir) {
        Ok(left) => left,
        Err(cause) => {
            // No backup, no replacement: the old folder goes back.
            exchange(&stage_dir, &skill.dest)?;
            return Err(discard(&stage_dir, cause.into()).into());
        }
    };
    let left_out = stage(vault, &skill.dest, &skill.files).map_err(|e| match e {
        AuthorError::GitAdd { stderr } => PromoteError::NotStaged {
            name: name.clone(),
            stderr,
        },
        other => other.into(),
    })?;
    if let Some(cause) = leftover {
        return Err(BackupError::Leftover {
            path: stage_dir,
            reason: cause.to_string(),
        }
        .into());
    }
    Ok(Promoted {
        dir: skill.dest.clone(),
        files: skill.files.len(),
        left_out,
    })
}

/// Copies the files into the new folder `stage_dir`, with their permission bits.
fn build(source: &Path, stage_dir: &Path, files: &[PathBuf]) -> std::io::Result<()> {
    fs_err::create_dir(stage_dir)?;
    for rel in files {
        let to = stage_dir.join(rel);
        if let Some(parent) = to.parent() {
            fs_err::create_dir_all(parent)?;
        }
        fs_err::copy(source.join(rel), &to)?;
    }
    Ok(())
}
