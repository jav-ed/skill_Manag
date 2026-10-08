//! Planning one skill: compare the tracked vault files with the destination folder.

use std::collections::BTreeSet;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use super::PlanError;
use super::inspect::{Entry, same_bytes, walk};
use super::model::{ChangeKind, FileChange, PlanKind, SkillPlan, SourceFile};
use crate::scan::Target;
use crate::vault::{ProblemKind, Skill, SkillFiles};

/// Builds the plan for `target`. Touches nothing on disk.
pub(super) fn plan_skill(
    target: &Target,
    skill: &Skill,
    files: &SkillFiles,
    create_skills_dir: bool,
) -> Result<SkillPlan, PlanError> {
    if let Some(problem) = files.problems.first() {
        let what = match problem.kind {
            ProblemKind::Symlink => "symlink",
            ProblemKind::Submodule => "submodule",
        };
        if problem.rel == Path::new(crate::vault::SELF_PATH) {
            return Err(PlanError::SkillFolderIsLink {
                skill: skill.name.clone(),
                what,
            });
        }
        return Err(PlanError::UnsupportedEntry {
            skill: skill.name.clone(),
            path: problem.rel.clone(),
            what,
        });
    }
    if files.tracked.is_empty() {
        return Err(PlanError::NoTrackedFiles {
            skill: skill.name.clone(),
        });
    }
    // The vault decides "is a skill" from the file on disk, git decides what is copied: both must agree,
    // or the copy would replace a working skill with folder contents that no agent recognises.
    if !files.tracked.iter().any(|t| t.rel == Path::new("SKILL.md")) {
        return Err(PlanError::SkillFileNotTracked {
            skill: skill.name.clone(),
        });
    }
    let sources = read_sources(skill, files)?;
    let mut plan = SkillPlan {
        target: target.clone(),
        source_dir: skill.dir.clone(),
        kind: PlanKind::Create,
        sources,
        changes: Vec::new(),
        removed: Vec::new(),
        creates_skills_dir: false,
        existing: None,
    };
    if let Some(link) = crate::scan::first_link_above(&target.path)? {
        return Err(PlanError::DestinationIsSymlink { path: link });
    }
    match fs_err::symlink_metadata(&target.path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            if !skills_dir_exists(&target.path)? {
                if !create_skills_dir {
                    return Err(PlanError::SkillsDirMissing {
                        path: target
                            .path
                            .parent()
                            .map(Path::to_path_buf)
                            .unwrap_or_default(),
                    });
                }
                plan.creates_skills_dir = true;
            }
            plan.changes = plan
                .sources
                .iter()
                .map(|s| FileChange {
                    rel: s.rel.clone(),
                    kind: ChangeKind::Added,
                })
                .collect();
        }
        Err(e) => return Err(e.into()),
        Ok(meta) if meta.file_type().is_symlink() => {
            return Err(PlanError::DestinationIsSymlink {
                path: target.path.clone(),
            });
        }
        Ok(meta) if !meta.is_dir() => {
            return Err(PlanError::DestinationNotADirectory {
                path: target.path.clone(),
            });
        }
        Ok(_) => compare(&mut plan)?,
    }
    Ok(plan)
}

/// Whether the skills directory of `skill_path` exists. A link or a file in its place is an error.
fn skills_dir_exists(skill_path: &Path) -> Result<bool, PlanError> {
    let Some(skills_dir) = skill_path.parent() else {
        return Ok(false);
    };
    match fs_err::symlink_metadata(skills_dir) {
        Ok(meta) if meta.is_dir() => Ok(true),
        Ok(meta) if meta.file_type().is_symlink() => Err(PlanError::DestinationIsSymlink {
            path: skills_dir.to_path_buf(),
        }),
        Ok(_) => Err(PlanError::SkillsDirMissing {
            path: skills_dir.to_path_buf(),
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
    }
}

/// Checks every tracked file in the vault working tree and records its permission bits.
fn read_sources(skill: &Skill, files: &SkillFiles) -> Result<Vec<SourceFile>, PlanError> {
    let mut sources = Vec::with_capacity(files.tracked.len());
    for tracked in &files.tracked {
        let path = skill.dir.join(&tracked.rel);
        let meta = match fs_err::symlink_metadata(&path) {
            Ok(meta) => meta,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(PlanError::MissingSourceFile { path });
            }
            Err(e) => return Err(e.into()),
        };
        if !meta.is_file() {
            return Err(PlanError::NotRegularFile { path });
        }
        sources.push(SourceFile {
            rel: tracked.rel.clone(),
            mode: meta.permissions().mode() & 0o777,
        });
    }
    Ok(sources)
}

fn compare(plan: &mut SkillPlan) -> Result<(), PlanError> {
    let existing = walk(&plan.target.path)?;
    for source in &plan.sources {
        let kind = match existing.get(&source.rel) {
            None | Some(Entry::Dir | Entry::Other) => Some(ChangeKind::Added),
            Some(Entry::File { len, mode, .. }) => file_change(plan, source, *len, *mode)?,
        };
        if let Some(kind) = kind {
            plan.changes.push(FileChange {
                rel: source.rel.clone(),
                kind,
            });
        }
    }
    let wanted_files: BTreeSet<&Path> = plan.sources.iter().map(|s| s.rel.as_path()).collect();
    let wanted_dirs: BTreeSet<&Path> = plan
        .sources
        .iter()
        .flat_map(|s| s.rel.ancestors().skip(1))
        .collect();
    plan.removed = existing
        .iter()
        .filter(|(rel, entry)| match entry {
            Entry::Dir => !wanted_dirs.contains(rel.as_path()),
            _ => !wanted_files.contains(rel.as_path()),
        })
        .map(|(rel, _)| rel.clone())
        .collect::<Vec<PathBuf>>();
    plan.existing = Some(existing);
    plan.kind = if plan.changes.is_empty() && plan.removed.is_empty() {
        PlanKind::Unchanged
    } else {
        PlanKind::Update
    };
    Ok(())
}

fn file_change(
    plan: &SkillPlan,
    source: &SourceFile,
    dest_len: u64,
    dest_mode: u32,
) -> Result<Option<ChangeKind>, PlanError> {
    let src = plan.source_dir.join(&source.rel);
    let dst = plan.target.path.join(&source.rel);
    let src_len = fs_err::metadata(&src)?.len();
    if src_len != dest_len || !same_bytes(&src, &dst)? {
        return Ok(Some(ChangeKind::Modified));
    }
    Ok((dest_mode != source.mode).then_some(ChangeKind::ModeChanged))
}
