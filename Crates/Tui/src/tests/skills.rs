//! The skills page: the vault's skills with a detail pane, read-only.

use super::{Harness, phase_name, world};
use crate::app::Screen;
use crate::input::Code;

fn work_of(ui: &Harness) -> &crate::screens::Work {
    let Screen::Work(work) = &ui.app.screen else {
        panic!("the skills page")
    };
    work
}

#[test]
fn the_page_lists_the_vault_with_group_and_a_summary_for_each_skill() {
    let mut ui = Harness::new(world());
    ui.open("Skills").wait_select();

    let screen = ui.screen();

    assert!(screen.contains("Vault — 3 skills"), "{screen}");
    for expected in ["coding", "tmux", "astro", "web", "mandatory"] {
        assert!(screen.contains(expected), "{expected} is listed:\n{screen}");
    }
    assert!(
        !screen.contains("selected"),
        "nothing is selected on a page that only looks:\n{screen}"
    );
}

#[test]
fn a_wide_terminal_shows_the_card_beside_the_list() {
    let mut ui = Harness::sized(world(), 120, 30);
    ui.open("Skills").wait_select();
    ui.press('/').type_text("astro").code(Code::Enter);

    insta::assert_snapshot!(ui.screen());
}

#[test]
fn a_tall_narrow_terminal_shows_the_card_under_the_list() {
    let mut ui = Harness::new(world());
    ui.open("Skills").wait_select();
    ui.press('/').type_text("tmux").code(Code::Enter);

    insta::assert_snapshot!(ui.screen());
}

#[test]
fn a_small_terminal_shows_the_list_alone() {
    let mut ui = Harness::sized(world(), 80, 14);
    ui.open("Skills").wait_select();

    let screen = ui.screen();

    assert!(
        !screen.contains("Projects"),
        "no room for a card:\n{screen}"
    );
    assert!(screen.contains("coding"), "{screen}");
}

#[test]
fn the_card_follows_the_cursor() {
    let mut ui = Harness::sized(world(), 120, 30);
    ui.open("Skills").wait_select();
    let first = ui.screen();
    ui.press('j');
    let second = ui.screen();

    assert_ne!(first, second, "another skill, another card");
    let name = |work: &crate::screens::Work| {
        let index = work.view.current().unwrap();
        work.items[index].name.clone()
    };
    assert!(
        second.contains(&format!("{}  (mandatory)", name(work_of(&ui))))
            || second.contains(&name(work_of(&ui))),
        "{second}"
    );
}

#[test]
fn the_filter_narrows_the_list_and_the_title_says_how_many_match() {
    let mut ui = Harness::new(world());
    ui.open("Skills").wait_select();
    ui.press('/').type_text("ast");

    let screen = ui.screen();

    assert!(screen.contains("Vault — 1 matching \"ast\""), "{screen}");
}

#[test]
fn space_select_all_and_enter_do_nothing() {
    let mut ui = Harness::new(world());
    ui.open("Skills").wait_select();

    ui.press(' ').press('a').code(Code::Enter);

    assert_eq!(phase_name(&ui.app), "select", "still the same page");
    assert!(work_of(&ui).selected.is_empty(), "nothing got selected");
    let rows = ui.screen();
    assert!(!rows.contains("[✓]"), "{rows}");
}

#[test]
fn clicking_a_row_moves_the_cursor_and_selects_nothing() {
    let mut ui = Harness::new(world());
    ui.open("Skills").wait_select();

    ui.click(crate::hit::Target::Row(2));

    assert_eq!(work_of(&ui).view.cursor, 2);
    assert!(work_of(&ui).selected.is_empty());
}

#[test]
fn the_help_line_names_only_keys_that_do_something_here() {
    let mut ui = Harness::new(world());
    ui.open("Skills").wait_select();

    let screen = ui.screen();
    let last = screen.lines().last().unwrap();

    assert!(last.contains("filter"), "{last}");
    assert!(!last.contains("toggle") && !last.contains("all"), "{last}");
}

