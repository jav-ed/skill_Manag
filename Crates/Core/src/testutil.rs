//! Helpers for unit tests: temporary trees and throwaway git repositories.

use std::path::{Path, PathBuf};
use std::process::Command;

/// A temporary directory with helpers to write files and run git inside it.
pub(crate) struct TempTree {
    dir: tempfile::TempDir,
}

impl TempTree {
    pub(crate) fn new() -> Self {
        Self {
            dir: tempfile::tempdir().unwrap(),
        }
    }

    pub(crate) fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Writes a file below the tree, creating parent directories.
    pub(crate) fn write(&self, rel: &str, content: &str) -> PathBuf {
        let path = self.dir.path().join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, content).unwrap();
        path
    }

    /// Writes several files at once.
    /// The content of a file below the tree.
    pub(crate) fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.path().join(rel)).unwrap()
    }

    pub(crate) fn write_all(&self, files: &[(&str, &str)]) {
        for (rel, content) in files {
            self.write(rel, content);
        }
    }

    /// Runs git in `sub` (relative to the tree) with a clean environment and a fixed identity.
    pub(crate) fn git(&self, sub: &str, args: &[&str]) -> String {
        let mut cmd = Command::new("git");
        cmd.current_dir(self.dir.path().join(sub));
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
            "-c",
            "init.defaultBranch=main",
        ]);
        cmd.args(args);
        let out = cmd.output().unwrap();
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    /// Makes `sub` a git repository and commits everything in it.
    pub(crate) fn git_init_commit(&self, sub: &str) {
        self.git(sub, &["init", "-q"]);
        self.git(sub, &["add", "-A"]);
        self.git(sub, &["commit", "-q", "-m", "init"]);
    }
}

/// A committed vault inside a temp tree, with discovery and git file lists already read.
pub(crate) struct Fixture {
    pub(crate) tree: TempTree,
    pub(crate) vault: crate::vault::Vault,
    pub(crate) files: crate::vault::VaultFiles,
}

impl Fixture {
    /// Writes `files` below `<tree>/vault`, commits them and reads the vault.
    pub(crate) fn new(files: &[(&str, &str)]) -> Self {
        let tree = TempTree::new();
        for (rel, content) in files {
            tree.write(&format!("vault/{rel}"), content);
        }
        tree.git_init_commit("vault");
        Self::reread(tree)
    }

    /// Re-reads the vault after the test changed it on disk or in git.
    pub(crate) fn reread(tree: TempTree) -> Self {
        let vault = crate::vault::discover(&tree.path().join("vault")).unwrap();
        let files = crate::vault::read_files(&vault).unwrap();
        Self { tree, vault, files }
    }

    pub(crate) fn refresh(self) -> Self {
        Self::reread(self.tree)
    }

    /// A target `<tree>/<project>/.agents/skills/<skill>`; the skills directory is created.
    pub(crate) fn target(&self, project: &str, skill: &str) -> crate::scan::Target {
        let project_dir = self.tree.path().join(project);
        let skills = project_dir.join(".agents/skills");
        std::fs::create_dir_all(&skills).unwrap();
        crate::scan::Target {
            project: project_dir,
            skill: skill.to_string(),
            path: skills.join(skill),
        }
    }

    pub(crate) fn plan(&self, targets: Vec<crate::scan::Target>) -> crate::plan::Plan {
        crate::plan::Plan::for_targets(&self.vault, &self.files, targets)
    }
}
