//! Where add and init write: a folder picker, and for init the name of the new folder.

use std::path::PathBuf;

use tui_input::Input as TextInput;

use super::picker::{Outcome, Picker};
use crate::binding::BACK;
use crate::hit::Target;
use crate::input::{self, Code, Key, KeyKind, Mouse};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Purpose {
    /// An existing project that receives skills.
    Add,
    /// A new project folder.
    Init,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stage {
    Folder,
    /// Init only: the name of the folder to make inside the one chosen.
    Name,
}

/// What the app must do after a key or a click.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PlaceAction {
    None,
    Leave,
    /// Look at the folder in the background: does it suit the purpose?
    Check(Purpose, PathBuf),
}

pub(crate) struct Place {
    pub(crate) purpose: Purpose,
    pub(crate) stage: Stage,
    pub(crate) picker: Picker,
    pub(crate) name: TextInput,
    /// The folder chosen in the picker; for init it is the parent of the new project.
    pub(crate) chosen: Option<PathBuf>,
    /// Why the folder was refused.
    pub(crate) error: Option<String>,
    /// A folder is being looked at in the background.
    pub(crate) checking: bool,
}

impl Place {
    pub(crate) fn new(purpose: Purpose, start: &std::path::Path) -> Self {
        Self {
            purpose,
            stage: Stage::Folder,
            picker: Picker::new(start),
            name: TextInput::default(),
            chosen: None,
            error: None,
            checking: false,
        }
    }

    /// Whether keys currently go into the name field.
    pub(crate) fn typing(&self) -> bool {
        self.stage == Stage::Name && !self.checking
    }

    /// The folder init would make, once a name is typed.
    pub(crate) fn target(&self) -> Option<PathBuf> {
        let name = self.name.value().trim();
        (!name.is_empty())
            .then(|| self.chosen.as_ref().map(|parent| parent.join(name)))
            .flatten()
    }

    pub(crate) fn on_key(&mut self, key: Key) -> PlaceAction {
        if key.kind == KeyKind::Release {
            return PlaceAction::None;
        }
        self.error = None;
        if self.typing() {
            return self.name_key(key);
        }
        if BACK.matches(key) {
            return PlaceAction::Leave;
        }
        if key.code == Code::Esc {
            return self.back();
        }
        // While a folder is being looked at, the picker keeps still: a second choice would race the first.
        if self.checking {
            return PlaceAction::None;
        }
        let outcome = self.picker.on_key(key);
        self.picked(outcome)
    }

    fn name_key(&mut self, key: Key) -> PlaceAction {
        match key.code {
            Code::Esc => return self.back(),
            Code::Enter => return self.named(),
            _ => {
                if let Some(request) = input::to_request(key) {
                    self.name.handle(request);
                }
            }
        }
        PlaceAction::None
    }

    /// One step back; from the first step the page is left.
    fn back(&mut self) -> PlaceAction {
        self.checking = false;
        match self.stage {
            Stage::Folder => PlaceAction::Leave,
            Stage::Name => {
                self.stage = Stage::Folder;
                PlaceAction::None
            }
        }
    }

    fn named(&mut self) -> PlaceAction {
        let name = self.name.value().trim();
        if name.is_empty() {
            self.error = Some("Type a name for the new folder.".to_string());
            return PlaceAction::None;
        }
        if name.contains('/') || name == "." || name == ".." {
            self.error = Some("The name must be one folder name, without slashes.".to_string());
            return PlaceAction::None;
        }
        match self.target() {
            Some(path) => {
                self.checking = true;
                PlaceAction::Check(Purpose::Init, path)
            }
            None => PlaceAction::None,
        }
    }

    fn picked(&mut self, outcome: Outcome) -> PlaceAction {
        let Outcome::Chosen(path) = outcome else {
            return PlaceAction::None;
        };
        match self.purpose {
            Purpose::Add => {
                self.chosen = Some(path.clone());
                self.checking = true;
                PlaceAction::Check(Purpose::Add, path)
            }
            Purpose::Init => {
                self.chosen = Some(path);
                self.stage = Stage::Name;
                PlaceAction::None
            }
        }
    }

    pub(crate) fn on_mouse(&mut self, mouse: Mouse, target: Option<Target>) -> PlaceAction {
        if self.checking || self.stage == Stage::Name {
            return PlaceAction::None;
        }
        let outcome = self.picker.on_mouse(mouse, target);
        self.picked(outcome)
    }

    /// The look at the folder came back.
    pub(crate) fn refused(&mut self, message: String) {
        // The user stepped back meanwhile: the answer is about a choice that no longer stands.
        if self.checking {
            self.checking = false;
            self.error = Some(message);
        }
    }
}
