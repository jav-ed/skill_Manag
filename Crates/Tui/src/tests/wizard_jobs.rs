//! Review round 2 for the setup wizard: the config is saved before the pointer (M5), looking at a folder
//! happens in the background (L12).

use super::{Harness, world};
use crate::app::Screen;
use crate::event::{Event, Job};
use crate::input::Code;
use crate::screens::Step;

fn step(ui: &Harness) -> &'static str {
    let Screen::Setup(setup) = &ui.app.screen else {
        return "not the wizard";
    };
    match setup.step {
        Step::Vault => "vault",
        Step::Root => "root",
        Step::Mandatory => "mandatory",
        Step::Save => "save",
        Step::Saved(_) => "saved",
    }
}

fn choose_vault(ui: &mut Harness) {
    ui.press('h');
    let Screen::Setup(setup) = &ui.app.screen else {
        panic!("the wizard is not open")
    };
    let index = setup
        .picker
        .names
        .iter()
        .position(|n| n == "vault")
        .unwrap();
    for _ in 0..index {
        ui.press('j');
    }
    ui.press('l').code(Code::Enter).wait_checked();
}

/// Walks the wizard from its first page to the save page with the defaults.
fn to_save_page(ui: &mut Harness) {
    choose_vault(ui);
    ui.code(Code::Enter).code(Code::Enter);
    assert_eq!(step(ui), "save");
}

/// Saves what the vault already says, so the settings are reloaded.
pub(super) fn save_unchanged_setup(ui: &mut Harness) {
    to_save_page(ui);
    ui.press('y');
    assert_eq!(step(ui), "saved");
    ui.code(Code::Enter);
}

#[test]
fn a_config_that_cannot_be_saved_leaves_the_pointer_where_it_was() {
    let mut ui = Harness::unconfigured(world());
    to_save_page(&mut ui);
    // The config turns unreadable after the wizard looked at it.
    ui.world.vault_file("config.yaml", "root: [unclosed\n");

    ui.press('y');

    let Screen::Setup(setup) = &ui.app.screen else {
        panic!("wizard")
    };
    assert!(matches!(&setup.step, Step::Saved(Err(_))), "the save fails");
    assert!(
        !ui.world.exists("home/config/skillmirror/vault"),
        "the pointer was written before the config was known to be good"
    );
}

#[test]
fn a_pointer_that_cannot_be_written_puts_the_config_back_as_it_was() {
    let mut ui = Harness::unconfigured(world());
    to_save_page(&mut ui);
    let before = ui.world.read("vault/config.yaml");
    // A file where the pointer's folder should be makes the pointer impossible to write.
    let config_home = ui.world.path().join("home/config");
    std::fs::create_dir_all(&config_home).unwrap();
    std::fs::write(config_home.join("skillmirror"), "in the way").unwrap();

    ui.press('y');

    let Screen::Setup(setup) = &ui.app.screen else {
        panic!("wizard")
    };
    assert!(matches!(&setup.step, Step::Saved(Err(_))), "the save fails");
    assert_eq!(ui.world.read("vault/config.yaml"), before);
}

#[test]
fn choosing_a_vault_returns_at_once_and_the_picker_keeps_still_meanwhile() {
    let mut ui = Harness::unconfigured(world());
    ui.press('h');
    let Screen::Setup(setup) = &ui.app.screen else {
        panic!("wizard")
    };
    let index = setup
        .picker
        .names
        .iter()
        .position(|n| n == "vault")
        .unwrap();
    for _ in 0..index {
        ui.press('j');
    }
    ui.press('l');
    let before = picker_cursor(&ui);

    ui.code(Code::Enter);

    let Screen::Setup(setup) = &ui.app.screen else {
        panic!("wizard")
    };
    assert!(
        setup.checking,
        "the look at the folder runs in the background"
    );
    assert_eq!(step(&ui), "vault");
    assert!(ui.screen().contains("Looking at the folder"));
    ui.press('j');
    assert_eq!(
        picker_cursor(&ui),
        before,
        "keys do not move the picker meanwhile"
    );
    ui.wait_checked();
    assert_eq!(step(&ui), "root");
}

fn picker_cursor(ui: &Harness) -> usize {
    let Screen::Setup(setup) = &ui.app.screen else {
        panic!("wizard")
    };
    setup.picker.view.cursor
}

#[test]
fn the_answer_for_a_folder_the_user_walked_away_from_is_ignored() {
    let mut ui = Harness::unconfigured(world());
    ui.press('h');
    let Screen::Setup(setup) = &ui.app.screen else {
        panic!("wizard")
    };
    let index = setup
        .picker
        .names
        .iter()
        .position(|n| n == "vault")
        .unwrap();
    for _ in 0..index {
        ui.press('j');
    }
    ui.press('l').code(Code::Enter);
    ui.press('q'); // leave the wizard while the folder is still being looked at

    // The answer arrives anyway.
    let event = ui.next_event();
    assert!(matches!(
        &event,
        Event::Job {
            job: Job::Checked(_),
            ..
        }
    ));
    ui.app.handle(event);

    assert_eq!(step(&ui), "not the wizard");
}
