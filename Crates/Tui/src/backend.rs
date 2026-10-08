//! The terminal: raw mode, alternate screen, mouse reporting and input conversion.
//!
//! Built on `termina` through `ratatui-termina`; `crossterm` 0.29 stalls on input bursts of about
//! 1 KB, which a mouse sweep or a paste produces (see the stack report).

use std::io::{self, Write as _};
use std::time::Duration;

use ratatui::Terminal;
use ratatui_termina::TerminaBackend;
use ratatui_termina::termina::escape::csi::{self, Csi};
use ratatui_termina::termina::event::{
    Event, KeyCode, KeyEventKind, Modifiers, MouseButton, MouseEventKind,
};
use ratatui_termina::termina::{EventReader, PlatformTerminal, Terminal as _};

use crate::input::{Button, Code, Input, Key, KeyKind, Mods, Mouse, MouseKind};

pub(crate) type AppTerminal = Terminal<TerminaBackend<PlatformTerminal>>;

/// Reads terminal events. Owned by the input thread.
pub(crate) struct InputSource(EventReader);

fn set(mode: csi::DecPrivateModeCode) -> Csi {
    Csi::Mode(csi::Mode::SetDecPrivateMode(csi::DecPrivateMode::Code(
        mode,
    )))
}

fn reset(mode: csi::DecPrivateModeCode) -> Csi {
    Csi::Mode(csi::Mode::ResetDecPrivateMode(csi::DecPrivateMode::Code(
        mode,
    )))
}

/// Raw mode, alternate screen and all-motion mouse reporting. A panic restores the terminal first.
pub(crate) fn start() -> io::Result<(AppTerminal, InputSource)> {
    use csi::DecPrivateModeCode::{
        AnyEventMouse, ButtonEventMouse, ClearAndEnableAlternateScreen, MouseTracking, RXVTMouse,
        SGRMouse,
    };
    let mut out = PlatformTerminal::new()?;
    out.enter_raw_mode()?;
    write!(
        out,
        "{}{}{}{}{}{}",
        set(ClearAndEnableAlternateScreen),
        set(MouseTracking),
        set(ButtonEventMouse),
        set(AnyEventMouse),
        set(RXVTMouse),
        set(SGRMouse)
    )?;
    out.flush()?;
    let reader = out.event_reader();
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let restore = "\x1b[?1006l\x1b[?1015l\x1b[?1003l\x1b[?1002l\x1b[?1000l\x1b[?1049l\x1b[?25h";
        drop(io::stderr().write_all(restore.as_bytes()));
        default_hook(info);
    }));
    let terminal = Terminal::new(TerminaBackend::new(out))?;
    Ok((terminal, InputSource(reader)))
}

/// Undoes [`start`]: mouse reporting off, main screen back, cursor shown.
pub(crate) fn stop(terminal: &mut AppTerminal) -> io::Result<()> {
    use csi::DecPrivateModeCode::{
        AnyEventMouse, ButtonEventMouse, ClearAndEnableAlternateScreen, MouseTracking, RXVTMouse,
        SGRMouse, ShowCursor,
    };
    let out = terminal.backend_mut();
    write!(
        out,
        "{}{}{}{}{}{}{}",
        reset(SGRMouse),
        reset(RXVTMouse),
        reset(AnyEventMouse),
        reset(ButtonEventMouse),
        reset(MouseTracking),
        reset(ClearAndEnableAlternateScreen),
        set(ShowCursor)
    )?;
    out.flush()
}

impl InputSource {
    /// Waits up to `timeout` for one event. Terminal replies (escape responses) are skipped.
    pub(crate) fn poll(&self, timeout: Duration) -> io::Result<Option<Input>> {
        let wanted = |event: &Event| !event.is_escape();
        if !self.0.poll(Some(timeout), wanted)? {
            return Ok(None);
        }
        Ok(Some(convert(&self.0.read(wanted)?)))
    }
}

fn convert(event: &Event) -> Input {
    match event {
        Event::Key(key) => convert_key(key).map_or(Input::Ignored, Input::Key),
        Event::Mouse(mouse) => convert_mouse(*mouse).map_or(Input::Ignored, Input::Mouse),
        Event::WindowResized(_) => Input::Resize,
        _ => Input::Ignored,
    }
}

fn convert_key(key: &ratatui_termina::termina::event::KeyEvent) -> Option<Key> {
    let code = match key.code {
        KeyCode::Char(c) => Code::Char(c),
        KeyCode::Enter => Code::Enter,
        KeyCode::Escape => Code::Esc,
        KeyCode::Tab => Code::Tab,
        KeyCode::Backspace => Code::Backspace,
        KeyCode::Delete => Code::Delete,
        KeyCode::Up => Code::Up,
        KeyCode::Down => Code::Down,
        KeyCode::Left => Code::Left,
        KeyCode::Right => Code::Right,
        KeyCode::Home => Code::Home,
        KeyCode::End => Code::End,
        KeyCode::PageUp => Code::PageUp,
        KeyCode::PageDown => Code::PageDown,
        _ => return None,
    };
    let mut mods = 0;
    for (flag, bit) in [
        (Modifiers::SHIFT, Mods::SHIFT),
        (Modifiers::ALT, Mods::ALT),
        (Modifiers::CONTROL, Mods::CTRL),
    ] {
        if key.modifiers.contains(flag) {
            mods |= bit.0;
        }
    }
    let kind = match key.kind {
        KeyEventKind::Press => KeyKind::Press,
        KeyEventKind::Repeat => KeyKind::Repeat,
        KeyEventKind::Release => KeyKind::Release,
    };
    Some(Key {
        code,
        mods: Mods(mods),
        kind,
    })
}

fn convert_mouse(mouse: ratatui_termina::termina::event::MouseEvent) -> Option<Mouse> {
    let button = |b: MouseButton| match b {
        MouseButton::Left => Button::Left,
        MouseButton::Right => Button::Right,
        MouseButton::Middle => Button::Middle,
    };
    let kind = match mouse.kind {
        MouseEventKind::Moved => MouseKind::Moved,
        MouseEventKind::Down(b) => MouseKind::Down(button(b)),
        MouseEventKind::Up(b) => MouseKind::Up(button(b)),
        MouseEventKind::Drag(b) => MouseKind::Drag(button(b)),
        MouseEventKind::ScrollUp => MouseKind::ScrollUp,
        MouseEventKind::ScrollDown => MouseKind::ScrollDown,
        _ => return None,
    };
    Some(Mouse {
        kind,
        column: mouse.column,
        row: mouse.row,
    })
}
