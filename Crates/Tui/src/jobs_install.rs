//! The background jobs of add and init: looking at the chosen folder, and installing into it.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::thread;

use skillmirror_core::agents::{AgentsPlan, apply_agents};
use skillmirror_core::apply::{ApplyOptions, Outcome, apply};
use skillmirror_core::backup::RunKind;
use skillmirror_core::config::Dirs;
use skillmirror_core::events::Event as CoreEvent;
use skillmirror_core::events::ignore_events;
use skillmirror_core::ops::{self, BridgeState};
use skillmirror_core::plan::Plan;

use crate::event::{Event, Job, JobId};
use crate::items::Project;
use crate::jobs::{begin, send};
use crate::results::{Kind, Results};
use crate::screens::{Install, Purpose};
use crate::session::describe;

/// A folder the user chose, and what it holds.
pub(crate) struct Placed {
    pub(crate) purpose: Purpose,
    pub(crate) project: Project,
}

/// Looks at a folder add or init was given.
pub(crate) fn spawn_place(tx: Sender<Event>, id: JobId, purpose: Purpose, path: PathBuf) {
    thread::spawn(move || {
        let placed = check(purpose, path);
        send(&tx, id, Job::Placed(Box::new(placed)));
    });
}

fn check(purpose: Purpose, path: PathBuf) -> Result<Placed, String> {
    let installed = match purpose {
        Purpose::Add => {
            ops::check_existing(&path).map_err(|e| describe(&e))?;
            installed_in(&path)?
        }
        Purpose::Init => {
            ops::check_new(&path).map_err(|e| describe(&e))?;
            BTreeSet::new()
        }
    };
    Ok(Placed {
        purpose,
        project: Project {
            path,
            installed,
            git: false,
            agents: true,
        },
    })
}

/// The skill folders the project has, by name.
fn installed_in(project: &std::path::Path) -> Result<BTreeSet<String>, String> {
    let dir = project.join(".agents").join("skills");
    let entries = match fs_err::read_dir(&dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeSet::new()),
        Err(e) => return Err(e.to_string()),
    };
    let mut names = BTreeSet::new();
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let is_dir = entry.file_type().map_err(|e| e.to_string())?.is_dir();
        if is_dir && !name.starts_with('.') {
            names.insert(name);
        }
    }
    Ok(names)
}

/// What the install job needs besides where to report.
pub(crate) struct InstallJob {
    pub(crate) kind: Kind,
    pub(crate) plan: Plan,
    pub(crate) install: Install,
    /// The agent folders to link to `.agents/skills` after a write.
    pub(crate) bridges: Vec<String>,
    /// The AGENTS.md of a new project, written in the same backup run after the skills.
    pub(crate) agents: Option<Box<AgentsPlan>>,
}

/// Installs the plan the user confirmed. Init makes the folder first and takes it back when nothing could
/// be written; the links the vault config asks for are made after a write.
pub(crate) fn spawn_install(tx: Sender<Event>, id: JobId, dirs: Dirs, job: InstallJob) {
    let InstallJob {
        kind,
        plan,
        install,
        bridges,
        agents,
    } = job;
    thread::spawn(move || {
        let run_kind = if kind == Kind::Init {
            RunKind::Init
        } else {
            RunKind::Add
        };
        let (backups, backup) = match begin(&dirs, run_kind) {
            Ok(started) => started,
            Err(message) => return send(&tx, id, Job::Failed(message)),
        };
        let made = if install.create {
            match ops::create_new(&install.project, install.git) {
                Ok(made) => Some(made),
                Err(e) => return send(&tx, id, Job::Failed(describe(&e))),
            }
        } else {
            None
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
        let applied = apply(plan, options, &progress);
        let wrote = applied.as_ref().map_or(0, |report| {
            report
                .applied
                .iter()
                .filter(|a| matches!(a.outcome, Outcome::Created | Outcome::Updated))
                .count()
        });
        // The file of a new project goes in the same run, after the skills, and only when some were installed.
        let agents_report = agents
            .as_ref()
            .filter(|_| wrote > 0)
            .map(|plan| apply_agents(plan, Some(&backup), total, &ignore_events));
        let mut warnings = Vec::new();
        // Nothing got installed: the folder and repository this job made go, so it can be run again.
        if wrote == 0
            && let Some(project) = &made
            && let Err(e) = project.take_back()
        {
            warnings.push(format!("Could not remove the empty project: {e}"));
        }
        match applied {
            Ok(report) => {
                let mut results =
                    Results::from_applied(report, kind).with_backup(&backup.finish(&backups));
                if let Some(report) = &agents_report {
                    let file = Results::from_agents(report);
                    results.skills.extend(file.skills);
                    results.warnings.extend(file.warnings);
                }
                results.warnings.extend(warnings);
                if wrote > 0 {
                    link(&bridges, &install.project, &mut results);
                }
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

/// Makes the links to `.agents/skills` that the vault config names, where nothing is in the way.
fn link(targets: &[String], project: &std::path::Path, results: &mut Results) {
    for bridge in ops::plan_project_bridges(targets, project) {
        let link = bridge.short(&bridge.link).display().to_string();
        match bridge.state {
            BridgeState::InPlace => {}
            BridgeState::Missing => match ops::create_bridge(&bridge) {
                Ok(()) => results
                    .notes
                    .push(format!("Linked {link} to {}", bridge.points_to.display())),
                Err(e) => results.warnings.push(format!("Could not link {link}: {e}")),
            },
            _ => {
                if let Some((message, hint)) = bridge.problem() {
                    results.warnings.push(format!("{message} ({hint})"));
                }
            }
        }
    }
}
