//! Moving a folder tree, across filesystems too.

use std::io;
use std::os::unix::fs::symlink;
use std::path::Path;

use rustix::io::Errno;

/// Moves `from` to `to` (which must not exist; its parent must). On one filesystem that is a single
/// rename. Across filesystems the tree is copied first and `from` removed after.
///
/// `Ok(Some(error))` means the copy is complete but `from` could not be removed. `Err` leaves `from`
/// as it was.
pub(crate) fn move_tree(from: &Path, to: &Path) -> io::Result<Option<io::Error>> {
    match std::fs::rename(from, to) {
        Ok(()) => Ok(None),
        Err(e) if e.raw_os_error() == Some(Errno::XDEV.raw_os_error()) => {
            if let Err(cause) = copy_tree(from, to) {
                // The partial copy is ours; `from` is untouched.
                drop(fs_err::remove_dir_all(to));
                return Err(cause);
            }
            Ok(fs_err::remove_dir_all(from).err())
        }
        Err(e) => Err(io::Error::new(
            e.kind(),
            format!("cannot move {} to {}: {e}", from.display(), to.display()),
        )),
    }
}

/// Moves a file to `to`, which must not exist (its parent must). Like [`move_tree`]: one rename, or a copy and
/// a removal across filesystems; `Ok(Some(error))` means the copy is complete but `from` stayed.
pub(crate) fn move_file(from: &Path, to: &Path) -> io::Result<Option<io::Error>> {
    match std::fs::rename(from, to) {
        Ok(()) => Ok(None),
        Err(e) if e.raw_os_error() == Some(Errno::XDEV.raw_os_error()) => {
            if let Err(cause) = fs_err::copy(from, to) {
                drop(fs_err::remove_file(to));
                return Err(cause);
            }
            Ok(fs_err::remove_file(from).err())
        }
        Err(e) => Err(io::Error::new(
            e.kind(),
            format!("cannot move {} to {}: {e}", from.display(), to.display()),
        )),
    }
}

/// Copies files, folders and symlinks with their permission bits. `to` must not exist.
pub(crate) fn copy_tree(from: &Path, to: &Path) -> io::Result<()> {
    let permissions = fs_err::symlink_metadata(from)?.permissions();
    fs_err::create_dir(to)?;
    for entry in fs_err::read_dir(from)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let source = entry.path();
        let target = to.join(entry.file_name());
        if kind.is_dir() {
            copy_tree(&source, &target)?;
        } else if kind.is_file() {
            fs_err::copy(&source, &target)?;
        } else if kind.is_symlink() {
            symlink(fs_err::read_link(&source)?, &target)?;
        } else {
            return Err(io::Error::other(format!(
                "{} is not a file, folder or link",
                source.display()
            )));
        }
    }
    // After the children, so a read-only folder can still be filled.
    fs_err::set_permissions(to, permissions)
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    #[test]
    fn copy_keeps_modes_links_and_nested_folders() {
        let dir = tempfile::tempdir().unwrap();
        let from = dir.path().join("from");
        std::fs::create_dir_all(from.join("sub")).unwrap();
        std::fs::write(from.join("run.sh"), "x").unwrap();
        std::fs::set_permissions(from.join("run.sh"), std::fs::Permissions::from_mode(0o755))
            .unwrap();
        std::fs::write(from.join("sub/a.md"), "a").unwrap();
        symlink("sub/a.md", from.join("link")).unwrap();
        let to = dir.path().join("to");

        copy_tree(&from, &to).unwrap();

        let mode = std::fs::metadata(to.join("run.sh"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o755);
        assert_eq!(std::fs::read_to_string(to.join("sub/a.md")).unwrap(), "a");
        assert_eq!(
            std::fs::read_link(to.join("link")).unwrap(),
            Path::new("sub/a.md")
        );
    }

    #[test]
    fn copy_fills_a_read_only_folder() {
        let dir = tempfile::tempdir().unwrap();
        let from = dir.path().join("from");
        std::fs::create_dir_all(&from).unwrap();
        std::fs::write(from.join("a"), "a").unwrap();
        std::fs::set_permissions(&from, std::fs::Permissions::from_mode(0o555)).unwrap();
        let to = dir.path().join("to");

        let result = copy_tree(&from, &to);

        std::fs::set_permissions(&from, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::set_permissions(&to, std::fs::Permissions::from_mode(0o755)).unwrap();
        result.unwrap();
        assert_eq!(std::fs::read_to_string(to.join("a")).unwrap(), "a");
    }

    #[test]
    fn move_on_one_filesystem_is_a_rename() {
        let dir = tempfile::tempdir().unwrap();
        let from = dir.path().join("from");
        std::fs::create_dir(&from).unwrap();
        std::fs::write(from.join("a"), "a").unwrap();
        let to = dir.path().join("to");

        assert!(move_tree(&from, &to).unwrap().is_none());

        assert!(!from.exists());
        assert_eq!(std::fs::read_to_string(to.join("a")).unwrap(), "a");
    }

    #[test]
    fn move_onto_an_existing_folder_fails_and_keeps_the_source() {
        let dir = tempfile::tempdir().unwrap();
        let (from, to) = (dir.path().join("from"), dir.path().join("to"));
        std::fs::create_dir(&from).unwrap();
        std::fs::create_dir(&to).unwrap();
        std::fs::write(to.join("keep"), "k").unwrap();

        let err = move_tree(&from, &to).unwrap_err();

        assert!(err.to_string().contains("cannot move"), "{err}");
        assert!(from.exists());
        assert!(to.join("keep").exists());
    }
}
