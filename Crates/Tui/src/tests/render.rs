use super::{Harness, world};
use crate::input::Code;

#[test]
fn the_menu_lists_every_entry_with_the_detail_of_the_selected_one() {
    let mut ui = Harness::new(world());
    insta::assert_snapshot!(ui.screen());
}

#[test]
fn the_sync_page_shows_what_each_skill_will_do() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    insta::assert_snapshot!(ui.screen());
}

#[test]
fn the_delete_dialog_names_the_damage() {
    let mut ui = Harness::new(world());
    ui.open("Delete").wait_select();
    ui.press(' ').code(Code::Enter);
    insta::assert_snapshot!(ui.screen());
}

#[test]
fn the_results_page_lists_failures_and_collapses_successes() {
    let mut ui = Harness::new(world());
    ui.open("List").wait_select();
    ui.press('a').press('s').wait_done();
    insta::assert_snapshot!(ui.screen());
    ui.press('d');
    let detailed = ui.screen();
    assert!(
        detailed.contains("updated"),
        "d shows every project: {detailed}"
    );
}

#[test]
fn the_filter_highlights_a_fuzzy_match_and_shows_its_prompt() {
    let mut ui = Harness::new(world());
    ui.open("List").wait_select();
    ui.press('/').type_text("cdg");
    let page = ui.screen();
    assert!(page.contains("filter: cdg"), "{page}");
    assert!(page.contains("matching \"cdg\""), "{page}");
    assert!(page.contains("coding"), "{page}");
    assert!(!page.contains("astro"), "{page}");
}

#[test]
fn the_full_help_lists_the_keys_of_the_current_screen() {
    let mut ui = Harness::new(world());
    ui.open("List").wait_select();
    ui.press('?');
    let page = ui.screen();
    assert!(page.contains("keys"), "{page}");
    assert!(page.contains("space") && page.contains("toggle"), "{page}");
    assert!(page.contains("delete"), "{page}");
}

#[test]
fn no_screen_panics_on_a_tiny_terminal() {
    for (w, h) in [(1, 1), (10, 3), (30, 6), (60, 10)] {
        for label in ["Sync", "List", "Delete", "Push"] {
            let mut ui = Harness::sized(world(), w, h);
            ui.screen();
            ui.open(label).wait_select();
            ui.screen();
            ui.press('?');
            ui.screen();
            ui.press('x');
            ui.press(' ').code(Code::Enter);
            ui.screen();
        }
    }
}

#[test]
fn an_empty_vault_selection_explains_itself() {
    let world = world();
    for project in ["one", "two"] {
        std::fs::remove_dir_all(world.root().join(project).join(".agents/skills/coding")).unwrap();
    }
    std::fs::remove_dir_all(world.root().join("one/.agents/skills/astro")).unwrap();
    let mut ui = Harness::new(world);
    ui.open("Sync").wait_select();
    assert!(
        ui.screen()
            .contains("No matching skills found in any project.")
    );
}
