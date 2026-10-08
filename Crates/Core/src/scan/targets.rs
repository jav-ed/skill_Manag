//! Turning the skills directories of a scan into per-skill targets.

use std::path::PathBuf;

use rayon::prelude::*;

use super::{ScanIssue, SkillsDir};

/// One skill folder inside one project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    /// The parent of `.agents`.
    pub project: PathBuf,
    pub skill: String,
    /// `<project>/.agents/skills/<skill>`.
    pub path: PathBuf,
}

/// Targets in walk order, plus the skills directories that could not be listed.
#[derive(Debug, Default)]
pub struct TargetSet {
    pub targets: Vec<Target>,
    pub issues: Vec<ScanIssue>,
}

/// Installed skill folders whose name `is_wanted` accepts. This is the opt-in rule: nothing is ever added.
pub fn sync_targets(dirs: &[SkillsDir], is_wanted: impl Fn(&str) -> bool + Sync) -> TargetSet {
    collect(dirs, |name| is_wanted(name))
}

/// Every installed skill folder, wanted or not.
pub fn all_targets(dirs: &[SkillsDir]) -> TargetSet {
    collect(dirs, |_| true)
}

/// One target per project and per named skill, whether or not the folder exists yet (push bypasses opt-in).
pub fn push_targets(dirs: &[SkillsDir], skills: &[String]) -> Vec<Target> {
    dirs.iter()
        .flat_map(|d| {
            skills.iter().map(|skill| Target {
                project: d.project.clone(),
                skill: skill.clone(),
                path: d.dir.join(skill),
            })
        })
        .collect()
}

fn collect(dirs: &[SkillsDir], keep: impl Fn(&str) -> bool + Sync) -> TargetSet {
    let parts: Vec<TargetSet> = dirs.par_iter().map(|d| list_one(d, &keep)).collect();
    let mut set = TargetSet::default();
    for part in parts {
        set.targets.extend(part.targets);
        set.issues.extend(part.issues);
    }
    set
}

fn list_one(dir: &SkillsDir, keep: &(impl Fn(&str) -> bool + Sync)) -> TargetSet {
    let mut set = TargetSet::default();
    let entries = match fs_err::read_dir(&dir.dir) {
        Ok(entries) => entries,
        Err(e) => {
            set.issues.push(ScanIssue {
                path: dir.dir.clone(),
                message: e.to_string(),
            });
            return set;
        }
    };
    let mut names = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                set.issues.push(ScanIssue {
                    path: dir.dir.clone(),
                    message: e.to_string(),
                });
                continue;
            }
        };
        // `file_type` does not follow symlinks, so a symlinked skill folder is invisible, as in the Go tool.
        match entry.file_type() {
            Ok(kind) if kind.is_dir() => {}
            Ok(_) => continue,
            Err(e) => {
                set.issues.push(ScanIssue {
                    path: entry.path(),
                    message: e.to_string(),
                });
                continue;
            }
        }
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            set.issues.push(ScanIssue {
                path: entry.path(),
                message: "the folder name is not valid UTF-8, so it cannot be managed".to_string(),
            });
            continue;
        };
        if keep(&name) {
            names.push(name);
        }
    }
    names.sort();
    set.targets = names
        .into_iter()
        .map(|skill| Target {
            project: dir.project.clone(),
            path: dir.dir.join(&skill),
            skill,
        })
        .collect();
    set
}
