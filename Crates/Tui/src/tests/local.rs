//! The Override page: the skills of the project around the start folder against the vault, and replacing the
//! vault's copy of the ones changed there.

use super::{Harness, launch, world};
use crate::app::Screen;
use crate::input::Code;
use crate::screens::LocalPhase;

/// Waits until the page is in a phase.
fn wait_phase(ui: &mut Harness, what: &str, wanted: fn(&LocalPhase) -> bool) {
    ui.wait_for(
        what,
        move |app| matches!(&app.screen, Screen::Local(l) if wanted(&l.phase)),
    );
}

fn wait_list(ui: &mut Harness) {
    wait_phase(ui, "the list", |p| matches!(p, LocalPhase::List));
}

fn wait_confirm(ui: &mut Harness) {
    wait_phase(ui, "the question", |p| matches!(p, LocalPhase::Confirm(_)));
}

fn wait_done(ui: &mut Harness) {
    wait_phase(ui, "the results", |p| matches!(p, LocalPhase::Done(_)));
}

fn vault_status(ui: &Harness) -> String {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(ui.world.vault())
        .args(["status", "--porcelain"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn the_page_names_the_project_it_found_and_lists_every_skill_with_its_state() {
    let mut ui = Harness::sized(world(), 100, 20);
    ui.open("Override");
    wait_list(&mut ui);

    let screen = ui.screen();

    for expected in [
        "the project in this folder",
        "projects/one",
        "astro",
        "edited",
        "SKILL.md modified",
        "coding",
        "local-only",
        "not in the vault",
        "2 skills can be replaced",
    ] {
        assert!(screen.contains(expected), "{expected}:\n{screen}");
    }
    // The world lives in a folder with a random name.
    insta::assert_snapshot!(screen.replace(&ui.world.path().display().to_string(), "<world>"));
}

#[test]
fn enter_asks_then_replaces_the_vault_copy_stages_it_and_history_can_undo_it() {
    let mut ui = Harness::new(world());
    ui.open("Override");
    wait_list(&mut ui);
    // The cursor starts on the first edited skill, astro.
    ui.press(' ').code(Code::Enter);
    wait_confirm(&mut ui);

    let question = ui.screen();
    assert!(
        question.contains("Replace 1 skill in the vault?"),
        "{question}"
    );
    assert!(question.contains("astro"), "{question}");
    assert_eq!(ui.world.read("vault/web/astro/SKILL.md"), "astro v2");

    ui.press('y');
    wait_done(&mut ui);

    assert_eq!(ui.world.read("vault/web/astro/SKILL.md"), "astro v1");
    assert!(!ui.world.exists("vault/web/astro/ref.md"));
    assert_eq!(
        vault_status(&ui),
        "M  web/astro/SKILL.md\nD  web/astro/ref.md\n"
    );
    let done = ui.screen();
    assert!(done.contains("astro replaced in the vault"), "{done}");
    assert!(done.contains("History can undo it"), "{done}");
}

#[test]
fn s_updates_the_other_projects_that_have_the_skill_after_the_vault_has_it() {
    let world = world();
    world.project_file("one/.agents/skills/coding/SKILL.md", "coding v3");
    let mut ui = Harness::new(world);
    ui.open("Override");
    wait_list(&mut ui);
    // astro first, coding second: tick coding only.
    ui.press('j').press(' ').press('s').code(Code::Enter);
    wait_confirm(&mut ui);

    let question = ui.screen();
    assert!(question.contains("1 other project"), "{question}");

    ui.press('y');
    wait_done(&mut ui);

    assert_eq!(ui.world.read("vault/coding/SKILL.md"), "coding v3");
    assert_eq!(
        ui.world.read("projects/two/.agents/skills/coding/SKILL.md"),
        "coding v3"
    );
    let done = ui.screen();
    assert!(
        done.contains("Other projects: 1 skill folder written in 1 project"),
        "{done}"
    );
}

#[test]
fn several_projects_below_the_start_folder_are_listed_to_pick_from() {
    let world = world();
    let mut launch = launch(&world, true);
    launch.cwd = world.root();
    let mut ui = Harness::with_launch(world, launch, 100, 20);
    ui.open("Override");
    wait_phase(&mut ui, "the projects", |p| matches!(p, LocalPhase::Pick));

    let screen = ui.screen();
    assert!(
        screen.contains("projects below this folder: pick the one to look at"),
        "{screen}"
    );

    ui.code(Code::Enter);
    wait_list(&mut ui);
    assert!(ui.screen().contains("the project you picked"));
}
