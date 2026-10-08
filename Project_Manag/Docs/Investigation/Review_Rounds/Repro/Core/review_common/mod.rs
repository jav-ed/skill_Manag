//! Helpers shared by the review reproducers: a throwaway git vault, targets, and short outcome text.
#![allow(dead_code)]

use std::path::Path;
use std::process::Command;

use skillmirror_core::apply::Outcome;
use skillmirror_core::scan::Target;
use skillmirror_core::vault::{Vault, VaultFiles, discover, read_files};

pub(crate) fn git(dir: &Path, args: &[&str]) {
    let mut cmd = Command::new("git");
    cmd.current_dir(dir);
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(key);
        }
    }
    cmd.env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null");
    cmd.args([
        "-c",
        "user.name=t",
        "-c",
        "user.email=t@t",
        "-c",
        "commit.gpgsign=false",
    ]);
    cmd.args(args);
    let out = cmd.output().unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

pub(crate) fn write(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

pub(crate) fn commit_all(dir: &Path) {
    git(dir, &["init", "-q"]);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "init"]);
}

pub(crate) fn target(project: &Path, skill: &str) -> Target {
    Target {
        project: project.to_path_buf(),
        skill: skill.to_string(),
        path: project.join(".agents/skills").join(skill),
    }
}

/// A vault with one skill `sk` whose `SKILL.md` says `NEW!`.
pub(crate) fn small_vault(tmp: &Path) -> (Vault, VaultFiles) {
    let vault = tmp.join("vault");
    write(&vault.join("sk/SKILL.md"), "NEW!\n");
    commit_all(&vault);
    let found = discover(&vault).unwrap();
    let files = read_files(&found).unwrap();
    (found, files)
}

pub(crate) fn short(outcome: &Outcome) -> String {
    format!("{outcome:?}").chars().take(120).collect()
}