#[test]
fn the_card_shows_the_description_and_the_files_git_does_not_track() {
    let world = world();
    world
        .vault_file(
            "coding/SKILL.md",
            "---\nname: coding\ndescription: Write careful code\n---\n",
        )
        .vault_file("coding/draft.md", "half finished")
        .commit_vault();
    world.vault_file("coding/notes.md", "not committed");
    let mut ui = Harness::sized(world, 120, 30);
    ui.open("Skills").wait_select();
    ui.press('/').type_text("coding").code(Code::Enter);

    let screen = ui.screen();

    assert!(screen.contains("Write careful code"), "{screen}");
    assert!(screen.contains("files (2): SKILL.md, draft.md"), "{screen}");
    assert!(
        screen.contains("not copied, git does not track: notes.md"),
        "{screen}"
    );
}

#[test]
fn a_skill_every_project_has_current_says_how_many_projects_it_is_in() {
    let world = world();
    let mut ui = Harness::sized(world, 120, 30);
    ui.open("Sync").wait_select();
    ui.run_confirmed();
    ui.press('q');
    ui.open("Skills").wait_select();
    ui.press('/').type_text("astro").code(Code::Enter);

    let screen = ui.screen();

    assert!(screen.contains("in 1 project"), "{screen}");
    assert!(screen.contains("up to date"), "{screen}");
}

#[test]
fn a_comparison_that_failed_is_a_problem_on_the_row_and_in_the_card() {
    let world = world();
    // The project copy of tmux is a link: the plan refuses to write through it.
    let skills = world.root().join("one/.agents/skills");
    std::fs::create_dir_all(&skills).unwrap();
    std::os::unix::fs::symlink(world.root().join("two"), skills.join("tmux")).unwrap();
    let mut ui = Harness::sized(world, 120, 30);
    ui.open("Skills").wait_select();
    ui.press('/').type_text("tmux").code(Code::Enter);

    let screen = ui.screen();

    assert!(screen.contains("1 problem"), "{screen}");
    assert!(screen.contains("symlink"), "{screen}");
}

#[test]
fn top_level_skills_come_first_then_the_groups() {
    let mut ui = Harness::new(world());
    ui.open("Skills").wait_select();

    let names: Vec<String> = work_of(&ui).items.iter().map(|i| i.name.clone()).collect();

    assert_eq!(names, ["coding", "tmux", "astro"]);
}

#[test]
fn a_mandatory_skill_that_is_outdated_lists_each_project_once() {
    let mut ui = Harness::sized(world(), 120, 30);
    ui.open("Skills").wait_select();
    let item = &work_of(&ui).items[0];
    assert_eq!(item.name, "coding");

    let projects = item
        .card
        .iter()
        .filter(|line| line.text.starts_with("projects/one"))
        .count();

    assert_eq!(projects, 1, "{:?}", item.card);
}

#[test]
fn a_failed_comparison_is_marked_as_a_problem_not_as_a_change() {
    let world = world();
    let skills = world.root().join("one/.agents/skills");
    std::fs::create_dir_all(&skills).unwrap();
    std::os::unix::fs::symlink(world.root().join("two"), skills.join("tmux")).unwrap();
    let mut ui = Harness::new(world);
    ui.open("Skills").wait_select();

    let tmux = work_of(&ui)
        .items
        .iter()
        .find(|i| i.name == "tmux")
        .unwrap();

    assert_eq!(
        tmux.note.as_ref().unwrap().tone,
        crate::items::Tone::Problem
    );
}

#[test]
fn a_card_that_does_not_fit_says_how_much_was_left_out() {
    let world = world();
    // Many projects have the skill, so the list under the heading is longer than the pane.
    for n in 0..12 {
        world.project_file(
            &format!("many{n:02}/.agents/skills/coding/SKILL.md"),
            "coding v1",
        );
    }
    let mut ui = Harness::sized(world, 80, 24);
    ui.open("Skills").wait_select();
    ui.press('/').type_text("coding").code(Code::Enter);

    let screen = ui.screen();

    assert!(screen.contains("more lines"), "{screen}");
    assert!(screen.contains("Projects"), "{screen}");
}
