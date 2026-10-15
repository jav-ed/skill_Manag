//! Reading the current state of a destination skill folder.

use std::collections::BTreeMap;
use std::io::Read;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// One entry below a destination skill folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Entry {
    /// `modified` is part of the entry so that an edit which keeps the size is still noticed, and
    /// `changed` (the inode change time) so that one which also puts the modification time back is too:
    /// a program cannot set it.
    File {
        len: u64,
        mode: u32,
        modified: Option<SystemTime>,
        changed: (i64, i64),
    },
    Dir,
    Other,
}

/// The state of a destination folder at one moment, by relative path.
pub(crate) type Snapshot = BTreeMap<PathBuf, Entry>;

/// An entry without its times: what is there, not when it was written.
#[derive(PartialEq, Eq)]
enum Shape {
    File { len: u64, mode: u32 },
    Dir,
    Other,
}

impl Entry {
    fn shape(self) -> Shape {
        match self {
            Self::File { len, mode, .. } => Shape::File { len, mode },
            Self::Dir => Shape::Dir,
            Self::Other => Shape::Other,
        }
    }
}

/// True when both snapshots hold the same paths, sizes and modes, whatever the times say.
pub(crate) fn same_shape(left: &Snapshot, right: &Snapshot) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|((a, x), (b, y))| a == b && x.shape() == y.shape())
}

/// Every entry below `dir` by relative path, without following symlinks.
pub(crate) fn walk(dir: &Path) -> std::io::Result<Snapshot> {
    let mut found = BTreeMap::new();
    visit(dir, Path::new(""), &mut found)?;
    Ok(found)
}

/// How old the newest change in a folder has to be before its snapshot can be trusted. The kernel stamps
/// files from a coarse clock that only moves once per tick (3 to 10 ms), so a second change in the same
/// tick as the first one leaves the change time as it was. Two ticks of the slowest common clock (100 Hz).
const SETTLE: Duration = Duration::from_millis(20);
/// Looks at a folder this many times before it is called busy; each look waits for a quiet moment.
const SETTLE_TRIES: usize = 50;

/// [`walk`] of a folder in which nothing changed during the last [`SETTLE`]. The guard that stops a write
/// over an edit made after the plan compares change times, which only tells two states apart when a change
/// made after the snapshot gets a later time than any in it; that holds once the snapshot is older than a
/// clock tick. A time in the future (a clock that is ahead, on a network folder) cannot be waited out and is
/// taken as it is.
pub(crate) fn walk_settled(dir: &Path) -> std::io::Result<Snapshot> {
    for _ in 0..SETTLE_TRIES {
        let snapshot = walk(dir)?;
        match wait_needed(&snapshot) {
            None => return Ok(snapshot),
            Some(wait) => std::thread::sleep(wait),
        }
    }
    Err(std::io::Error::other(format!(
        "{} was written to {SETTLE_TRIES} times in a row, so there is no quiet moment to plan against; plan again when nothing writes there",
        dir.display()
    )))
}

/// How long until the newest change in `snapshot` is older than [`SETTLE`]; `None` when it already is.
fn wait_needed(snapshot: &Snapshot) -> Option<Duration> {
    let now = SystemTime::now();
    snapshot
        .values()
        .filter_map(|entry| match entry {
            Entry::File {
                changed: (secs, nanos),
                ..
            } => Some((u64::try_from(*secs).ok()?, u32::try_from(*nanos).ok()?)),
            Entry::Dir | Entry::Other => None,
        })
        .filter_map(|(secs, nanos)| {
            now.duration_since(UNIX_EPOCH + Duration::new(secs, nanos))
                .ok()
        })
        // An age that is already SETTLE or more has nothing left to wait for.
        .filter_map(|age| SETTLE.checked_sub(age))
        .max()
}

fn visit(base: &Path, rel: &Path, found: &mut Snapshot) -> std::io::Result<()> {
    for entry in fs_err::read_dir(base.join(rel))? {
        let entry = entry?;
        let rel = rel.join(entry.file_name());
        let kind = entry.file_type()?;
        if kind.is_dir() {
            found.insert(rel.clone(), Entry::Dir);
            visit(base, &rel, found)?;
        } else if kind.is_file() {
            let meta = entry.metadata()?;
            found.insert(
                rel,
                Entry::File {
                    len: meta.len(),
                    mode: meta.permissions().mode() & 0o777,
                    modified: meta.modified().ok(),
                    changed: (meta.ctime(), meta.ctime_nsec()),
                },
            );
        } else {
            found.insert(rel, Entry::Other);
        }
    }
    Ok(())
}

/// True when both files hold the same bytes. The lengths are already known to be equal.
pub(crate) fn same_bytes(left: &Path, right: &Path) -> std::io::Result<bool> {
    const CHUNK: usize = 64 * 1024;
    let mut left = fs_err::File::open(left)?;
    let mut right = fs_err::File::open(right)?;
    let mut left_buf = vec![0u8; CHUNK];
    let mut right_buf = vec![0u8; CHUNK];
    loop {
        let left_len = read_full(&mut left, &mut left_buf)?;
        let right_len = read_full(&mut right, &mut right_buf)?;
        if left_len != right_len || left_buf.get(..left_len) != right_buf.get(..right_len) {
            return Ok(false);
        }
        if left_len == 0 {
            return Ok(true);
        }
    }
}

fn read_full(reader: &mut impl Read, buf: &mut [u8]) -> std::io::Result<usize> {
    let mut filled = 0;
    while filled < buf.len() {
        let Some((_, rest)) = buf.split_at_mut_checked(filled) else {
            break;
        };
        match reader.read(rest)? {
            0 => break,
            n => filled += n,
        }
    }
    Ok(filled)
}
