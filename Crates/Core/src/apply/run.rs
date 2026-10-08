//! Applying a whole plan with bounded parallelism.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use rayon::prelude::*;

use super::place::{Keep, Replaces, discard, place};
use super::{ApplyError, stage};
use crate::backup::Run;
use crate::events::{Event, Observer, Status};
use crate::plan::inspect;
use crate::plan::{Plan, PlanError, PlanKind, SkillPlan};
use crate::runid::run_id;
use crate::scan::Target;

/// Tuning for [`apply`].
#[derive(Debug, Clone, Copy)]
pub struct ApplyOptions<'a> {
    /// Targets written at the same time.
    pub threads: usize,
    /// Where replaced folders and created skills are noted. Without it the old copy is deleted.
    pub backup: Option<&'a Run>,
}

impl Default for ApplyOptions<'_> {
    fn default() -> Self {
        Self {
            threads: 8,
            backup: None,
        }
    }
}

/// Why a target was not written.
#[derive(Debug)]
pub enum Failure {
    Plan(PlanError),
    Apply(ApplyError),
}

#[derive(Debug)]
pub enum Outcome {
    Unchanged,
    Created,
    Updated,
    Failed(Failure),
}

/// An old copy that could not be removed after a successful update. The skill itself is correct.
#[derive(Debug)]
pub struct Leftover {
    pub path: PathBuf,
    pub reason: String,
}

/// The result of one target, with its plan when planning succeeded.
#[derive(Debug)]
pub struct Applied {
    pub target: Target,
    pub plan: Option<SkillPlan>,
    pub outcome: Outcome,
    pub leftover: Option<Leftover>,
}

#[derive(Debug, Default)]
pub struct ApplyReport {
    pub applied: Vec<Applied>,
}

impl ApplyReport {
    pub fn failed(&self) -> usize {
        self.applied
            .iter()
            .filter(|a| matches!(a.outcome, Outcome::Failed(_)))
            .count()
    }
}

/// Writes every plan entry that differs from its destination, reporting each target through `observer`.
/// A failing target never stops the others and never leaves a half-written skill.
///
/// That holds when the process is killed or a write fails. It does not hold across a power loss: the
/// staged files are not `fsync`ed before the swap, because one `fsync` per file over hundreds of
/// targets costs seconds and the skills are git-tracked copies that the next sync restores.
pub fn apply(
    plan: Plan,
    options: ApplyOptions<'_>,
    observer: Observer<'_>,
) -> Result<ApplyReport, ApplyError> {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(options.threads.max(1))
        .build()
        .map_err(|e| ApplyError::Pool(e.to_string()))?;
    let run = run_id()?;
    // Created once, before the pool starts, so that no target can take away what its siblings need.
    let prepared = prepare_skills_dirs(&plan);
    let mut applied: Vec<Applied> = pool.install(|| {
        plan.entries
            .into_par_iter()
            .enumerate()
            .map(|(index, entry)| {
                let (plan, outcome, leftover) = match entry.result {
                    Ok(skill_plan) => {
                        let (outcome, leftover) =
                            write_one(&skill_plan, &run, index, options.backup, &prepared);
                        (Some(skill_plan), outcome, leftover)
                    }
                    Err(e) => (None, Outcome::Failed(Failure::Plan(e)), None),
                };
                observer(Event::TargetDone {
                    target: entry.target.clone(),
                    status: status_of(&outcome),
                });
                Applied {
                    target: entry.target,
                    plan,
                    outcome,
                    leftover,
                }
            })
            .collect()
    });
    take_back_unused_dirs(&prepared, &mut applied);
    Ok(ApplyReport { applied })
}

/// What making the skills directory of one new project came to.
type Prepared = BTreeMap<PathBuf, Result<Vec<PathBuf>, (std::io::ErrorKind, String)>>;

/// Creates `.agents/skills` once for every new project in the plan. The value holds the directories
/// this call had to create, innermost first.
fn prepare_skills_dirs(plan: &Plan) -> Prepared {
    let mut prepared = Prepared::new();
    for skill_plan in plan.entries.iter().filter_map(|e| e.result.as_ref().ok()) {
        let Some(skills_dir) = skill_plan
            .target
            .path
            .parent()
            .filter(|_| skill_plan.creates_skills_dir)
        else {
            continue;
        };
        prepared
            .entry(skills_dir.to_path_buf())
            .or_insert_with(|| create_dirs(skills_dir).map_err(|e| (e.kind(), e.to_string())));
    }
    prepared
}

/// Creates `skills_dir` and returns the directories that did not exist before, innermost first.
fn create_dirs(skills_dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let created: Vec<PathBuf> = skills_dir
        .ancestors()
        .take(2)
        .filter(|dir| !dir.exists())
        .map(Path::to_path_buf)
        .collect();
    fs_err::create_dir_all(skills_dir)?;
    Ok(created)
}

