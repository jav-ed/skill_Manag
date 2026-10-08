//! The machine side: git, the pointer file, the old tool's leftovers, the scan root and the backup store.

use std::path::Path;

use super::{Report, Severity};
use crate::Hint;
use crate::backup::Backups;
use crate::config::{Dirs, Flags, Settings, read_pointer};
use crate::scan::scan;

/// Above this size the backup store is worth a word.
const BIG_STORE: u64 = 1024 * 1024 * 1024;

/// Checks that do not need a working vault: git, the pointer, the old tool, the backup store.
pub(super) fn machine(report: &mut Report, flags: &Flags, dirs: &Dirs) {
    report.ran("git");
    match crate::git::command().arg("--version").output() {
        Ok(out) if out.status.success() => {}
        Ok(_) => report.add(Severity::Error, "git", None, "`git --version` failed", None),
        Err(error) => report.add(
            Severity::Error,
            "git",
            None,
            format!("git cannot be started: {error}"),
            Some("the vault is read through git; install it".to_string()),
        ),
    }

    report.ran("pointer");
    match read_pointer(dirs) {
        Ok(Some(vault)) if !vault.is_dir() => report.add(
            Severity::Warning,
            "pointer",
            Some(dirs.pointer_file().display().to_string()),
            format!("points at {}, which is not a folder", vault.display()),
            Some("run `skillmirror tui` to choose the vault again".to_string()),
        ),
        Ok(Some(vault)) => {
            if let Some(flag) = &flags.vault
                && flag != &vault
            {
                report.add(
                    Severity::Note,
                    "pointer",
                    None,
                    format!("--vault wins over the pointer file ({})", vault.display()),
                    None,
                );
            }
        }
        Ok(None) => {}
        Err(error) => report.add(
            Severity::Error,
            "pointer",
            Some(dirs.pointer_file().display().to_string()),
            error.to_string(),
            error.hint(),
        ),
    }

    report.ran("legacy");
    if dirs.legacy_pointer_file().exists() {
        let hint = if dirs.pointer_file().exists() {
            "the old pointer is still there; `skillmirror migrate --retire` removes it"
        } else {
            "`skillmirror migrate` copies it"
        };
        report.add(
            Severity::Note,
            "legacy",
            Some(dirs.legacy_pointer_file().display().to_string()),
            "the old skill_Manag tool left its vault pointer here",
            Some(hint.to_string()),
        );
    }

    backups(report, dirs);
}

fn backups(report: &mut Report, dirs: &Dirs) {
    report.ran("backups");
    let store = Backups::in_dirs(dirs);
    let ids = match store.run_ids() {
        Ok(ids) => ids,
        Err(error) => {
            report.add(
                Severity::Warning,
                "backups",
                None,
                error.to_string(),
                error.hint(),
            );
            return;
        }
    };
    for id in &ids {
        if let Err(error) = store.load(id) {
            report.add(
                Severity::Warning,
                "backups",
                Some(id.clone()),
                error.to_string(),
                Some("`skillmirror history` skips nothing; delete the damaged run folder if it is not needed".to_string()),
            );
        }
    }
    let size = folder_size(store.root());
    if size > BIG_STORE {
        report.add(
            Severity::Warning,
            "backups",
            Some(store.root().display().to_string()),
            format!("{} runs use {} MiB", ids.len(), size / (1024 * 1024)),
            Some(
                "only the newest runs are kept; delete older run folders by hand to free space"
                    .to_string(),
            ),
        );
    }
}

fn folder_size(root: &Path) -> u64 {
    let mut total = 0;
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(meta) = entry.metadata() else { continue };
            if meta.is_dir() {
                stack.push(entry.path());
            } else {
                total += meta.len();
            }
        }
    }
    total
}

/// The scan root: it must exist, scan cleanly and sit on a filesystem that can swap folders atomically.
pub(super) fn root(report: &mut Report, settings: &Settings) {
    report.ran("scan");
    let root = match settings.root() {
        Ok(root) => root.value.clone(),
        Err(error) => {
            report.add(
                Severity::Warning,
                "scan",
                None,
                error.to_string(),
                error.hint(),
            );
            return;
        }
    };
    match scan(&root, &settings.scan_options()) {
        Ok(found) => {
            for issue in &found.issues {
                let leftover = issue.message.contains("interrupted run");
                report.add(
                    Severity::Warning,
                    if leftover { "leftovers" } else { "scan" },
                    Some(issue.path.display().to_string()),
                    issue.message.clone(),
                    None,
                );
            }
            report.ran("leftovers");
        }
        Err(error) => report.add(
            Severity::Error,
            "scan",
            Some(root.display().to_string()),
            error.to_string(),
            error.hint(),
        ),
    }

    report.ran("filesystem");
    if let Some(kind) = unsupported_filesystem(&root) {
        report.add(
            Severity::Warning,
            "filesystem",
            Some(root.display().to_string()),
            format!(
                "the scan root is on {kind}, which may not support swapping a folder atomically (renameat2); writes into projects there would fail"
            ),
            Some("keep the projects on a local filesystem such as ext4, xfs or btrfs".to_string()),
        );
    }
}

/// The name of a filesystem type that is known not to support `RENAME_EXCHANGE`, when `path` is on one.
fn unsupported_filesystem(path: &Path) -> Option<&'static str> {
    let stat = rustix::fs::statfs(path).ok()?;
    unsupported_magic(u64::try_from(stat.f_type).ok()?)
}

/// Filesystem type numbers from `statfs(2)`.
pub(in crate::ops) fn unsupported_magic(magic: u64) -> Option<&'static str> {
    match magic {
        0x6969 => Some("NFS"),
        0xFF53_4D42 | 0xFE53_4D42 => Some("CIFS/SMB"),
        0x6573_5546 => Some("a FUSE filesystem"),
        0x4d44 => Some("FAT (vfat)"),
        _ => None,
    }
}
