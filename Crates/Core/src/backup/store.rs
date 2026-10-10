//! The backup store: every tree a run replaced or removed, kept so `undo` can bring it back.
//!
//! Layout below `<state>/backups/`:
//!
//! ```text
//! <run-id>/<index>/entry.json   which project and skill, and what the run did to it
//! <run-id>/<index>/tree/        the old folder (absent for a skill the run created)
//! ```

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use serde::{Deserialize, Serialize};

use super::{BackupError, clock, tree};
use crate::config::Dirs;
use crate::scan::Target;

/// Runs kept after a run that stored something. Older runs are removed.
pub const KEEP_RUNS: usize = 30;

pub(super) const ENTRY_FILE: &str = "entry.json";
const TREE_DIR: &str = "tree";
/// Where an entry about a single file keeps the old file.
pub(super) const FILE_SLOT: &str = "file";

/// The command that made a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RunKind {
    Sync,
    Push,
    Delete,
    Add,
    Init,
    Undo,
}

impl RunKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Sync => "sync",
            Self::Push => "push",
            Self::Delete => "delete",
            Self::Add => "add",
            Self::Init => "init",
            Self::Undo => "undo",
        }
    }
}

/// What a run did to one skill folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Change {
    Created,
    Updated,
    Deleted,
}

/// What an entry is about.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Subject {
    /// A skill folder, saved as a tree. Notes written before AGENTS.md existed have no subject: this.
    #[default]
    Skill,
    /// The project's AGENTS.md, saved as one file; the entry's `skill` holds its file name.
    Agents,
}

impl Subject {
    // Serde calls the `skip_serializing_if` function with a reference, whatever the type is.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn is_skill(&self) -> bool {
        *self == Self::Skill
    }
}

/// The note stored next to a saved tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub kind: RunKind,
    #[serde(with = "super::pathjson")]
    pub project: PathBuf,
    pub skill: String,
    pub change: Change,
    #[serde(default, skip_serializing_if = "Subject::is_skill")]
    pub subject: Subject,
}

/// All backups, on disk.
#[derive(Debug, Clone)]
pub struct Backups {
    root: PathBuf,
}

impl Backups {
    /// The store below the state directory.
    pub fn in_dirs(dirs: &Dirs) -> Self {
        Self::at(dirs.state().join("backups"))
    }

