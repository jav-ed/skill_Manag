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
    /// Look at the chosen vault folder (it reads git and every skill, so the app does it off the UI thread).
    Check(PathBuf),
    Save(SaveRequest),
}

/// What looking at a vault folder found.
#[derive(Debug)]
pub(crate) struct VaultCheck {
    path: PathBuf,
    skills: Vec<String>,
    existing: VaultConfig,
}

/// Reads the folder as a vault: it must hold skills, be a git repository and have a valid config.
/// This is the slow part of choosing a vault, which is why it is a function of its own.
pub(crate) fn check_vault(path: PathBuf) -> Result<VaultCheck, String> {
    let vault = discover(&path).map_err(|e| describe(&e))?;
    // The vault must be a git repository: its tracked files decide what is copied.
    read_files(&vault).map_err(|e| describe(&e))?;
    let existing = VaultConfig::load(&path).map_err(|e| describe(&e))?;
    Ok(VaultCheck {
        skills: vault.skills.keys().cloned().collect(),
        path,
        existing,
    })
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
    /// A folder is being looked at in the background.
    pub(crate) checking: bool,
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
            checking: false,
            start,
            existing: VaultConfig::default(),
        }
    }

    /// The picker took a directory for the current step.
    pub(super) fn chosen(&mut self, path: PathBuf) -> SetupAction {
        match self.step {
            Step::Vault => {
                self.checking = true;
                self.error = None;
                SetupAction::Check(path)
            }
            Step::Root => {
                self.root = Some(path);
                self.step = Step::Mandatory;
                SetupAction::None
            }
            _ => SetupAction::None,
        }
    }

    /// The background look at the chosen vault came back.
    pub(crate) fn checked(&mut self, result: Result<VaultCheck, String>) {
        // The user stepped back meanwhile: the answer is about a choice that no longer stands.
        if !self.checking {
            return;
        }
        self.checking = false;
        match result {
            Ok(check) => self.accept(check),
            Err(message) => self.error = Some(message),
        }
    }

    fn accept(&mut self, check: VaultCheck) {
        let VaultCheck {
            path,
            skills: names,
            existing,
        } = check;
        let lacking: Vec<&String> = existing
            .mandatory
            .iter()
            .filter(|n| !names.contains(n))
            .collect();
        let mut skills: Vec<Skill> = names
            .iter()
            .map(|name| Skill {
                checked: existing.mandatory.contains(name),
                name: name.clone(),
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
        self.checking = false;
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

    pub(super) fn picked(&mut self, outcome: Outcome) -> SetupAction {
        match outcome {
            Outcome::Chosen(path) => self.chosen(path),
            Outcome::Pending => SetupAction::None,
        }
    }

    pub(crate) fn pointer(&self) -> &Path {
        &self.start.pointer
    }
}
