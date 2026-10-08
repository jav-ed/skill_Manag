//! Applying a whole plan with bounded parallelism.

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
    let applied = pool.install(|| {
        plan.entries
            .into_par_iter()
            .enumerate()
            .map(|(index, entry)| {
                let (plan, outcome, leftover) = match entry.result {
                    Ok(skill_plan) => {
                        let (outcome, leftover) =
                            write_one(&skill_plan, &run, index, options.backup);
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
    Ok(ApplyReport { applied })
}

fn write_one(
    plan: &SkillPlan,
    run: &str,
    index: usize,
    backup: Option<&Run>,
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
    let created = match create_skills_dir(plan, agents_dir) {
        Ok(created) => created,
        Err(e) => return failed(e),
    };
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
        Err(e) => failed(remove_created(&created, e)),
    }
}

/// Creates the skills directory of a new project and returns what it had to create, outermost last.
fn create_skills_dir(plan: &SkillPlan, agents_dir: &Path) -> Result<Vec<PathBuf>, ApplyError> {
    let Some(skills_dir) = plan
        .target
        .path
        .parent()
        .filter(|_| plan.creates_skills_dir)
    else {
        return Ok(Vec::new());
    };
    let created: Vec<PathBuf> = [skills_dir, agents_dir]
        .into_iter()
        .filter(|dir| !dir.exists())
        .map(Path::to_path_buf)
        .collect();
    fs_err::create_dir_all(skills_dir)?;
    Ok(created)
}

/// Takes back the directories a failed target created, so a failure leaves no empty `.agents` behind.
fn remove_created(created: &[PathBuf], cause: ApplyError) -> ApplyError {
    for dir in created {
        match fs_err::remove_dir(dir) {
            Ok(()) => {}
            // Someone else started using it meanwhile; it is theirs now.
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::DirectoryNotEmpty
                ) => {}
            Err(e) => {
                return ApplyError::LeftBehind {
                    stage: dir.clone(),
                    cause: cause.to_string(),
                    cleanup: e.to_string(),
                };
            }
        }
    }
    cause
}

/// An unchanged plan is only true while nobody touched the folder since.
fn verify_unchanged(plan: &SkillPlan) -> Result<(), ApplyError> {
    let Some(expected) = &plan.existing else {
        return Ok(());
    };
    if inspect::walk(&plan.target.path)? == *expected {
        Ok(())
    } else {
        Err(ApplyError::DestinationChanged {
            dest: plan.target.path.clone(),
        })
    }
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