    pub fn at(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Starts a run. Nothing is created on disk until the run stores something.
    pub fn begin(&self, kind: RunKind) -> Result<Run, BackupError> {
        let id = clock::new_id()?;
        Ok(Run {
            dir: self.root.join(&id),
            id,
            kind,
            stored: AtomicUsize::new(0),
        })
    }

    /// Run ids, newest first. A missing store is an empty list.
    pub fn run_ids(&self) -> Result<Vec<String>, BackupError> {
        let mut ids = Vec::new();
        match fs_err::read_dir(&self.root) {
            Ok(entries) => {
                for entry in entries {
                    let entry = entry?;
                    if entry.file_type()?.is_dir()
                        && let Some(name) = entry.file_name().to_str()
                    {
                        ids.push(name.to_string());
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        ids.sort_unstable_by(|a, b| b.cmp(a));
        Ok(ids)
    }

    /// Every run with its entries, newest first.
    pub fn runs(&self) -> Result<Vec<LoadedRun>, BackupError> {
        self.run_ids()?.iter().map(|id| self.load(id)).collect()
    }

    /// One run. Entries come back in the order the run wrote them.
    pub fn load(&self, id: &str) -> Result<LoadedRun, BackupError> {
        let dir = self.dir_of(id)?;
        let mut entries = Vec::new();
        for item in fs_err::read_dir(&dir)? {
            let item = item?;
            let Some(index) = item
                .file_name()
                .to_str()
                .and_then(|n| n.parse::<usize>().ok())
            else {
                continue;
            };
            entries.push(read_entry(index, item.path())?);
        }
        entries.sort_by_key(|e| e.index);
        Ok(LoadedRun {
            id: id.to_string(),
            dir,
            entries,
        })
    }

    /// The newest run that still has an entry.
    pub fn latest(&self) -> Result<LoadedRun, BackupError> {
        for id in self.run_ids()? {
            let run = self.load(&id)?;
            if !run.entries.is_empty() {
                return Ok(run);
            }
        }
        Err(BackupError::Empty)
    }

    /// Removes the oldest runs so that `keep` remain. Returns how many were removed.
    pub fn prune(&self, keep: usize) -> Result<usize, BackupError> {
        let ids = self.run_ids()?;
        let old = ids.get(keep..).unwrap_or_default();
        for id in old {
            fs_err::remove_dir_all(self.root.join(id))?;
        }
        Ok(old.len())
    }

    /// A run folder for a name from the user; only names this store could have made are accepted.
    fn dir_of(&self, id: &str) -> Result<PathBuf, BackupError> {
        let plain = !id.is_empty() && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-');
        let dir = self.root.join(id);
        if plain && dir.is_dir() {
            Ok(dir)
        } else {
            Err(BackupError::NoSuchRun { id: id.to_string() })
        }
    }
}

/// A run that is saving trees. Safe to share between threads.
#[derive(Debug)]
pub struct Run {
    id: String,
    kind: RunKind,
    pub(super) dir: PathBuf,
    pub(super) stored: AtomicUsize,
}

/// What a finished run left behind.
#[derive(Debug)]
pub struct Finished {
    pub id: String,
    pub stored: usize,
    /// Set when old runs could not be removed; the run itself is saved.
    pub prune_error: Option<BackupError>,
}

impl Run {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn kind(&self) -> RunKind {
        self.kind
    }

    /// Entries saved so far.
    pub fn stored(&self) -> usize {
        self.stored.load(Ordering::Relaxed)
    }

    /// Ends the run and, when it saved something, drops the runs beyond [`KEEP_RUNS`].
    pub fn finish(self, backups: &Backups) -> Finished {
        let stored = self.stored();
        let prune_error = if stored > 0 {
            backups.prune(KEEP_RUNS).err()
        } else {
            None
        };
        Finished {
            id: self.id,
            stored,
            prune_error,
        }
    }

    /// Notes that `target` is about to be created, so `undo` can remove it. Call before creating it.
    pub(crate) fn record_created(&self, index: usize, target: &Target) -> Result<(), BackupError> {
        self.write_entry(index, target, Change::Created)
    }

    /// Takes back [`Run::record_created`] when the creation failed.
    pub(crate) fn forget(&self, index: usize) -> std::io::Result<()> {
        fs_err::remove_dir_all(self.dir.join(index.to_string()))?;
        self.stored.fetch_sub(1, Ordering::Relaxed);
        Ok(())
    }

    /// Moves `old`, the folder the run just replaced or removed, into the store.
    ///
    /// On `Err` nothing was stored and `old` is untouched. `Ok(Some(error))` means it is stored but
    /// `old` could not be removed afterwards (only possible across filesystems).
    pub(crate) fn keep(
        &self,
        index: usize,
        target: &Target,
        change: Change,
        old: &Path,
    ) -> Result<Option<std::io::Error>, BackupError> {
        self.write_entry(index, target, change)?;
        let slot = self.dir.join(index.to_string());
        match tree::move_tree(old, &slot.join(TREE_DIR)) {
            Ok(left) => Ok(left),
            Err(cause) => {
                drop(self.forget(index));
                Err(cause.into())
            }
        }
    }

    fn write_entry(
        &self,
        index: usize,
        target: &Target,
        change: Change,
    ) -> Result<(), BackupError> {
        self.write_note(
            index,
            &Entry {
                kind: self.kind,
                project: target.project.clone(),
                skill: target.skill.clone(),
                change,
                subject: Subject::Skill,
            },
        )
    }

    pub(super) fn write_note(&self, index: usize, entry: &Entry) -> Result<(), BackupError> {
        let slot = self.dir.join(index.to_string());
        fs_err::create_dir_all(&slot)?;
        let path = slot.join(ENTRY_FILE);
        let text = serde_json::to_string(&entry).map_err(|e| BackupError::Note {
            path: path.clone(),
            reason: e.to_string(),
        })?;
        if let Err(cause) = fs_err::write(&path, text) {
            drop(fs_err::remove_dir_all(&slot));
            return Err(cause.into());
        }
        self.stored.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

/// One saved skill folder of a run.
#[derive(Debug, Clone)]
pub struct LoadedEntry {
    pub index: usize,
    /// `<store>/<run>/<index>`.
    pub dir: PathBuf,
    pub entry: Entry,
    /// False for created skills, and for a note whose tree was lost.
    pub has_tree: bool,
}

impl LoadedEntry {
    pub fn tree(&self) -> PathBuf {
        self.dir.join(TREE_DIR)
    }
}

/// A run read back from disk.
#[derive(Debug, Clone)]
pub struct LoadedRun {
    pub id: String,
    pub dir: PathBuf,
    pub entries: Vec<LoadedEntry>,
}

impl LoadedRun {
    /// The command of the run, from its first entry.
    pub fn kind(&self) -> Option<RunKind> {
        self.entries.first().map(|e| e.entry.kind)
    }
}

fn read_entry(index: usize, dir: PathBuf) -> Result<LoadedEntry, BackupError> {
    let path = dir.join(ENTRY_FILE);
    let text = fs_err::read_to_string(&path).map_err(|e| BackupError::BadEntry {
        path: path.clone(),
        reason: e.to_string(),
    })?;
    let entry: Entry = serde_json::from_str(&text).map_err(|e| BackupError::BadEntry {
        path,
        reason: e.to_string(),
    })?;
    let has_tree = match entry.subject {
        Subject::Skill => dir.join(TREE_DIR).is_dir(),
        Subject::Agents => dir.join(FILE_SLOT).is_file(),
    };
    Ok(LoadedEntry {
        index,
        dir,
        entry,
        has_tree,
    })
}
