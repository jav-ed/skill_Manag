//! What the scan could not read is counted on the page and listed on request.

use super::{Harness, world};
use crate::app::Screen;
use crate::hit::Target;
use crate::input::Code;
use crate::screens::Phase;
use skillmirror_testkit::World;

/// The standard world plus the leftover of a run that died.
fn world_with_leftover() -> World {
    let world = world();
    world.project_file("one/.agents/.stage-4000000000-1-0/half/SKILL.md", "x");
    world
}

fn selecting(ui: &Harness) -> bool {
    matches!(&ui.app.screen, Screen::Work(w) if matches!(w.phase, Phase::Select))
}

#[test]
fn problems_are_counted_on_the_page_and_listed_on_demand() {
    let mut ui = Harness::new(world_with_leftover());
    ui.open("Sync").wait_select();
    let page = ui.screen();
    assert!(page.contains("1 scan problem (i)"), "{page}");
    assert!(
        page.contains("i problems"),
        "the footer names the key: {page}"
    );

    ui.press('i');

    let page = ui.screen();
    assert!(page.contains("Scan problems"), "{page}");
    assert!(page.contains(".stage-4000000000-1-0"), "{page}");
    assert!(page.contains("left over from an interrupted run"), "{page}");
}

#[test]
fn any_key_closes_the_list_and_does_nothing_else() {
    let mut ui = Harness::new(world_with_leftover());
    ui.open("Sync").wait_select();
    let before = ui.screen();
    ui.press('i');

    // `q` would leave the page and space would toggle a row if the list did not take the key.
    ui.press('q');

    assert!(selecting(&ui), "still on the page");
    assert_eq!(ui.screen(), before, "nothing was toggled either");
    ui.press('i').code(Code::Enter);
    assert!(
        selecting(&ui),
        "enter closes the list, it does not start a sync"
    );
    assert!(!ui.screen().contains("Scan problems"));
}

#[test]
fn a_click_closes_the_list() {
    let mut ui = Harness::new(world_with_leftover());
    ui.open("Sync").wait_select();
    ui.press('i');
    ui.draw();

    ui.click(Target::Dismiss);

    assert!(!ui.screen().contains("Scan problems"));
}

#[test]
fn a_clean_scan_has_no_note_and_i_does_nothing() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();

    ui.press('i');

    let page = ui.screen();
    assert!(!page.contains("scan problem"), "{page}");
    assert!(!page.contains("Scan problems"), "no empty box: {page}");
    assert!(!page.contains("i problems"), "{page}");
    // The key after `i` reaches the page: an open list would have taken it.
    ui.press(' ');
    assert!(ui.screen().contains("1 / 2 selected"), "{}", ui.screen());
}

#[test]
fn the_other_pages_show_the_note_too() {
    let mut ui = Harness::new(world_with_leftover());
    ui.open("List").wait_select();
    assert!(ui.screen().contains("1 scan problem (i)"));
    ui.press('i');
    assert!(ui.screen().contains("Scan problems"));
}
