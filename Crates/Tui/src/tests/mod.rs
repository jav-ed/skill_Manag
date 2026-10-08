//! Tests that drive the whole interface with synthetic events against a throwaway vault.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

mod add;
mod flow;
mod history;
mod init;
mod issues;
mod mouse;
mod place_support;
mod render;
mod round2_jobs;
mod round2_wizard;
mod wizard;

use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use skillmirror_core::config::{Dirs, EnvOverrides, Flags, Settings};
use skillmirror_testkit::World;
use unicode_width::UnicodeWidthStr;

use crate::app::{App, Launch, Screen};
use crate::event::Event;
use crate::input::{Code, Input, Key, Mods, Mouse, MouseKind};
use crate::screens::Phase;

pub(super) struct Harness {
    pub(super) world: World,
    pub(super) app: App,
    rx: Receiver<Event>,
    terminal: Terminal<TestBackend>,
}

impl Harness {
    pub(super) fn new(world: World) -> Self {
        Self::sized(world, 80, 24)
    }

    pub(super) fn sized(world: World, width: u16, height: u16) -> Self {
        let launch = launch(&world, true);
        Self::with_launch(world, launch, width, height)
    }

    /// A launch without vault or root flags: the settings come from the pointer file and the vault config.
    pub(super) fn unconfigured(world: World) -> Self {
        let launch = launch(&world, false);
        Self::with_launch(world, launch, 80, 24)
    }

    fn with_launch(world: World, launch: Launch, width: u16, height: u16) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            world,
            app: App::new(launch, tx),
            rx,
            terminal: Terminal::new(TestBackend::new(width, height)).unwrap(),
        }
    }

    pub(super) fn key(&mut self, key: Key) -> &mut Self {
        self.app.handle(Event::Term(Input::Key(key)));
        self
    }

    pub(super) fn press(&mut self, c: char) -> &mut Self {
        self.key(Key::char(c))
    }

    pub(super) fn code(&mut self, code: Code) -> &mut Self {
        self.key(Key::press(code))
    }

    pub(super) fn code_enter(&mut self) -> &mut Self {
        self.code(Code::Enter)
    }

    pub(super) fn ctrl(&mut self, c: char) -> &mut Self {
        self.key(Key::with(Code::Char(c), Mods::CTRL))
    }

    pub(super) fn type_text(&mut self, text: &str) -> &mut Self {
        for c in text.chars() {
            self.press(c);
        }
        self
    }

    pub(super) fn mouse(&mut self, kind: MouseKind, column: u16, row: u16) -> &mut Self {
        self.draw();
        self.app
            .handle(Event::Term(Input::Mouse(Mouse { kind, column, row })));
        self
    }

    /// Draws the app and returns the screen as plain text.
    pub(super) fn screen(&mut self) -> String {
        self.draw();
        dump(self.terminal.backend().buffer())
    }

    pub(super) fn draw(&mut self) {
        self.terminal
            .draw(|frame| crate::ui::draw(frame, &mut self.app))
            .unwrap();
    }

    /// Feeds job events to the app until the predicate holds. Fails after five seconds.
    pub(super) fn wait_for(&mut self, what: &str, done: impl Fn(&App) -> bool) -> &mut Self {
        while !done(&self.app) {
            let event = self
                .rx
                .recv_timeout(Duration::from_secs(5))
                .unwrap_or_else(|_| panic!("timed out waiting for {what}"));
            self.app.handle(event);
        }
        self
    }

    pub(super) fn wait_select(&mut self) -> &mut Self {
        self.wait_for(
            "the selection page",
            |app| matches!(&app.screen, Screen::Work(w) if matches!(w.phase, Phase::Select)),
        )
    }

    /// Waits until the setup wizard has looked at the folder it was given.
    pub(super) fn wait_checked(&mut self) -> &mut Self {
        self.wait_for(
            "the vault check",
            |app| matches!(&app.screen, Screen::Setup(s) if !s.checking),
        )
    }

    /// The next job report, taken out of the queue without being handled.
    pub(super) fn next_event(&mut self) -> Event {
        self.rx
            .recv_timeout(Duration::from_secs(5))
            .expect("a job report")
    }

    pub(super) fn wait_confirm(&mut self) -> &mut Self {
        self.wait_for(
            "the confirmation page",
            |app| matches!(&app.screen, Screen::Work(w) if matches!(w.phase, Phase::Confirm(_))),
        )
    }

    /// Enter on a sync or push page, then yes on the page that lists what would be written.
    pub(super) fn run_confirmed(&mut self) -> &mut Self {
        self.code(Code::Enter).wait_confirm().press('y').wait_done()
    }

    pub(super) fn wait_done(&mut self) -> &mut Self {
        self.wait_for(
            "the results page",
            |app| matches!(&app.screen, Screen::Work(w) if matches!(w.phase, Phase::Done(_))),
        )
    }

    /// Opens a menu entry by its label and waits for the scan.
    pub(super) fn open(&mut self, label: &str) -> &mut Self {
        for _ in 0..crate::screens::ENTRIES.len() {
            self.press('k');
        }
        let index = crate::screens::ENTRIES
            .iter()
            .position(|e| e.label == label)
            .unwrap();
        for _ in 0..index {
            self.press('j');
        }
        self.code(Code::Enter)
    }
}

