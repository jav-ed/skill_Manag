//! Backend-neutral input types. The app never sees terminal-library types, so the library can be
//! swapped without touching the app, the bindings or the tests.

use tui_input::InputRequest;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Code {
    Char(char),
    Enter,
    Esc,
    Tab,
    Backspace,
    Delete,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Mods(pub u8);

impl Mods {
    pub(crate) const NONE: Mods = Mods(0);
    pub(crate) const SHIFT: Mods = Mods(1);
    pub(crate) const ALT: Mods = Mods(2);
    pub(crate) const CTRL: Mods = Mods(4);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeyKind {
    Press,
    Repeat,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Key {
    pub(crate) code: Code,
    pub(crate) mods: Mods,
    pub(crate) kind: KeyKind,
}

#[cfg(test)]
impl Key {
    pub(crate) fn press(code: Code) -> Self {
        Self::with(code, Mods::NONE)
    }

    pub(crate) fn with(code: Code, mods: Mods) -> Self {
        Self {
            code,
            mods,
            kind: KeyKind::Press,
        }
    }

    pub(crate) fn char(c: char) -> Self {
        Self::press(Code::Char(c))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Button {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MouseKind {
    Moved,
    Down(Button),
    Up(Button),
    Drag(Button),
    ScrollUp,
    ScrollDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Mouse {
    pub(crate) kind: MouseKind,
    pub(crate) column: u16,
    pub(crate) row: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Input {
    Key(Key),
    Mouse(Mouse),
    Resize,
    /// Focus, paste markers and anything the app does not use.
    Ignored,
}

/// Editing keys for a single-line text field (emacs style, like `tui-input`'s own backends).
pub(crate) fn to_request(key: Key) -> Option<InputRequest> {
    use InputRequest::{
        DeleteLine, DeleteNextChar, DeletePrevChar, DeletePrevWord, GoToEnd, GoToNextChar,
        GoToNextWord, GoToPrevChar, GoToPrevWord, GoToStart, InsertChar,
    };
    if key.kind == KeyKind::Release {
        return None;
    }
    match (key.code, key.mods) {
        (Code::Backspace, Mods::NONE) => Some(DeletePrevChar),
        (Code::Delete, Mods::NONE) => Some(DeleteNextChar),
        (Code::Left, Mods::NONE) => Some(GoToPrevChar),
        (Code::Right, Mods::NONE) => Some(GoToNextChar),
        (Code::Left, Mods::CTRL) => Some(GoToPrevWord),
        (Code::Right, Mods::CTRL) => Some(GoToNextWord),
        (Code::Home, Mods::NONE) | (Code::Char('a'), Mods::CTRL) => Some(GoToStart),
        (Code::End, Mods::NONE) | (Code::Char('e'), Mods::CTRL) => Some(GoToEnd),
        (Code::Char('u'), Mods::CTRL) => Some(DeleteLine),
        (Code::Char('w'), Mods::CTRL) => Some(DeletePrevWord),
        (Code::Char(c), Mods::NONE | Mods::SHIFT) => Some(InsertChar(c)),
        _ => None,
    }
}
