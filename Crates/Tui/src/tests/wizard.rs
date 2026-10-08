use super::{Harness, world};
use crate::app::Screen;
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

/// Presses `j` until the picker cursor is on the directory with this name.
fn move_to(ui: &mut Harness, name: &str) {
    let Screen::Setup(setup) = &ui.app.screen else {
        panic!("the wizard is not open")
    };
    let index = setup.picker.names.iter().position(|n| n == name).unwrap();
    for _ in 0..index {
        ui.press('j');
    }
}

/// Walks from the home directory of the world to its vault and chooses it.
fn choose_vault(ui: &mut Harness) {
    ui.press('h');
    move_to(ui, "vault");
    ui.press('l').code(Code::Enter).wait_checked();
}

#[test]
fn without_settings_the_wizard_opens_by_itself_and_saves_pointer_and_config() {
    let mut ui = Harness::unconfigured(world());
    assert_eq!(step(&ui), "vault", "no vault: the wizard starts at once");
    assert!(ui.screen().contains("→ Vault"));
    choose_vault(&mut ui);
    assert_eq!(step(&ui), "root");
    let page = ui.screen();
    assert!(page.contains("✓ Vault"), "{page}");
    ui.code(Code::Enter);
    assert_eq!(
        step(&ui),
        "mandatory",
        "the root from the vault config is where the picker starts"
    );
    let page = ui.screen();
    assert!(
        page.contains("[✓] coding") && page.contains("[ ] astro"),
        "{page}"
    );
    ui.press(' ');
    ui.code(Code::Enter);
    assert_eq!(step(&ui), "save");
    let page = ui.screen();
    assert!(
        page.contains("Mandatory") && page.contains("astro, coding, tmux"),
        "{page}"
    );
    ui.press('y');
    assert_eq!(step(&ui), "saved");
    assert!(ui.screen().contains("Saved"));

    let pointer = ui.world.read("home/config/skillmirror/vault");
    assert_eq!(pointer.trim(), ui.world.vault().display().to_string());
    let config = ui.world.read("vault/config.yaml");
    assert!(
        config.contains("  - astro") && config.contains("  - tmux"),
        "{config}"
    );

    ui.code(Code::Enter);
    assert_eq!(step(&ui), "not the wizard");
    ui.open("Sync").wait_select();
    assert!(
        ui.screen().contains("coding"),
        "the new settings are in use"
    );
}

#[test]
fn cancelling_the_wizard_writes_nothing() {
    let mut ui = Harness::unconfigured(world());
    choose_vault(&mut ui);
    ui.code(Code::Enter).code(Code::Enter);
    assert_eq!(step(&ui), "save");
    ui.press('n');
    assert_eq!(step(&ui), "not the wizard");
    assert!(!ui.world.exists("home/config/skillmirror/vault"));
    let config = ui.world.read("vault/config.yaml");
    assert!(!config.contains("astro"));
}

#[test]
fn escape_goes_back_one_step_and_q_leaves() {
    let mut ui = Harness::unconfigured(world());
    choose_vault(&mut ui);
    ui.code(Code::Enter);
    assert_eq!(step(&ui), "mandatory");
    ui.code(Code::Esc);
    assert_eq!(step(&ui), "root");
    ui.code(Code::Esc);
    assert_eq!(step(&ui), "vault");
    ui.press('q');
    assert_eq!(step(&ui), "not the wizard");
    assert!(matches!(ui.app.screen, Screen::Menu(_)));
}

#[test]
fn a_folder_that_is_not_a_vault_is_refused_with_the_reason() {
    let mut ui = Harness::unconfigured(world());
    ui.code(Code::Enter).wait_checked();
    assert_eq!(step(&ui), "vault", "the home folder holds no skills");
    let page = ui.screen();
    assert!(page.contains('✗'), "{page}");
}

#[test]
fn a_flag_that_overrides_the_saved_vault_is_mentioned_before_saving() {
    let mut ui = Harness::new(world());
    ui.open("Setup");
    choose_vault(&mut ui);
    ui.code(Code::Enter).code(Code::Enter);
    assert!(
        ui.screen().contains("wins over what you save"),
        "{}",
        ui.screen()
    );
}

#[test]
fn the_mouse_walks_the_picker_and_toggles_the_checklist() {
    let mut ui = Harness::unconfigured(world());
    ui.click(crate::hit::Target::PickerUp);
    let Screen::Setup(setup) = &ui.app.screen else {
        panic!("wizard")
    };
    assert!(
        setup.picker.names.contains(&"vault".to_string()),
        "the click moved to the parent"
    );
    let index = setup
        .picker
        .names
        .iter()
        .position(|n| n == "vault")
        .unwrap();
    ui.click(crate::hit::Target::PickerRow(index));
    ui.click(crate::hit::Target::PickerRow(index));
    ui.click(crate::hit::Target::PickerSelect).wait_checked();
    assert_eq!(step(&ui), "root");
    ui.click(crate::hit::Target::PickerSelect);
    assert_eq!(step(&ui), "mandatory");
    ui.click(crate::hit::Target::CheckRow(0));
    let Screen::Setup(setup) = &ui.app.screen else {
        panic!("wizard")
    };
    assert!(setup.skills.first().is_some_and(|s| s.checked));
    ui.code(Code::Enter);
    ui.click(crate::hit::Target::Button(0));
    assert_eq!(step(&ui), "saved");
}
