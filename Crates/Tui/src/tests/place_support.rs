//! Helpers of the add and init tests: the page that asks for a folder, and the vault config with links.

use super::Harness;
use crate::app::Screen;
use crate::input::Code;
use crate::screens::{Phase, Place};
use skillmirror_testkit::World;

pub(super) fn place(ui: &Harness) -> &Place {
    match &ui.app.screen {
        Screen::Place(place) => place,
        _ => panic!("not on the page that asks for a folder"),
    }
}

/// Walks the picker into the folder with this name below the one it shows.
pub(super) fn into(ui: &mut Harness, name: &str) {
    let at = place(ui)
        .picker
        .names
        .iter()
        .position(|n| n == name)
        .unwrap_or_else(|| panic!("{name} is not listed: {:?}", place(ui).picker.names));
    for _ in 0..at {
        ui.press('j');
    }
    ui.press('l');
}

/// Add: the project `name` below the scan root.
pub(super) fn add_to(ui: &mut Harness, name: &str) {
    ui.open("Add");
    into(ui, name);
    ui.code(Code::Enter).wait_select();
}

/// The vault config asks for the `claude` links.
pub(super) fn with_targets(world: &World) {
    world
        .vault_config_with(&["coding", "tmux"], "targets: [claude]")
        .commit_vault();
}

pub(super) fn selected_page(ui: &mut Harness) -> String {
    let page = ui.screen();
    assert!(matches!(&ui.app.screen, Screen::Work(w) if matches!(w.phase, Phase::Select)));
    page
}
