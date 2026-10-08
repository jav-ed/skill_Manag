//! A throwaway world for tests: a git-tracked vault, a scan root with projects, and an isolated home.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stdout,
    clippy::missing_panics_doc
)]

use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

/// Temporary directory holding `vault/`, `projects/` and `home/`.
pub struct World {
    dir: TempDir,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        for sub in ["vault", "projects", "home"] {
            std::fs::create_dir_all(dir.path().join(sub)).unwrap();
        }
        Self { dir }
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    pub fn vault(&self) -> PathBuf {
        self.dir.path().join("vault")
    }

    /// The scan root.
    pub fn root(&self) -> PathBuf {
        self.dir.path().join("projects")
    }

    /// Writes a file below the vault.
    pub fn vault_file(&self, rel: &str, content: &str) -> &Self {
        self.write(&format!("vault/{rel}"), content);
        self
    }

    /// Writes a file below the scan root.
    pub fn project_file(&self, rel: &str, content: &str) -> &Self {
        self.write(&format!("projects/{rel}"), content);
        self
    }

    /// Writes `root:` and `mandatory:` into the vault config.
    pub fn vault_config(&self, mandatory: &[&str]) -> &Self {
        let list = mandatory.join(", ");
        self.vault_file(
            "config.yaml",
            &format!("root: {}\nmandatory: [{list}]\n", self.root().display()),
        )
    }

    /// Like [`World::vault_config`], with extra YAML (for example `profiles:`) appended.
    pub fn vault_config_with(&self, mandatory: &[&str], extra_yaml: &str) -> &Self {
        let list = mandatory.join(", ");
        self.vault_file(
            "config.yaml",
            &format!(
                "root: {}\nmandatory: [{list}]\n{extra_yaml}\n",
                self.root().display()
            ),
        )
    }

    fn write(&self, rel: &str, content: &str) {
        let path = self.dir.path().join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    /// Reads a file below the world directory.
    pub fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.dir.path().join(rel)).unwrap()
    }

    pub fn exists(&self, rel: &str) -> bool {
        self.dir.path().join(rel).exists()
    }

    /// Initialises the vault as a git repository (once) and commits everything in it.
    pub fn commit_vault(&self) -> &Self {
        if !self.vault().join(".git").exists() {
            self.git(&["init", "-q"]);
        }
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "--allow-empty", "-m", "snapshot"]);
        self
    }

    fn git(&self, args: &[&str]) {
        let mut cmd = Command::new("git");
        cmd.current_dir(self.vault());
        for (key, _) in std::env::vars_os() {
            if key.to_string_lossy().starts_with("GIT_") {
                cmd.env_remove(key);
            }
        }
        let out = cmd
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .args([
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "-c",
                "commit.gpgsign=false",
                "-c",
                "init.defaultBranch=main",
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    /// Environment that isolates a run of the tool from the real machine: own home and XDG directories.
    /// Pass it to a command after clearing the inherited environment.
    pub fn env(&self) -> Vec<(&'static str, PathBuf)> {
        let home = self.dir.path().join("home");
        vec![
            ("HOME", home.clone()),
            ("XDG_CONFIG_HOME", home.join(".config")),
            ("XDG_STATE_HOME", home.join(".local/state")),
            ("XDG_CACHE_HOME", home.join(".cache")),
            (
                "PATH",
                std::env::var_os("PATH")
                    .map(PathBuf::from)
                    .unwrap_or_default(),
            ),
        ]
    }

    /// A standard fixture: three vault skills (one nested in a group), two projects that use some of them.
    pub fn standard() -> Self {
        let world = Self::new();
        world
            .vault_file("coding/SKILL.md", "coding v2")
            .vault_file("web/astro/SKILL.md", "astro v2")
            .vault_file("web/astro/ref.md", "ref")
            .vault_file("tmux/SKILL.md", "tmux v2")
            .project_file("one/.agents/skills/coding/SKILL.md", "coding v1")
            .project_file("one/.agents/skills/astro/SKILL.md", "astro v1")
            .project_file("one/.agents/skills/local-only/SKILL.md", "mine")
            .project_file("two/.agents/skills/coding/SKILL.md", "coding v1")
            .project_file("three/.agents/skills/.keep", "")
            .project_file("plain/src/main.rs", "");
        world.vault_config(&["coding", "tmux"]).commit_vault();
        world
    }
}