pub(super) fn dump(buffer: &Buffer) -> String {
    let mut lines = Vec::new();
    for y in 0..buffer.area.height {
        let mut line = String::new();
        let mut x = 0;
        while x < buffer.area.width {
            let symbol = buffer[(x, y)].symbol();
            if let Some(visible) = hyperlink_text(symbol) {
                line.push_str(&visible);
                x += u16::try_from(visible.width()).unwrap().max(1);
            } else {
                line.push_str(symbol);
                x += 1;
            }
        }
        lines.push(line.trim_end().to_string());
    }
    lines.join("\n").trim_end().to_string()
}

/// The visible text of an OSC 8 hyperlink cell.
fn hyperlink_text(symbol: &str) -> Option<String> {
    let body = symbol.strip_prefix("\x1b]8;;")?;
    let (_, rest) = body.split_once("\x1b\\")?;
    let (text, _) = rest.split_once("\x1b]8;;")?;
    Some(text.to_string())
}

/// The standard world with an extra skill group and nothing else changed.
pub(super) fn world() -> World {
    World::standard()
}

impl Harness {
    /// The middle of the region a draw registered for `target`.
    pub(super) fn center_of(&mut self, target: crate::hit::Target) -> (u16, u16) {
        self.draw();
        let rect = self
            .app
            .hits
            .rect_of(target)
            .unwrap_or_else(|| panic!("{target:?} is not on screen"));
        (rect.x + rect.width / 2, rect.y + rect.height / 2)
    }

    pub(super) fn click(&mut self, target: crate::hit::Target) -> &mut Self {
        let (x, y) = self.center_of(target);
        self.mouse(MouseKind::Down(crate::input::Button::Left), x, y)
    }

    pub(super) fn hover(&mut self, target: crate::hit::Target) -> &mut Self {
        let (x, y) = self.center_of(target);
        self.mouse(MouseKind::Moved, x, y)
    }
}

fn launch(world: &World, with_flags: bool) -> Launch {
    let flags = if with_flags {
        Flags {
            vault: Some(world.vault()),
            root: Some(world.root()),
        }
    } else {
        Flags::default()
    };
    let dirs = Dirs::under(&world.path().join("home"));
    let settings = Settings::load(&flags, &EnvOverrides::default(), &dirs).unwrap();
    let reload_dirs = dirs.clone();
    Launch {
        settings,
        dirs,
        reload: Box::new(move || Settings::load(&flags, &EnvOverrides::default(), &reload_dirs)),
    }
}
