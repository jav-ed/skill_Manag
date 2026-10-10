//! Background jobs. Each one runs on its own thread and reports through the event channel, so the
//! interface never blocks on the disk. Every report carries the id of its job, so the app can tell a
//! report that still matters from one that arrives after the screen moved on.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::thread;

use skillmirror_core::agents::{
    AgentsError, AgentsPlan, Intent, apply_agents, installed_skills, plan_agents,
};
use skillmirror_core::apply::{ApplyOptions, apply};
use skillmirror_core::backup::{Backups, Run, RunKind};
use skillmirror_core::config::{Dirs, Settings};
use skillmirror_core::events::Event as CoreEvent;
use skillmirror_core::ops::{self, diff_of_plan};
use skillmirror_core::plan::Plan;

use crate::diffview::{lines_of, lines_of_agents};
use crate::event::{Event, Job, JobId};
use crate::preview::Preview;
use crate::results::{Kind, Results};
use crate::screens::{DiffPage, Pending, check_vault};
use crate::session::{Session, describe, load};
use crate::undo::{do_undo, list_runs, plan_undo};

pub(crate) fn send(tx: &Sender<Event>, id: JobId, job: Job) {
    // A closed channel means the UI has ended; there is nobody left to tell.
    drop(tx.send(Event::Job { id, job }));
}

pub(crate) fn spawn_load(tx: Sender<Event>, id: JobId, settings: Settings) {
    thread::spawn(move || {
        let loaded = load(settings);
        send(&tx, id, Job::Loaded(Box::new(loaded)));
    });
}

/// Looks at a folder the setup wizard was given.
pub(crate) fn spawn_check(tx: Sender<Event>, id: JobId, path: PathBuf) {
    thread::spawn(move || {
        let checked = check_vault(path);
        send(&tx, id, Job::Checked(Box::new(checked)));
    });
}

/// Works out what a sync or push of the selected targets would do, without writing anything.
pub(crate) fn spawn_plan(tx: Sender<Event>, id: JobId, session: Arc<Session>, pending: Pending) {
    thread::spawn(move || {
        if pending.kind == Kind::Agents {
            return match plan_agents_page(&session, pending) {
                Ok(planned) => send(&tx, id, Job::Planned(Box::new(planned))),
                Err(message) => send(&tx, id, Job::Failed(message)),
            };
        }
        let vault = &session.workspace.vault;
        let files = &session.workspace.files;
        // Add and init may create `.agents/skills`; sync and push only work where it is.
        let plan = if pending.install.is_some() {
            Plan::for_targets_creating(vault, files, pending.targets.clone())
        } else {
            Plan::for_targets(vault, files, pending.targets.clone())
        };
        let preview = Preview::of(&plan);
        let agents = match init_agents(&session, &pending) {
            Ok(agents) => agents,
            Err(message) => return send(&tx, id, Job::Failed(message)),
        };
        let planned = Pending {
            plan: Some(plan),
            preview: Some(preview),
            agents,
            ..pending
        };
        send(&tx, id, Job::Planned(Box::new(planned)));
    });
}

/// The AGENTS.md of a new project, planned like the skills: nothing is made if the text cannot be used or
/// the skills it names are not among the ones ticked.
fn init_agents(session: &Session, pending: &Pending) -> Result<Option<Box<AgentsPlan>>, String> {
    let Some(install) = pending
        .install
        .as_ref()
        .filter(|i| i.agents && pending.kind == Kind::Init)
    else {
        return Ok(None);
    };
    let source = session.agents.source.clone()?;
    let skills: BTreeSet<String> = pending.targets.iter().map(|t| t.skill.clone()).collect();
    source
        .require_skills(&install.project, &skills)
        .map_err(|e| match e {
            // The page has a key for this; the command-line flag would send the user elsewhere.
            AgentsError::MissingSkills { missing, project } => format!(
                "The AGENTS.md text names the skills {missing}, which {} would not have\n  tick them on this page, or press m here to leave AGENTS.md out",
                project.display()
            ),
            other => describe(&other),
        })?;
    Ok(Some(Box::new(plan_agents(
        std::slice::from_ref(&install.project),
        &source,
        Intent::ADD,
    ))))
}

/// Works out what writing the block into the selected projects would do: files made, blocks put in front
/// of a file that has none, blocks rewritten, and what cannot be written. Nothing is written.
fn plan_agents_page(session: &Session, pending: Pending) -> Result<Pending, String> {
    let source = session.agents.source.clone()?;
    let projects: Vec<PathBuf> = pending.targets.iter().map(|t| t.project.clone()).collect();
    let intent = Intent {
        create: true,
        update: true,
        force: false,
    };
    let mut plan = plan_agents(&projects, &source, intent);
    plan.refuse_missing_skills(installed_skills);
    let preview = Preview::of_agents(&plan);
    Ok(Pending {
        agents: Some(Box::new(plan)),
        preview: Some(preview),
        ..pending
    })
}

/// Starts the backup run that keeps what a job replaces or removes.
pub(crate) fn begin(dirs: &Dirs, kind: RunKind) -> Result<(Backups, Run), String> {
    let backups = Backups::in_dirs(dirs);
    let run = backups
        .begin(kind)
        .map_err(|e| describe(&skillmirror_core::Error::from(e)))?;
    Ok((backups, run))
}

