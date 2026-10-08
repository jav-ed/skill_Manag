//! Symlinks around skill folders: refusing to work through them, and telling two spellings of a folder apart.

use std::path::{Path, PathBuf};

/// The first of `<project>/.agents/skills` and `<project>/.agents` that is a symlink.
///
/// The scan never follows symlinks, so a skill path from a scan is clean. Paths given by hand
/// (`--project`) and paths that changed after the scan are not, and writing or deleting through
/// such a link would reach into a folder that other projects share.
pub fn first_link_above(skill_path: &Path) -> std::io::Result<Option<PathBuf>> {
    for dir in skill_path.ancestors().skip(1).take(2) {
        match fs_err::symlink_metadata(dir) {
            Ok(meta) if meta.file_type().is_symlink() => return Ok(Some(dir.to_path_buf())),
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
    }
    Ok(None)
}

/// Whether both paths are the same folder, written directly or through a link or a relative path.
pub fn same_folder(left: &Path, right: &Path) -> bool {
    left == right || matches!((left.canonicalize(), right.canonicalize()), (Ok(a), Ok(b)) if a == b)
}
