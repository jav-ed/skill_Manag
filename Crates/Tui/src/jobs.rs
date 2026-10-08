//! Background jobs. Each one runs on its own thread and reports through the event channel, so the
//! interface never blocks on the disk.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::thread;

use skillmirror_core::apply::{ApplyOptions, apply};
use skillmirror_core::config::Settings;
use skillmirror_core::events::Event as CoreEvent;
use skillmirror_core::ops;
use skillmirror_core::plan::Plan;
use skillmirror_core::scan::Target;

use crate::event::{Event, Job};
use crate::results::{Kind, Results};
use crate::session::{Session, describe, load};

fn send(tx: &Sender<Event>, job: Job) {
    // A closed channel means the UI has ended; there is nobody left to tell.
    drop(tx.send(Event::Job(job)));
}

pub(crate) fn spawn_load(tx: Sender<Event>, settings: Settings) {
    thread::spawn(move || {
        let loaded = load(settings);
        send(&tx, Job::Loaded(Box::new(loaded)));
    });
}

/// Applies the vault copy to the targets (sync and push).
pub(crate) fn spawn_apply(
    tx: Sender<Event>,
    session: Arc<Session>,
    kind: Kind,
    targets: Vec<Target>,
) {
    thread::spawn(move || {
        let total = targets.len();
        let done = AtomicUsize::new(0);
        let plan = Plan::for_targets(&session.workspace.vault, &session.workspace.files, targets);
        let progress = |event: CoreEvent| {
            if matches!(event, CoreEvent::TargetDone { .. }) {
                let now = done.fetch_add(1, Ordering::Relaxed) + 1;
                send(&tx, Job::Progress { done: now, total });
            }
        };
        match apply(plan, ApplyOptions::default(), &progress) {
            Ok(report) => send(
                &tx,
                Job::Finished(Box::new(Results::from_applied(report, kind))),
            ),
            Err(e) => send(
                &tx,
                Job::Failed(describe(&skillmirror_core::Error::from(e))),
            ),
        }
    });
}

pub(crate) fn spawn_delete(tx: Sender<Event>, targets: Vec<Target>) {
    thread::spawn(move || {
        let total = targets.len();
        let done = AtomicUsize::new(0);
        let progress = |event: CoreEvent| {
            if matches!(event, CoreEvent::TargetDone { .. }) {
                let now = done.fetch_add(1, Ordering::Relaxed) + 1;
                send(&tx, Job::Progress { done: now, total });
            }
        };
        let report = ops::delete(targets, false, &progress);
        send(&tx, Job::Finished(Box::new(Results::from_deleted(report))));
    });
}
