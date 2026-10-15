//! The pane beside the list of the sync, push and delete pages: which projects the skill under the cursor
//! is in, and which of them the run would change.

use super::{Harness, world};
use crate::input::Code;

/// The standard world plus two projects whose `coding` is already the vault's, one of them nested.
fn world_with_current_copies() -> skillmirror_testkit::World {
    let world = world();
    world
        .project_file("four/.agents/skills/coding/SKILL.md", "coding v2")
        .project_file("nested/five/.agents/skills/coding/SKILL.md", "coding v2");
    world
}

#[test]
fn the_pane_has_three_views_and_keeps_the_open_one_while_the_cursor_moves() {
    let mut ui = Harness::sized(world_with_current_copies(), 120, 30);
    ui.open("Sync").wait_select();
    ui.press('j');

    let affected = ui.screen();
    assert!(
        affected.contains("[Affected 2]  Others 2  All 4"),
        "{affected}"
    );
    assert!(affected.contains("one  outdated"), "{affected}");
    assert!(affected.contains("two  outdated"), "{affected}");
    assert!(
        !affected.contains("nested/five"),
        "up to date projects are in the other view:\n{affected}"
    );

    ui.code(Code::Tab);
    let others = ui.screen();
    assert!(others.contains("[Others 2]"), "{others}");
    assert!(
        others.contains("nested/five"),
        "the path is below the scan root:\n{others}"
    );
    assert!(!others.contains("outdated"), "{others}");

    ui.code(Code::Tab);
    let all = ui.screen();
    assert!(all.contains("Would change (2)"), "{all}");
    assert!(all.contains("Up to date (2)"), "{all}");

    ui.code(Code::Tab).press('k').code(Code::Tab);
    let moved = ui.screen();
    assert!(
        moved.contains("[Others 0]") && moved.contains("Every project is affected."),
        "the view stays on Others when the cursor goes to astro:\n{moved}"
    );
}

#[test]
fn the_delete_pane_names_every_project_the_skill_would_go_from() {
    let mut ui = Harness::sized(world_with_current_copies(), 120, 30);
    ui.open("Delete").wait_select();
    ui.press('/').type_text("coding").code(Code::Enter);

    let screen = ui.screen();

    assert!(
        screen.contains("Would be removed from 4 projects"),
        "{screen}"
    );
    for project in ["one", "two", "four", "nested/five"] {
        assert!(screen.contains(project), "{project} is named:\n{screen}");
    }
}

#[test]
fn a_wide_terminal_shows_the_projects_as_a_second_window_beside_the_list() {
    let mut ui = Harness::sized(world_with_current_copies(), 120, 30);
    ui.open("Sync").wait_select();
    // The list only knows its height after the first draw.
    ui.draw();
    ui.press('j').code(Code::Tab).code(Code::Tab);

    insta::assert_snapshot!(ui.screen());
}

#[test]
fn a_long_path_breaks_at_its_folders_and_never_inside_a_name() {
    let world = world();
    world.project_file(
        "a_long_folder_name/another_long_folder_name/third_long_folder/.agents/skills/coding/SKILL.md",
        "coding v1",
    );
    let mut ui = Harness::sized(world, 120, 30);
    ui.open("Sync").wait_select();
    ui.draw();
    ui.press('j');

    let screen = ui.screen();

    // 62 characters do not fit the pane in one row, so the path is split after a folder.
    assert!(screen.contains("another_long_folder_name/"), "{screen}");
    assert!(screen.contains("third_long_folder"), "{screen}");
}

#[test]
fn the_pane_scrolls_with_ctrl_d_and_u_and_starts_again_at_the_top_for_another_row() {
    let world = world();
    for n in 0..30 {
        world.project_file(
            &format!("many{n:02}/.agents/skills/coding/SKILL.md"),
            "coding v2",
        );
    }
    let mut ui = Harness::sized(world, 120, 20);
    ui.open("Sync").wait_select();
    ui.draw();
    ui.press('j').code(Code::Tab);

    let top = ui.screen();
    assert!(
        top.contains("many00") && top.contains("more lines"),
        "{top}"
    );

    ui.ctrl('d');
    let scrolled = ui.screen();
    assert!(!scrolled.contains("many00"), "{scrolled}");
    assert!(scrolled.contains("many05"), "{scrolled}");

    ui.ctrl('u');
    assert!(ui.screen().contains("many00"));

    // Every key press is drawn, which is where the pane notices another row.
    ui.ctrl('d').press('k');
    ui.draw();
    ui.press('j');
    assert!(
        ui.screen().contains("many00"),
        "another row starts at the top"
    );
}