/// A new project whose targets all failed gets back the directories this run made for it, so a failure
/// leaves no empty `.agents`. A project with at least one new skill keeps them.
fn take_back_unused_dirs(prepared: &Prepared, applied: &mut [Applied]) {
    for (skills_dir, made) in prepared {
        let Ok(made) = made else { continue };
        let of_project = |a: &Applied| a.target.path.parent() == Some(skills_dir.as_path());
        if made.is_empty()
            || applied
                .iter()
                .any(|a| of_project(a) && !matches!(a.outcome, Outcome::Failed(_)))
        {
            continue;
        }
        if let Err(stuck) = remove_dirs(made)
            && let Some(first) = applied.iter_mut().find(|a| of_project(a))
            && let Outcome::Failed(Failure::Apply(cause)) = &first.outcome
        {
            first.outcome = Outcome::Failed(Failure::Apply(ApplyError::LeftBehind {
                stage: stuck.0,
                cause: cause.to_string(),
                cleanup: stuck.1,
            }));
        }
    }
}

/// Removes the directories in order. Missing or no longer empty ones belong to someone else by now.
fn remove_dirs(dirs: &[PathBuf]) -> Result<(), (PathBuf, String)> {
    for dir in dirs {
        match fs_err::remove_dir(dir) {
            Ok(()) => {}
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::DirectoryNotEmpty
                ) => {}
            Err(e) => return Err((dir.clone(), e.to_string())),
        }
    }
    Ok(())
}

fn write_one(
    plan: &SkillPlan,
    run: &str,
    index: usize,
    backup: Option<&Run>,
    prepared: &Prepared,
) -> (Outcome, Option<Leftover>) {
    let failed = |e: ApplyError| (Outcome::Failed(Failure::Apply(e)), None);
    if plan.kind == PlanKind::Unchanged {
        return match verify_unchanged(plan) {
            Ok(()) => (Outcome::Unchanged, None),
            Err(e) => failed(e),
        };
    }
    let Some(agents_dir) = plan.target.path.parent().and_then(Path::parent) else {
        return failed(ApplyError::Io(std::io::Error::other(
            "skill path has no .agents parent",
        )));
    };
    if let Some(Err((kind, message))) = plan.target.path.parent().and_then(|dir| prepared.get(dir))
    {
        return failed(ApplyError::Io(std::io::Error::new(*kind, message.clone())));
    }
    // Outside `skills/`, so an agent that lists the skills never sees a half-built copy.
    let stage_dir = agents_dir.join(format!(".stage-{run}-{index}"));
    let keep = backup.map(|run| Keep {
        run,
        index,
        target: &plan.target,
    });
    match write_staged(plan, &stage_dir, keep) {
        Ok(leftover) if plan.kind == PlanKind::Create => (Outcome::Created, leftover),
        Ok(leftover) => (Outcome::Updated, leftover),
        Err(e) => failed(e),
    }
}

/// An unchanged plan is only true while nobody touched the folder since.
fn verify_unchanged(plan: &SkillPlan) -> Result<(), ApplyError> {
    let Some(expected) = &plan.existing else {
        return Ok(());
    };
    let now = inspect::walk(&plan.target.path)?;
    if now == *expected || saved_again_unchanged(plan, &now, expected)? {
        Ok(())
    } else {
        Err(ApplyError::DestinationChanged {
            dest: plan.target.path.clone(),
        })
    }
}

/// A file saved again with the same bytes has new times but is still the file the plan compared.
fn saved_again_unchanged(
    plan: &SkillPlan,
    now: &inspect::Snapshot,
    expected: &inspect::Snapshot,
) -> Result<bool, ApplyError> {
    if !inspect::same_shape(now, expected) {
        return Ok(false);
    }
    for source in &plan.sources {
        let vault = plan.source_dir.join(&source.rel);
        if !inspect::same_bytes(&vault, &plan.target.path.join(&source.rel))? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn write_staged(
    plan: &SkillPlan,
    stage_dir: &Path,
    keep: Option<Keep<'_>>,
) -> Result<Option<Leftover>, ApplyError> {
    stage::build(plan, stage_dir).map_err(|cause| discard(stage_dir, cause))?;
    let replaces = if plan.kind == PlanKind::Create {
        Replaces::Nothing
    } else {
        Replaces::Folder {
            expected: plan.existing.as_ref(),
        }
    };
    place(stage_dir, &plan.target.path, replaces, keep)
}

fn status_of(outcome: &Outcome) -> Status {
    match outcome {
        Outcome::Unchanged => Status::Unchanged,
        Outcome::Created => Status::Created,
        Outcome::Updated => Status::Updated,
        Outcome::Failed(Failure::Plan(e)) => Status::Failed(e.to_string()),
        Outcome::Failed(Failure::Apply(e)) => Status::Failed(e.to_string()),
    }
}
