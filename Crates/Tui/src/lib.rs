//! The terminal interface of skillmirror: a persistent app with a menu and the sync, push, list and
//! delete screens. All disk work happens on background threads, so the interface never blocks.

mod backend;
mod binding;
mod error;
mod filter;
mod hit;
mod items;
mod jobs;
mod num;
mod preview;
mod results;
mod screens;
mod session;
mod theme;
mod undo;

mod app;
mod event;
mod input;
mod ui;

#[cfg(test)]
mod tests;

use std::io::IsTerminal;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::Duration;

pub use app::Launch;
pub use error::TuiError;

use app::{App, Effect};
use backend::AppTerminal;
use event::{Event, InputThread};

/// How often the spinner and the progress bar move.
const TICK: Duration = Duration::from_millis(80);

/// Runs the interface until the user quits. Needs a terminal on stdin and stdout.
pub fn run(launch: Launch) -> Result<(), TuiError> {
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return Err(TuiError::NoTerminal);
    }
    let (tx, rx) = mpsc::channel();
    let (mut terminal, source) = backend::start()?;
    let input = InputThread::spawn(tx.clone(), source);
    let mut app = App::new(launch, tx);
    let result = event_loop(&mut terminal, &mut app, &rx);
    drop(input);
    backend::stop(&mut terminal)?;
    result?;
    match app.failure() {
        Some(message) => Err(TuiError::Input(message.to_string())),
        None => Ok(()),
    }
}

fn event_loop(
    terminal: &mut AppTerminal,
    app: &mut App,
    rx: &Receiver<Event>,
) -> std::io::Result<()> {
    while !app.should_quit() {
        terminal.draw(|frame| ui::draw(frame, app))?;
        let first = if app.animating() {
            match rx.recv_timeout(TICK) {
                Ok(event) => Some(event),
                Err(RecvTimeoutError::Timeout) => {
                    app.on_tick();
                    None
                }
                Err(RecvTimeoutError::Disconnected) => break,
            }
        } else {
            rx.recv().ok()
        };
        let Some(first) = first else { continue };
        app.handle(first);
        // A mouse sweep or a resize floods the queue: handle everything queued, then draw once.
        while let Ok(event) = rx.try_recv() {
            app.handle(event);
        }
        for effect in app.take_effects() {
            perform(effect);
        }
    }
    Ok(())
}

/// Opening a browser is best effort: a missing `xdg-open` must not end the session.
fn perform(effect: Effect) {
    let Effect::OpenUrl(url) = effect;
    let spawned = Command::new("xdg-open")
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    if let Ok(mut child) = spawned {
        thread::spawn(move || drop(child.wait()));
    }
}
