//! The only place that writes to the terminal. Colour is decided by `anstream`: off for pipes and `NO_COLOR`.
#![allow(clippy::print_stdout, clippy::print_stderr)]

#[macro_use]
mod macros;
mod author;
mod backup;
mod bridge;
mod diff;
mod doctor;
mod human;
mod info;
mod json;
mod lossy;
mod progress;
mod status;
mod style;
mod view;

use std::io::{BufRead, IsTerminal, Write};

pub(crate) use author::{AuthoredJson, Pointer, VaultInitJson, render_authored, render_vault_init};
pub(crate) use backup::{
    HistoryJson, HistoryRow, UndoJson, UndoRow, UndoStatus, render_history, render_undo,
};
pub(crate) use bridge::{BridgeJson, BridgeRow, BridgeStatus, render_bridge_notes, render_bridges};
pub(crate) use diff::{DiffJson, render_diff};
pub(crate) use doctor::{DoctorJson, render_doctor};
pub(crate) use human::{
    render_delete, render_installed, render_rows, render_summary, render_vault,
};
pub(crate) use info::{InfoJson, render_info};
pub(crate) use json::{DeleteJson, ListJson, RunJson, VaultJson};
pub(crate) use progress::ScanLine;
pub(crate) use status::{ProjectRow, StatusJson, StatusSummary, render_status};
pub(crate) use view::{DeleteRow, DeleteStatus, InstalledRow, Row, SkillRow, Summary, Tense};

/// Prints text to stdout.
pub(crate) fn print(text: &str) {
    anstream::print!("{text}");
}

/// Prints one line to stdout.
pub(crate) fn line(text: &str) {
    anstream::println!("{text}");
}

/// Prints text to stderr.
pub(crate) fn eprint(text: &str) {
    anstream::eprint!("{text}");
}

pub(crate) fn error_line(text: &str) {
    anstream::eprintln!(
        "{}error:{} {text}",
        style::ERROR.render(),
        style::ERROR.render_reset()
    );
}

pub(crate) fn hint_line(text: &str) {
    anstream::eprintln!(
        "{}hint:{} {text}",
        style::MUTED.render(),
        style::MUTED.render_reset()
    );
}

pub(crate) fn warn_line(text: &str) {
    anstream::eprintln!(
        "{}warning:{} {text}",
        style::WARNING.render(),
        style::WARNING.render_reset()
    );
}

/// Whether a person can answer a prompt.
pub(crate) fn can_prompt() -> bool {
    std::io::stdin().is_terminal()
}

/// Asks a yes or no question on stderr and reads the answer from stdin. Anything but `y` or `yes` is no.
pub(crate) fn confirm(question: &str) -> std::io::Result<bool> {
    anstream::eprint!("{question} [y/N] ");
    std::io::stderr().flush()?;
    let mut answer = String::new();
    std::io::stdin().lock().read_line(&mut answer)?;
    Ok(matches!(
        answer.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}
