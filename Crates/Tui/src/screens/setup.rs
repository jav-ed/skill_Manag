//! The setup wizard: choose the vault, choose the scan root, pick the mandatory skills, save.

use std::path::{Path, PathBuf};

use skillmirror_core::config::VaultConfig;
use skillmirror_core::vault::{discover, read_files};

use super::list_view::ListView;
use super::picker::{Outcome, Picker};
use crate::session::describe;

pub(crate) enum Step {
    Vault,
    Root,
    Mandatory,
    Save,
    /// The save finished, with the error message when it failed.
    Saved(Result<(), String>),
}

/// What the wizard knows when it opens.
pub(crate) struct SetupStart {
    pub(crate) home: PathBuf,
    /// The file that remembers the vault.
    pub(crate) pointer: PathBuf,
    pub(crate) vault: Option<PathBuf>,
    pub(crate) root: Option<PathBuf>,
    /// Why a saved value would not take effect in this run, such as a `--vault` flag.
    pub(crate) note: Option<String>,
}

/// Everything the app needs to write.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SaveRequest {
    pub(crate) vault: PathBuf,
    pub(crate) root: PathBuf,
    pub(crate) mandatory: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SetupAction {
    None,
    Leave,
    Save(SaveRequest),
}

pub(crate) struct Setup {
    pub(crate) step: Step,
    pub(crate) picker: Picker,
    pub(crate) vault: Option<PathBuf>,
    pub(crate) root: Option<PathBuf>,
    /// Skills of the chosen vault, plus mandatory names the vault lacks, with their check marks.
    pub(crate) skills: Vec<Skill>,
    pub(crate) list: ListView,
    pub(crate) note: Option<String>,
    /// Why the chosen folder was refused.
    pub(crate) error: Option<String>,
    pub(super) start: SetupStart,
    existing: VaultConfig,
}

pub(crate) struct Skill {
    pub(crate) name: String,
    pub(crate) in_vault: bool,
    pub(crate) checked: bool,
}

impl Setup {
    pub(crate) fn new(start: SetupStart) -> Self {
        let first = start
            .vault
            .as_deref()
            .filter(|p| p.is_dir())
            .unwrap_or(&start.home)
            .to_path_buf();
        Self {
            step: Step::Vault,
            picker: Picker::new(&first),
            vault: None,
            root: None,
            skills: Vec::new(),
            list: ListView::default(),
            note: start.note.clone(),
            error: None,
            start,
            existing: VaultConfig::default(),
        }
    }

    /// The picker took a directory for the current step.
    pub(super) fn chosen(&mut self, path: PathBuf) {
        match self.step {
            Step::Vault => self.choose_vault(path),
            Step::Root => {
                self.root = Some(path);
                self.step = Step::Mandatory;
            }
            _ => {}
        }
    }

    fn choose_vault(&mut self, path: PathBuf) {
        let vault = match discover(&path) {
            Ok(vault) => vault,
            Err(e) => {
                self.error = Some(describe(&e));
                return;
            }
        };
        // The vault must be a git repository: its tracked files decide what is copied.
        if let Err(e) = read_files(&vault) {
            self.error = Some(describe(&e));
            return;
        }
        let existing = match VaultConfig::load(&path) {
            Ok(config) => config,
            Err(e) => {
                self.error = Some(describe(&e));
                return;
            }
        };
        let mut names: Vec<String> = vault.skills.keys().cloned().collect();
        let lacking: Vec<&String> = existing
            .mandatory
            .iter()
            .filter(|n| !names.contains(n))
            .collect();
        let mut skills: Vec<Skill> = names
            .drain(..)
            .map(|name| Skill {
                checked: existing.mandatory.contains(&name),
                name,
                in_vault: true,
            })
            .collect();
        skills.extend(lacking.into_iter().map(|name| Skill {
            name: name.clone(),
            in_vault: false,
            checked: true,
        }));
        self.list
            .set((0..skills.len()).map(|i| (i, Vec::new())).collect());
        self.skills = skills;
        let root_start = [existing.root.clone(), self.start.root.clone()]
            .into_iter()
            .flatten()
            .find(|p| p.is_dir())
            .unwrap_or_else(|| self.start.home.clone());
        self.existing = existing;
        self.vault = Some(path);
        self.error = None;
        self.picker = Picker::new(&root_start);
        self.step = Step::Root;
    }

    /// One step back; from the first step the wizard is left.
    pub(super) fn back(&mut self) -> SetupAction {
        self.error = None;
        match self.step {
            Step::Vault | Step::Saved(_) => return SetupAction::Leave,
            Step::Root => {
                let vault = self
                    .vault
                    .clone()
                    .unwrap_or_else(|| self.start.home.clone());
                self.picker = Picker::new(&vault);
                self.step = Step::Vault;
            }
            Step::Mandatory => {
                let root = self.root.clone().unwrap_or_else(|| self.start.home.clone());
                self.picker = Picker::new(&root);
                self.step = Step::Root;
            }
            Step::Save => self.step = Step::Mandatory,
        }
        SetupAction::None
    }

    pub(super) fn toggle(&mut self, index: usize) {
        if let Some(skill) = self.skills.get_mut(index) {
            skill.checked = !skill.checked;
        }
    }

    pub(super) fn toggle_all(&mut self) {
        let all_on = self.skills.iter().all(|s| s.checked);
        for skill in &mut self.skills {
            skill.checked = !all_on;
        }
    }

    pub(super) fn request(&self) -> Option<SaveRequest> {
        Some(SaveRequest {
            vault: self.vault.clone()?,
            root: self.root.clone()?,
            mandatory: self
                .skills
                .iter()
                .filter(|s| s.checked && s.in_vault)
                .map(|s| s.name.clone())
                .collect(),
        })
    }

    pub(crate) fn finish(&mut self, result: Result<(), String>) {
        self.step = Step::Saved(result);
    }

    pub(super) fn picked(&mut self, outcome: Outcome) {
        if let Outcome::Chosen(path) = outcome {
            self.chosen(path);
        }
    }

    pub(crate) fn pointer(&self) -> &Path {
        &self.start.pointer
    }
}
