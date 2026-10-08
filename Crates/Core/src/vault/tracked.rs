//! Which files of each skill are copied: exactly the files git tracks in the vault index.

use std::collections::{BTreeMap, HashMap};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use super::{Vault, VaultError};

const MODE_SYMLINK: &[u8] = b"120000";
const MODE_GITLINK: &[u8] = b"160000";

/// The relative path recorded for a problem that is the skill folder itself.
pub const SELF_PATH: &str = ".";

/// A tracked file of a skill, path relative to the skill folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackedFile {
    pub rel: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProblemKind {
    Symlink,
    Submodule,
}

/// A tracked entry that cannot be mirrored as a regular file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileProblem {
    pub rel: PathBuf,
    pub kind: ProblemKind,
}

/// Everything git says about one skill folder.
#[derive(Debug, Default, Clone)]
pub struct SkillFiles {
    pub tracked: Vec<TrackedFile>,
    /// Files present in the working tree that git does not track and does not ignore. Never copied.
    pub untracked: Vec<PathBuf>,
    pub problems: Vec<FileProblem>,
}

/// Git's view of every skill in a vault, read with two `git` calls in total.
#[derive(Debug, Default)]
pub struct VaultFiles {
    per_skill: BTreeMap<String, SkillFiles>,
}

impl VaultFiles {
    pub fn get(&self, skill: &str) -> Option<&SkillFiles> {
        self.per_skill.get(skill)
    }
}

/// Lists tracked and untracked files of every skill. The vault must be inside a git repository.
pub fn read_files(vault: &Vault) -> Result<VaultFiles, VaultError> {
    let by_dir: HashMap<&Path, &str> = vault
        .skills
        .values()
        .map(|s| (s.rel.as_path(), s.name.as_str()))
        .collect();
    let mut files = VaultFiles::default();
    for name in vault.skills.keys() {
        files.per_skill.insert(name.clone(), SkillFiles::default());
    }

    let staged = run_git(&vault.path, &["ls-files", "-z", "--stage", "--", "."])?;
    for record in staged.split(|b| *b == 0).filter(|r| !r.is_empty()) {
        let entry = parse_stage_record(record)?;
        if entry.stage != b"0" {
            return Err(VaultError::Unmerged { path: entry.path });
        }
        // A link or submodule that is the skill folder itself has no path inside the skill.
        let is_special = matches!(entry.mode.as_slice(), MODE_SYMLINK | MODE_GITLINK);
        let itself = is_special
            .then(|| by_dir.get(entry.path.as_path()))
            .flatten();
        let Some((skill, rel)) = itself
            .map(|name| (*name, PathBuf::from(SELF_PATH)))
            .or_else(|| owner(&by_dir, &entry.path))
        else {
            continue;
        };
        let Some(bucket) = files.per_skill.get_mut(skill) else {
            continue;
        };
        match entry.mode.as_slice() {
            MODE_SYMLINK => bucket.problems.push(FileProblem {
                rel,
                kind: ProblemKind::Symlink,
            }),
            MODE_GITLINK => bucket.problems.push(FileProblem {
                rel,
                kind: ProblemKind::Submodule,
            }),
            _ => bucket.tracked.push(TrackedFile { rel }),
        }
    }

    let others = run_git(
        &vault.path,
        &[
            "ls-files",
            "-z",
            "--others",
            "--exclude-standard",
            "--",
            ".",
        ],
    )?;
    for path in others.split(|b| *b == 0).filter(|r| !r.is_empty()) {
        let path = PathBuf::from(OsStr::from_bytes(path));
        if let Some((skill, rel)) = owner(&by_dir, &path)
            && let Some(bucket) = files.per_skill.get_mut(skill)
        {
            bucket.untracked.push(rel);
        }
    }
    Ok(files)
}

struct StageEntry {
    mode: Vec<u8>,
    stage: Vec<u8>,
    path: PathBuf,
}

/// Parses `<mode> <object> <stage>\t<path>`, the record format of `git ls-files --stage -z`.
fn parse_stage_record(record: &[u8]) -> Result<StageEntry, VaultError> {
    let bad = |detail: &str| VaultError::GitOutput {
        args: "ls-files -z --stage".to_string(),
        detail: detail.to_string(),
    };
    let tab = record
        .iter()
        .position(|b| *b == b'\t')
        .ok_or_else(|| bad("record without a tab"))?;
    let (meta, path) = record.split_at(tab);
    let mut fields = meta.split(|b| *b == b' ');
    let mode = fields.next().ok_or_else(|| bad("missing mode"))?;
    let _object = fields.next().ok_or_else(|| bad("missing object id"))?;
    let stage = fields.next().ok_or_else(|| bad("missing stage"))?;
    Ok(StageEntry {
        mode: mode.to_vec(),
        stage: stage.to_vec(),
        path: PathBuf::from(OsStr::from_bytes(path.get(1..).unwrap_or_default())),
    })
}

/// The skill that contains `path` and the path relative to that skill. Skills never nest, so one ancestor matches at most.
fn owner<'a>(by_dir: &HashMap<&Path, &'a str>, path: &Path) -> Option<(&'a str, PathBuf)> {
    path.ancestors().skip(1).find_map(|dir| {
        let name = by_dir.get(dir)?;
        let rel = path.strip_prefix(dir).ok()?.to_path_buf();
        Some((*name, rel))
    })
}

fn run_git(vault: &Path, args: &[&str]) -> Result<Vec<u8>, VaultError> {
    let mut cmd = crate::git::command();
    cmd.arg("-C").arg(vault).args(args);
    let out = cmd.output().map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => VaultError::GitMissing,
        _ => VaultError::Io(e),
    })?;
    if out.status.success() {
        return Ok(out.stdout);
    }
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    if stderr.contains("not a git repository") {
        return Err(VaultError::NotAGitRepo {
            path: vault.to_path_buf(),
        });
    }
    Err(VaultError::GitFailed {
        args: args.join(" "),
        status: out.status.to_string(),
        stderr,
    })
}
