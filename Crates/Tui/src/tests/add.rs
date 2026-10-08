//! Add through the interface: choose a project, choose skills, see the plan, write.

use super::place_support::{add_to, into, place, selected_page, with_targets};
use super::{Harness, world};
use crate::app::Screen;
use crate::hit::Target;
use crate::input::Code;

#[test]
fn add_installs_the_chosen_skill_into_the_chosen_project() {
    let mut ui = Harness::new(world());
    add_to(&mut ui, "two");

    let page = selected_page(&mut ui);
    assert!(page.contains("Select skills to add to"), "{page}");
    assert!(page.contains("projects/two"), "{page}");
    assert!(page.contains("0 / 3 selected"), "{page}");
    assert!(
        page.contains("already in the project"),
        "coding is there already: {page}"
    );
    assert!(page.contains("mandatory"), "tmux is mandatory: {page}");

    // astro, coding, tmux: tick tmux.
    ui.press('j').press('j').press(' ');
    ui.code(Code::Enter).wait_confirm();
    let page = ui.screen();
    assert!(page.contains("Add 1 skill to projects/two?"), "{page}");
    assert!(
        !ui.world.exists("projects/two/.agents/skills/tmux"),
        "nothing is written before yes"
    );

    ui.press('y').wait_done();
    let page = ui.screen();
    assert!(page.contains("Add results"), "{page}");
    assert!(page.contains("added to 1 project"), "{page}");
    assert_eq!(
        ui.world.read("projects/two/.agents/skills/tmux/SKILL.md"),
        "tmux v2"
    );
    assert!(
        !ui.world.exists("projects/one/.agents/skills/tmux"),
        "other projects are not touched"
    );
}

#[test]
fn add_makes_the_skills_directory_of_a_project_that_has_none() {
    let mut ui = Harness::new(world());
    add_to(&mut ui, "plain");
    ui.press(' ')
        .code(Code::Enter)
        .wait_confirm()
        .press('y')
        .wait_done();

    assert!(
        ui.world
            .exists("projects/plain/.agents/skills/astro/SKILL.md"),
        "the first skill of the list is astro"
    );
}

#[test]
fn add_refuses_a_project_whose_agents_folder_is_a_link() {
    let world = world();
    std::fs::create_dir_all(world.path().join("elsewhere")).unwrap();
    std::os::unix::fs::symlink(
        world.path().join("elsewhere"),
        world.root().join("plain/.agents"),
    )
    .unwrap();
    let mut ui = Harness::new(world);
    ui.open("Add");
    into(&mut ui, "plain");

    ui.code(Code::Enter);
    assert!(place(&ui).checking);
    ui.wait_for(
        "the look at the folder",
        |app| matches!(&app.screen, Screen::Place(p) if !p.checking),
    );

    let page = ui.screen();
    assert!(page.contains("must be a real directory"), "{page}");
    assert!(matches!(&ui.app.screen, Screen::Place(_)), "still asking");
}

#[test]
fn add_links_the_project_when_the_vault_config_asks_for_it() {
    let world = world();
    with_targets(&world);
    let mut ui = Harness::new(world);
    add_to(&mut ui, "two");
    ui.press('j').press('j').press(' ');
    ui.code(Code::Enter).wait_confirm().press('y').wait_done();

    let page = ui.screen();
    assert!(page.contains("Linked .claude/skills"), "{page}");
    assert_eq!(
        std::fs::read_link(ui.world.root().join("two/.claude/skills")).unwrap(),
        std::path::Path::new("../.agents/skills")
    );
}

#[test]
fn add_reports_a_real_claude_folder_and_keeps_it() {
    let world = world();
    with_targets(&world);
    world.project_file("two/.claude/skills/mine/SKILL.md", "mine");
    let mut ui = Harness::new(world);
    add_to(&mut ui, "two");
    ui.press('j').press('j').press(' ');
    ui.code(Code::Enter).wait_confirm().press('y').wait_done();

    let page = ui.screen();
    assert!(page.contains("real folder"), "{page}");
    assert_eq!(
        ui.world.read("projects/two/.claude/skills/mine/SKILL.md"),
        "mine"
    );
    assert_eq!(
        ui.world.read("projects/two/.agents/skills/tmux/SKILL.md"),
        "tmux v2",
        "the install itself went through"
    );
}

#[test]
fn leaving_the_folder_page_and_a_late_check_do_not_open_a_page() {
    let mut ui = Harness::new(world());
    ui.open("Add");
    into(&mut ui, "two");
    ui.code(Code::Enter);
    assert!(place(&ui).checking);

    ui.code(Code::Esc);
    assert!(matches!(&ui.app.screen, Screen::Menu(_)));
    let late = ui.next_event();
    ui.app.handle(late);

    assert!(
        matches!(&ui.app.screen, Screen::Menu(_)),
        "the answer about a folder the user left is dropped"
    );
}

#[test]
fn a_mouse_click_chooses_the_folder_for_add() {
    let mut ui = Harness::new(world());
    ui.open("Add");
    into(&mut ui, "three");

    ui.click(Target::PickerSelect).wait_select();

    assert!(ui.screen().contains("projects/three"));
}

#[test]
fn the_menu_cursor_comes_back_to_the_entry_of_a_finished_add() {
    let mut ui = Harness::new(world());
    add_to(&mut ui, "two");
    ui.press(' ')
        .code(Code::Enter)
        .wait_confirm()
        .press('y')
        .wait_done();

    ui.press('q');

    let Screen::Menu(menu) = &ui.app.screen else {
        panic!("menu")
    };
    assert_eq!(crate::screens::ENTRIES[menu.cursor].label, "Add");
}
