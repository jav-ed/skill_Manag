//! A directory picker: lists the sub-directories of one directory and lets the user walk the tree.

use std::path::{Path, PathBuf};

use super::list_view::ListView;
use crate::binding::{DOWN, PAGE_DOWN, PAGE_UP, UP};
use crate::hit::Target;
use crate::input::{Button, Code, Key, Mouse, MouseKind};

pub(crate) enum Outcome {
    Pending,
    /// The user chose the directory the picker is in.
    Chosen(PathBuf),
}

pub(crate) struct Picker {
    pub(crate) cwd: PathBuf,
    pub(crate) names: Vec<String>,
    pub(crate) view: ListView,
    pub(crate) show_hidden: bool,
    /// Set when a directory could not be read; shown, never swallowed.
    pub(crate) error: Option<String>,
}

impl Picker {
    /// Starts in `start`. When that cannot be read the error is shown and the picker stays empty.
    pub(crate) fn new(start: &Path) -> Self {
        let mut picker = Self {
            cwd: start.to_path_buf(),
            names: Vec::new(),
            view: ListView::default(),
            show_hidden: false,
            error: None,
        };
        picker.go(start);
        picker
    }

    fn go(&mut self, path: &Path) {
        match list_dirs(path, self.show_hidden) {
            Ok(names) => {
                self.cwd = path.to_path_buf();
                self.view
                    .set((0..names.len()).map(|i| (i, Vec::new())).collect());
                self.names = names;
                self.error = None;
            }
            Err(e) => self.error = Some(format!("{}: {e}", path.display())),
        }
    }

    fn open_current(&mut self) {
        if let Some(name) = self.view.current().and_then(|i| self.names.get(i)) {
            let next = self.cwd.join(name);
            self.go(&next);
        }
    }

    fn parent(&mut self) {
        if let Some(parent) = self.cwd.parent().map(Path::to_path_buf) {
            self.go(&parent);
        }
    }

    fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
        let here = self.cwd.clone();
        self.go(&here);
    }

    pub(crate) fn on_key(&mut self, key: Key) -> Outcome {
        if UP.matches(key) {
            self.view.move_by(-1);
        } else if DOWN.matches(key) {
            self.view.move_by(1);
        } else if PAGE_UP.matches(key) {
            self.view.page(-1);
        } else if PAGE_DOWN.matches(key) {
            self.view.page(1);
        } else {
            match key.code {
                Code::Right | Code::Char('l') => self.open_current(),
                Code::Left | Code::Char('h') | Code::Backspace => self.parent(),
                Code::Char('.') => self.toggle_hidden(),
                Code::Enter if self.error.is_none() => return Outcome::Chosen(self.cwd.clone()),
                _ => {}
            }
        }
        Outcome::Pending
    }

    pub(crate) fn on_mouse(&mut self, mouse: Mouse, target: Option<Target>) -> Outcome {
        match (mouse.kind, target) {
            (MouseKind::Moved, Some(Target::PickerRow(i))) => self.view.cursor = i,
            (MouseKind::ScrollDown, Some(Target::PickerRow(_))) => self.view.scroll_by(3),
            (MouseKind::ScrollUp, Some(Target::PickerRow(_))) => self.view.scroll_by(-3),
            (MouseKind::Down(Button::Left), Some(Target::PickerRow(i))) => {
                // The first click selects a row, a click on the selected row opens it.
                if self.view.cursor == i {
                    self.open_current();
                } else {
                    self.view.cursor = i;
                }
            }
            (MouseKind::Down(Button::Left), Some(Target::PickerUp)) => self.parent(),
            (MouseKind::Down(Button::Left), Some(Target::PickerHidden)) => self.toggle_hidden(),
            (MouseKind::Down(Button::Left), Some(Target::PickerSelect)) if self.error.is_none() => {
                return Outcome::Chosen(self.cwd.clone());
            }
            _ => {}
        }
        Outcome::Pending
    }
}

fn list_dirs(path: &Path, show_hidden: bool) -> std::io::Result<Vec<String>> {
    let mut names = Vec::new();
    for entry in fs_err::read_dir(path)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        // `metadata` follows symlinks, so a link to a directory is listed as a directory.
        let is_dir = entry.metadata().is_ok_and(|m| m.is_dir());
        if is_dir && (show_hidden || !name.starts_with('.')) {
            names.push(name);
        }
    }
    names.sort_by_key(|n| n.to_lowercase());
    Ok(names)
}
