//! Reading the current state of a destination skill folder.

use std::collections::BTreeMap;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// One entry below a destination skill folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Entry {
    /// `modified` is part of the entry so that an edit which keeps the size is still noticed.
    File {
        len: u64,
        mode: u32,
        modified: Option<SystemTime>,
    },
    Dir,
    Other,
}

/// The state of a destination folder at one moment, by relative path.
pub(crate) type Snapshot = BTreeMap<PathBuf, Entry>;

/// Every entry below `dir` by relative path, without following symlinks.
pub(crate) fn walk(dir: &Path) -> std::io::Result<Snapshot> {
    let mut found = BTreeMap::new();
    visit(dir, Path::new(""), &mut found)?;
    Ok(found)
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
                },
            );
        } else {
            found.insert(rel, Entry::Other);
        }
    }
    Ok(())
}

/// True when both files hold the same bytes. The lengths are already known to be equal.
pub(super) fn same_bytes(left: &Path, right: &Path) -> std::io::Result<bool> {
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
