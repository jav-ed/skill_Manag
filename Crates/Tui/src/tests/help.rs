//! The help overlay: topics with headings, and what each key is good for.

use super::{Harness, world};
use crate::input::Code;

#[test]
fn the_help_of_a_page_has_a_heading_per_topic_and_says_what_the_keys_do() {
    let mut ui = Harness::sized(world(), 100, 60);
    ui.open("Sync").wait_select();
    ui.press('?');

    let sync = ui.screen();

    for heading in ["Move", "Choose", "Find", "Run", "General"] {
        assert!(sync.contains(heading), "{heading}:\n{sync}");
    }
    assert!(sync.contains("The projects of the row"), "{sync}");
    assert!(sync.contains("work out what a sync would write"), "{sync}");
    assert!(sync.contains("your scan root"), "{sync}");
    assert!(
        sync.contains("ctrl+d  ctrl+u  scroll"),
        "the label and its text do not touch:\n{sync}"
    );

    let mut ui = Harness::sized(world(), 100, 60);
    ui.open("Skills").wait_select();
    ui.press('?');
    let skills = ui.screen();
    assert!(
        !skills.contains("Choose"),
        "nothing is ticked on that page:\n{skills}"
    );
    assert!(skills.contains("The card"), "{skills}");
}

#[test]
fn a_question_mark_typed_into_the_filter_of_the_pane_is_a_letter() {
    let mut ui = Harness::sized(world(), 120, 30);
    ui.open("Sync").wait_select();
    ui.draw();
    ui.press('f').press('?');

    assert!(!ui.app.help, "typing does not open the help");
    assert!(ui.screen().contains("filter: ?"));
}

#[test]
fn a_help_that_does_not_fit_scrolls_and_any_other_key_closes_it() {
    let mut ui = Harness::sized(world(), 80, 14);
    ui.open("Sync").wait_select();
    ui.press('?');

    let top = ui.screen();
    assert!(top.contains("scroll"), "the border says it scrolls:\n{top}");
    assert!(top.contains("Move"), "{top}");

    for _ in 0..30 {
        ui.press('j');
    }
    let bottom = ui.screen();
    assert!(ui.app.help, "arrows do not close it");
    assert!(bottom.contains("General"), "{bottom}");
    assert!(!bottom.contains("Move"), "{bottom}");

    ui.code(Code::Esc);
    assert!(!ui.app.help);
}
