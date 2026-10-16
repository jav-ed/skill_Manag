//! The jobs of the Override page. Each one opens the vault and reads the project afresh, so what is shown,
//! what is asked and what is written come from the disk as it is then, not from an earlier look.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::thread;

use skillmirror_core::apply::{Failure, Outcome};
use skillmirror_core::backup::{Finished, RunKind};
use skillmirror_core::config::{Dirs, Settings};
use skillmirror_core::events::{Event as CoreEvent, ignore_events};
use skillmirror_core::ops::{self, PromotePlan, Scope, Workspace};
use skillmirror_core::scan::{same_folder, sync_targets};

use super::send;
use crate::event::{Event, Job, JobId};
use crate::screens::local::{Compared, Done, Located, Outcome as SkillOutcome, Pending, Spread};
use crate::session::describe;

/// Where to look for the project.
pub(crate) enum Look {
    /// The folder the interface was started in: the project around it, or the ones below it.
    Around(PathBuf),
    /// A project the user picked.
    At(PathBuf),
}

fn open(settings: Settings) -> Result<Workspace, String> {
    Workspace::open(settings).map_err(|e| describe(&e))
}

pub(crate) fn spawn_locate(tx: Sender<Event>, id: JobId, settings: Settings, look: Look) {
    thread::spawn(move || {
        let found = locate(settings, look);
        send(&tx, id, Job::Located(Box::new(found)));
    });
}

fn locate(settings: Settings, look: Look) -> Result<Located, String> {
    let options = settings.scan_options();
    let workspace = open(settings)?;
    match look {
        Look::Around(here) => match ops::locate(&here, &options).map_err(|e| describe(&e))? {
            ops::Located::One { root, found } => compare(&workspace, root, Some(found)),
            ops::Located::Several(projects) => Ok(Located::Several(projects)),
        },
        Look::At(project) => compare(&workspace, project, None),
    }
}

fn compare(
    workspace: &Workspace,
    project: PathBuf,
    found: Option<ops::Found>,
) -> Result<Located, String> {
    let changes = ops::local_changes(workspace, &project).map_err(|e| describe(&e))?;
    Ok(Located::One(Compared {
        project,
        found,
        problems: changes.issues.len(),
        skills: changes.skills,
    }))
}

/// Reads the project again and checks that the skills can be replaced. Nothing is written.
pub(crate) fn spawn_plan(
    tx: Sender<Event>,
    id: JobId,
    settings: Settings,
    project: PathBuf,
    names: Vec<String>,
    spread: bool,
) {
    thread::spawn(move || {
        let planned = plan(settings, &project, &names, spread);
        send(&tx, id, Job::PromotePlanned(Box::new(planned)));
    });
}

fn plan(
    settings: Settings,
    project: &Path,
    names: &[String],
    spread: bool,
) -> Result<Pending, String> {
    let workspace = open(settings)?;
    let changes = ops::local_changes(&workspace, project).map_err(|e| describe(&e))?;
    let plan = ops::plan_promote(&workspace, &changes, names).map_err(|e| describe(&e))?;
    let spread = if spread {
        Some(others_with(&workspace, project, names)?)
    } else {
        None
    };
    Ok(Pending { plan, spread })
}

/// How many projects other than `project` have one of the skills.
fn others_with(workspace: &Workspace, project: &Path, names: &[String]) -> Result<usize, String> {
    let report = workspace.scan(&ignore_events).map_err(|e| describe(&e))?;
    let wanted: BTreeSet<&str> = names.iter().map(String::as_str).collect();
    let set = sync_targets(&report.skills_dirs, |name| wanted.contains(name));
    let projects: BTreeSet<&Path> = set
        .targets
        .iter()
        .filter(|t| !same_folder(&t.project, project))
        .map(|t| t.project.as_path())
        .collect();
    Ok(projects.len())
}

/// Replaces the vault's copies, then, when asked, updates the other projects from the new vault.
pub(crate) fn spawn_promote(
    tx: Sender<Event>,
    id: JobId,
    settings: Settings,
    dirs: Dirs,
    plan: PromotePlan,
    spread: bool,
) {
    thread::spawn(
        move || match promote(&tx, id, settings, &dirs, &plan, spread) {
            Ok(done) => send(&tx, id, Job::Promoted(Box::new(done))),
            Err(message) => send(&tx, id, Job::Failed(message)),
        },
    );
}

fn promote(
    tx: &Sender<Event>,
    id: JobId,
    settings: Settings,
    dirs: &Dirs,
    plan: &PromotePlan,
    spread: bool,
) -> Result<Done, String> {
    let workspace = open(settings.clone())?;
    let report = ops::promote(&workspace.vault, plan, dirs)
        .map_err(|e| describe(&skillmirror_core::Error::from(e)))?;
    let promoted: Vec<String> = report
        .skills
        .iter()
        .filter(|s| s.result.is_ok())
        .map(|s| s.name.clone())
        .collect();
    let outcomes = report
        .skills
        .iter()
        .map(|s| SkillOutcome {
            name: s.name.clone(),
            result: match &s.result {
                Ok(done) => Ok(done.files),
                Err(e) => Err(describe(e)),
            },
            left_out: s.result.as_ref().map_or(0, |d| d.left_out.len()),
        })
        .collect();
    let spread =
        (spread && !promoted.is_empty()).then(|| update_others(tx, id, settings, dirs, &promoted));
    Ok(Done {
        outcomes,
        backup: saved(&report.backup),
        spread,
    })
}

fn saved(finished: &Finished) -> Option<String> {
    (finished.stored > 0).then(|| finished.id.clone())
}

/// A sync of the promoted skills into every project that has them, from a fresh read of the vault: the
/// file lists git gave before the promote do not know the new files.
fn update_others(
    tx: &Sender<Event>,
    id: JobId,
    settings: Settings,
    dirs: &Dirs,
    names: &[String],
) -> Result<Spread, String> {
    let workspace = open(settings)?;
    let scanned = workspace.scan(&ignore_events).map_err(|e| describe(&e))?;
    let scope = Scope {
        skills: Some(names.iter().cloned().collect()),
        project: None,
    };
    let plan = ops::plan_sync_scoped(&workspace, &scanned, &scope).map_err(|e| describe(&e))?;
    let total = plan.entries.len();
    let done = std::sync::atomic::AtomicUsize::new(0);
    let progress = |event: CoreEvent| {
        if matches!(event, CoreEvent::TargetDone { .. }) {
            let now = done.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
            send(tx, id, Job::Progress { done: now, total });
        }
    };
    let outcome = ops::run_plan(dirs, RunKind::Sync, plan, &progress).map_err(|e| describe(&e))?;
    let written: Vec<_> = outcome
        .report
        .applied
        .iter()
        .filter(|a| matches!(a.outcome, Outcome::Created | Outcome::Updated))
        .collect();
    let projects: BTreeSet<&Path> = written.iter().map(|a| a.target.project.as_path()).collect();
    let failed = outcome
        .report
        .applied
        .iter()
        .filter_map(|a| match &a.outcome {
            Outcome::Failed(why) => Some(format!(
                "{} in {}: {}",
                a.target.skill,
                a.target.project.display(),
                failure(why)
            )),
            _ => None,
        })
        .collect();
    Ok(Spread {
        projects: projects.len(),
        written: written.len(),
        failed,
    })
}

fn failure(why: &Failure) -> String {
    match why {
        Failure::Plan(e) => describe(e),
        Failure::Apply(e) => describe(e),
    }
}
