//! What the server shares between requests.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use skillmirror_core::backup::now_utc;
use skillmirror_core::config::{Dirs, EnvOverrides, Flags, Settings};
use skillmirror_core::events::ignore_events;
use skillmirror_core::ops::{StatusReport, Workspace, status};
use skillmirror_core::scan::ScanReport;

use super::jobs::Jobs;
use super::plans::Plans;

/// How the server was started.
#[derive(Debug, Clone)]
pub struct Config {
    pub flags: Flags,
    pub env: EnvOverrides,
    pub dirs: Dirs,
    /// Without this the server only looks: every request that would write is refused.
    pub allow_write: bool,
}

impl Config {
    pub(crate) fn settings(&self) -> Result<Settings, String> {
        Settings::load(&self.flags, &self.env, &self.dirs)
            .map_err(|e| skillmirror_core::describe(&e))
    }
}

/// The vault, one scan of the root and the comparison of every project, taken together.
pub(crate) struct Snapshot {
    pub(crate) workspace: Workspace,
    pub(crate) scan: ScanReport,
    pub(crate) status: StatusReport,
    /// `2026-10-08 12:00 UTC`, for the page header.
    pub(crate) at: String,
}

impl Snapshot {
    /// Reads the settings again, opens the vault, scans the root and compares. Slow on a big root, so it
    /// runs on a blocking thread.
    pub(crate) fn take(config: &Config) -> Result<Self, String> {
        let describe = |e: &skillmirror_core::Error| skillmirror_core::describe(e);
        let workspace = Workspace::open(config.settings()?).map_err(|e| describe(&e))?;
        let scan = workspace.scan(&ignore_events).map_err(|e| describe(&e))?;
        let status =
            status(&workspace, &scan).map_err(|e| describe(&skillmirror_core::Error::from(e)))?;
        Ok(Self {
            workspace,
            scan,
            status,
            at: now_utc(),
        })
    }
}

pub(crate) struct AppState {
    pub(crate) config: Config,
    pub(crate) port: u16,
    /// The one-time word in the printed link; gone after the first use.
    token: Mutex<Option<String>>,
    sessions: Mutex<Vec<String>>,
    snapshot: Mutex<Option<Arc<Snapshot>>>,
    pub(crate) plans: Mutex<Plans>,
    pub(crate) jobs: Mutex<Jobs>,
    last_seen: Mutex<Instant>,
}

/// A lock that a panicking request does not poison for everybody else.
pub(crate) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl AppState {
    pub(crate) fn new(config: Config, port: u16, token: String) -> Self {
        Self {
            config,
            port,
            token: Mutex::new(Some(token)),
            sessions: Mutex::new(Vec::new()),
            snapshot: Mutex::new(None),
            plans: Mutex::new(Plans::default()),
            jobs: Mutex::new(Jobs::default()),
            last_seen: Mutex::new(Instant::now()),
        }
    }

    pub(crate) fn touch(&self) {
        *lock(&self.last_seen) = Instant::now();
    }

    pub(crate) fn idle_for(&self) -> Duration {
        lock(&self.last_seen).elapsed()
    }

    /// Spends the launch token when `given` is it. A token works once.
    pub(crate) fn spend_token(&self, given: &str) -> bool {
        let mut token = lock(&self.token);
        let matches = token
            .as_deref()
            .is_some_and(|expected| super::guard::same(expected, given));
        if matches {
            *token = None;
        }
        matches
    }

    pub(crate) fn add_session(&self, id: String) {
        lock(&self.sessions).push(id);
    }

    pub(crate) fn is_session(&self, given: &str) -> bool {
        lock(&self.sessions)
            .iter()
            .any(|known| super::guard::same(known, given))
    }

    /// The current snapshot, taken first when there is none.
    pub(crate) fn snapshot(&self) -> Result<Arc<Snapshot>, String> {
        if let Some(snapshot) = lock(&self.snapshot).clone() {
            return Ok(snapshot);
        }
        self.refresh()
    }

    /// Looks at the disk again and replaces the snapshot.
    pub(crate) fn refresh(&self) -> Result<Arc<Snapshot>, String> {
        let snapshot = Arc::new(Snapshot::take(&self.config)?);
        *lock(&self.snapshot) = Some(Arc::clone(&snapshot));
        Ok(snapshot)
    }

    /// Forgets the snapshot, so the next page looks at the disk again. After a write.
    pub(crate) fn forget(&self) {
        *lock(&self.snapshot) = None;
    }
}