/// Writes the AGENTS.md plan the user confirmed.
pub(crate) fn spawn_agents(tx: Sender<Event>, id: JobId, dirs: Dirs, plan: AgentsPlan) {
    thread::spawn(move || {
        let (backups, backup) = match begin(&dirs, RunKind::Agents) {
            Ok(started) => started,
            Err(message) => return send(&tx, id, Job::Failed(message)),
        };
        let total = plan.entries.len();
        let done = AtomicUsize::new(0);
        let progress = |event: CoreEvent| {
            if matches!(event, CoreEvent::TargetDone { .. }) {
                let now = done.fetch_add(1, Ordering::Relaxed) + 1;
                send(&tx, id, Job::Progress { done: now, total });
            }
        };
        let report = apply_agents(&plan, Some(&backup), 0, &progress);
        let results = Results::from_agents(&report).with_backup(&backup.finish(&backups));
        send(&tx, id, Job::Finished(Box::new(results)));
    });
}

/// Applies the plan the user confirmed (sync and push).
pub(crate) fn spawn_apply(tx: Sender<Event>, id: JobId, dirs: Dirs, kind: Kind, plan: Plan) {
    thread::spawn(move || {
        let run_kind = if kind == Kind::Push {
            RunKind::Push
        } else {
            RunKind::Sync
        };
        let (backups, backup) = match begin(&dirs, run_kind) {
            Ok(started) => started,
            Err(message) => return send(&tx, id, Job::Failed(message)),
        };
        let total = plan.entries.len();
        let done = AtomicUsize::new(0);
        let progress = |event: CoreEvent| {
            if matches!(event, CoreEvent::TargetDone { .. }) {
                let now = done.fetch_add(1, Ordering::Relaxed) + 1;
                send(&tx, id, Job::Progress { done: now, total });
            }
        };
        let options = ApplyOptions {
            backup: Some(&backup),
            ..ApplyOptions::default()
        };
        match apply(plan, options, &progress) {
            Ok(report) => {
                let results = Results::from_applied(report, kind);
                let results = results.with_backup(&backup.finish(&backups));
                send(&tx, id, Job::Finished(Box::new(results)));
            }
            Err(e) => send(
                &tx,
                id,
                Job::Failed(describe(&skillmirror_core::Error::from(e))),
            ),
        }
    });
}

pub(crate) fn spawn_delete(tx: Sender<Event>, id: JobId, dirs: Dirs, pending: Pending) {
    thread::spawn(move || {
        let (backups, backup) = match begin(&dirs, RunKind::Delete) {
            Ok(started) => started,
            Err(message) => return send(&tx, id, Job::Failed(message)),
        };
        let targets = pending.targets;
        let total = targets.len();
        let done = AtomicUsize::new(0);
        let progress = |event: CoreEvent| {
            if matches!(event, CoreEvent::TargetDone { .. }) {
                let now = done.fetch_add(1, Ordering::Relaxed) + 1;
                send(&tx, id, Job::Progress { done: now, total });
            }
        };
        let report = ops::delete(targets, false, Some(&backup), &progress);
        let results = Results::from_deleted(report).with_backup(&backup.finish(&backups));
        send(&tx, id, Job::Finished(Box::new(results)));
    });
}

/// Reads the backup store.
pub(crate) fn spawn_runs(tx: Sender<Event>, id: JobId, dirs: Dirs) {
    thread::spawn(move || send(&tx, id, Job::Runs(Box::new(list_runs(&dirs)))));
}

/// Checks what undoing a run would do, without changing anything.
pub(crate) fn spawn_undo_plan(tx: Sender<Event>, id: JobId, dirs: Dirs, run: String) {
    thread::spawn(move || {
        let planned = plan_undo(&dirs, &run);
        send(&tx, id, Job::UndoPlanned(Box::new(planned)));
    });
}

/// Undoes a run the user confirmed. `total` is how many folders the confirmation listed.
pub(crate) fn spawn_undo(tx: Sender<Event>, id: JobId, dirs: Dirs, run: String, total: usize) {
    thread::spawn(move || {
        let done = AtomicUsize::new(0);
        let progress = |event: CoreEvent| {
            if matches!(event, CoreEvent::TargetDone { .. }) {
                let now = done.fetch_add(1, Ordering::Relaxed) + 1;
                send(&tx, id, Job::Progress { done: now, total });
            }
        };
        let undone = do_undo(&dirs, &run, &progress);
        send(&tx, id, Job::Undone(Box::new(undone)));
    });
}

/// Reads the files that the plan of a question would change and turns the differences into lines.
pub(crate) fn spawn_diff(tx: Sender<Event>, id: JobId, pending: Pending) {
    thread::spawn(move || {
        let mut lines = pending
            .plan
            .as_ref()
            .map(|plan| lines_of(&diff_of_plan(plan)))
            .unwrap_or_default();
        if let Some(plan) = &pending.agents {
            lines.extend(lines_of_agents(plan));
        }
        let page = DiffPage {
            lines,
            pending,
            scroll: 0,
        };
        send(&tx, id, Job::Diffed(Box::new(page)));
    });
}
