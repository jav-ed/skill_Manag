//! The changes page: the old text beside the new, reached from the list (`v` on a skill) and from the question
//! of a run. Each test names the rule it protects and what it is worth (1 to 10): these run on every
//! `cargo test`.

mod entry;
mod reading;

use ratatui::style::Color;
use skillmirror_testkit::World;

use super::{Harness, column, phase_name, world};
use crate::app::Screen;
use crate::input::Code;
use crate::screens::Phase;

fn phase(ui: &Harness) -> &'static str {
    phase_name(&ui.app)
}

fn wait_diff(ui: &mut Harness) {
    ui.wait_for(
        "the changes",
        |app| matches!(&app.screen, Screen::Work(w) if matches!(w.phase, Phase::Diff(_))),
    );
}

/// The sync of everything, asked and then looked at: `astro` (two files) in one project and `coding` in two.
fn open_the_question_changes(ui: &mut Harness) {
    ui.open("Sync").wait_select();
    ui.code(Code::Enter).wait_confirm().press('v');
    wait_diff(ui);
}

/// The changes of `coding`, the second skill of the sync list, looked at from the list.
pub(super) fn open_the_list_changes_of_coding(ui: &mut Harness) {
    ui.open("Sync").wait_select();
    ui.press('j').press('v');
    wait_diff(ui);
}

/// `coding` is forty lines long and the vault's copy has three of them edited, far enough apart for the page
/// to fold what lies between.
fn long_world() -> World {
    let text = |edits: &[(usize, &str)]| -> String {
        (1..=40)
            .map(|n| match edits.iter().find(|(at, _)| *at == n) {
                Some((_, line)) => format!("{line}\n"),
                None => format!("line {n}\n"),
            })
            .collect()
    };
    let new = text(&[
        (5, "line 5 edited"),
        (20, "line 20 edited"),
        (35, "line 35 edited"),
    ]);
    let old = text(&[]);
    let world = world();
    world
        .vault_file("coding/SKILL.md", &new)
        .project_file("one/.agents/skills/coding/SKILL.md", &old)
        .project_file("two/.agents/skills/coding/SKILL.md", &old);
    world
}

/// `coding` has paragraphs that do not fit a row, edited in a few words, so that the page has to wrap.
pub(super) fn wide_lines_world() -> World {
    let paragraph = |tail: &str| {
        format!(
            "Customising is expected: most projects change the look, drop chrome they do not need and add their own components, {tail}\n"
        )
    };
    let old: String = ["one", "two", "three"]
        .iter()
        .map(|t| paragraph(t))
        .collect();
    let new: String = ["one", "2", "three", "four"]
        .iter()
        .map(|t| paragraph(t))
        .collect();
    let world = world();
    world
        .vault_file("coding/SKILL.md", &new)
        .project_file("one/.agents/skills/coding/SKILL.md", &old)
        .project_file("two/.agents/skills/coding/SKILL.md", &old);
    world
}

/// The background colour of the cell `shift` cells after the start of the first `needle` on the screen.
fn background(ui: &mut Harness, needle: &str, shift: usize) -> Color {
    let screen = ui.screen();
    let (y, row) = screen
        .lines()
        .enumerate()
        .find(|(_, row)| row.contains(needle))
        .unwrap_or_else(|| panic!("no row holds {needle:?}:\n{screen}"));
    let x = column(row, needle) + shift;
    ui.terminal.backend().buffer()[(u16::try_from(x).unwrap(), u16::try_from(y).unwrap())].bg
}
