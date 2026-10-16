//! Telling "the project changed this skill" from "the project still has an older version of it": the
//! vault is a git repository, so every version it ever held is known. A project copy that is exactly one
//! of them is behind the vault, not edited.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use super::LocalError;

/// A file as git knows it: the blob and whether it is executable.
pub(super) type Versions = BTreeMap<PathBuf, (String, bool)>;

/// The git blob of each project file, computed without writing anything into the vault.
pub(super) fn project_versions(
    vault: &Path,
    base: &Path,
    files: &[(PathBuf, bool)],
) -> Result<Versions, LocalError> {
    let mut input = Vec::new();
    for (rel, _) in files {
        let path = base.join(rel);
        if path.as_os_str().as_bytes().contains(&b'\n') {
            return Err(LocalError::Git {
                command: "hash-object".to_string(),
                stderr: format!("the file name {} has a line break in it", path.display()),
            });
        }
        input.extend_from_slice(path.as_os_str().as_bytes());
        input.push(b'\n');
    }
    let out = run(
        // `--no-filters`: the bytes on disk, as `plan` compares them, not what a git attribute would make of them.
        git(vault).args(["hash-object", "--no-filters", "--stdin-paths"]),
        Some(&input),
        "hash-object",
    )?;
    let ids: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_string)
        .collect();
    if ids.len() != files.len() {
        return Err(LocalError::Git {
            command: "hash-object".to_string(),
            stderr: format!("{} files in, {} ids out", files.len(), ids.len()),
        });
    }
    Ok(files
        .iter()
        .zip(ids)
        .map(|((rel, exec), id)| (rel.clone(), (id, *exec)))
        .collect())
}

/// How many changes of the skill the vault has made since the version `project` holds, when the project
/// holds a version the vault had. The skill is followed through folder moves: it is the folder with its
/// name, wherever it was.
pub(super) fn commits_behind(
    vault: &Path,
    name: &str,
    project: &Versions,
) -> Result<Option<usize>, LocalError> {
    let glob = format!(":(glob)**/{name}/**");
    let commits = lines(&run(
        git(vault).args(["log", "--format=%H", "--", &glob]),
        None,
        "log",
    )?);
    if commits.is_empty() {
        return Ok(None);
    }
    // Every folder this skill ever had, from the paths its commits touched.
    let touched = run(
        // `--relative`: paths from the vault folder, which is also where `ls-tree` reads them from, even
        // when the vault is a folder inside a bigger repository.
        git(vault).args([
            "log",
            "--relative",
            "--name-only",
            "--format=",
            "-z",
            "--",
            &glob,
        ]),
        None,
        "log --name-only",
    )?;
    let dirs = skill_dirs(&touched.stdout, name);
    for (position, commit) in commits.iter().enumerate() {
        let mut listing = git(vault);
        listing.args(["ls-tree", "-r", "-z", commit, "--"]);
        listing.args(dirs.iter().map(|d| d.as_os_str()));
        let tree = run(&mut listing, None, "ls-tree")?;
        if same_tree(&tree.stdout, &dirs, project) {
            return Ok(Some(position));
        }
    }
    Ok(None)
}

/// The folders named `name` that the paths belong to: the path up to and including that component.
fn skill_dirs(raw: &[u8], name: &str) -> BTreeSet<PathBuf> {
    raw.split(|b| *b == 0)
        .filter(|p| !p.is_empty())
        .filter_map(|p| {
            let path = Path::new(OsStr::from_bytes(p));
            let at = path.components().position(|c| c.as_os_str() == name)?;
            Some(path.components().take(at + 1).collect())
        })
        .collect()
}

/// Whether the tree listing holds, under one of `dirs`, exactly the files and blobs of `project`.
fn same_tree(listing: &[u8], dirs: &BTreeSet<PathBuf>, project: &Versions) -> bool {
    let mut per_dir: BTreeMap<&Path, Versions> = BTreeMap::new();
    for record in listing.split(|b| *b == 0).filter(|r| !r.is_empty()) {
        // `<mode> blob <id>\t<path>`
        let Some(tab) = record.iter().position(|b| *b == b'\t') else {
            continue;
        };
        let (meta, path) = record.split_at(tab);
        let path = Path::new(OsStr::from_bytes(path.get(1..).unwrap_or_default()));
        let meta = String::from_utf8_lossy(meta);
        let mut fields = meta.split(' ');
        let (Some(mode), Some(_kind), Some(id)) = (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        let Some(dir) = dirs.iter().find(|d| path.starts_with(d)) else {
            continue;
        };
        let Ok(rel) = path.strip_prefix(dir) else {
            continue;
        };
        per_dir
            .entry(dir.as_path())
            .or_default()
            .insert(rel.to_path_buf(), (id.to_string(), mode == "100755"));
    }
    per_dir.values().any(|files| files == project)
}

fn git(vault: &Path) -> Command {
    let mut cmd = crate::git::command();
    cmd.arg("-C").arg(vault);
    cmd
}

fn lines(out: &Output) -> Vec<String> {
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_string)
        .collect()
}

fn run(cmd: &mut Command, input: Option<&[u8]>, name: &str) -> Result<Output, LocalError> {
    use std::io::Write;
    use std::process::Stdio;
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    cmd.stdin(if input.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    let mut child = cmd.spawn()?;
    if let (Some(bytes), Some(mut stdin)) = (input, child.stdin.take()) {
        stdin.write_all(bytes)?;
    }
    let out = child.wait_with_output()?;
    if out.status.success() {
        return Ok(out);
    }
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    // A vault without a first commit has no versions yet; that is an answer, not a failure.
    if stderr.contains("does not have any commits yet") || stderr.contains("unknown revision") {
        return Ok(Output {
            stdout: Vec::new(),
            ..out
        });
    }
    Err(LocalError::Git {
        command: name.to_string(),
        stderr,
    })
}
