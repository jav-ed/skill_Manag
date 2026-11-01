//! Reading the changes page: the colours, the keys that walk it, folds and modes, the mouse, and what it does
//! at every size.

use super::{
    background, long_world, open_the_list_changes_of_coding, open_the_question_changes, phase,
    wide_lines_world,
};
use crate::hit::Target;
use crate::input::{Button, Code, MouseKind};
use crate::screens::diff::{MODE, NEXT};
use crate::tests::{Harness, row_with, world};
use crate::theme;

// Rule: the tint says which side a line is on and what changed in it: the removed line is red on the left,
// the added line green on the right, the changed word brighter than the rest of its line. 7/10: colour is how
// a diff is read, and no text snapshot can see it.
#[test]
fn removed_text_is_red_and_added_text_green_with_the_changed_word_brighter() {
    let mut ui = Harness::new(world());
    open_the_list_changes_of_coding(&mut ui);

    // `- coding v1`: the word `coding` starts two cells in, `v1` nine.
    assert_eq!(background(&mut ui, "- coding v1", 2), theme::REMOVED_LINE);
    assert_eq!(background(&mut ui, "- coding v1", 9), theme::REMOVED_WORD);
    assert_eq!(background(&mut ui, "+ coding v2", 2), theme::ADDED_LINE);
    assert_eq!(background(&mut ui, "+ coding v2", 9), theme::ADDED_WORD);
}

// Rule: n and p walk the changes one by one, g and G go to the ends, and a page that is at the last change
// stays there instead of starting again. 8/10: on a long diff these keys are the way through it.
#[test]
fn n_and_p_walk_the_changes_and_g_and_capital_g_go_to_the_ends() {
    let mut ui = Harness::sized(long_world(), 100, 30);
    open_the_question_changes(&mut ui);
    let top = ui.screen();
    assert!(top.contains("change 1 of 5"), "{top}");

    ui.press('n');
    let second = ui.screen();
    assert!(second.contains("change 2 of 5"), "{second}");
    ui.press('n');
    let end = ui.screen();
    assert!(
        end.contains("line 35 edited") && end.contains("line 40") && end.contains("change 5 of 5"),
        "{end}"
    );
    ui.press('n');
    assert_eq!(ui.screen(), end, "at the last change the page stays");

    ui.press('p');
    assert_eq!(ui.screen(), second, "p goes back one change");
    ui.press('G');
    assert_eq!(ui.screen(), end);
    ui.press('g');
    assert_eq!(ui.screen(), top);
}

// Rule: the file keys and the skill keys put the next head at the top, and the line under the title names the
// skill and file that are there. 7/10: without it a long page does not say where it is.
#[test]
fn the_file_and_skill_keys_jump_from_head_to_head() {
    let mut ui = Harness::sized(long_world(), 100, 30);
    open_the_question_changes(&mut ui);
    let top = ui.screen();

    ui.press(']');
    assert!(ui.screen().contains("astro  SKILL.md"), "first file");
    ui.press(']');
    assert!(ui.screen().contains("astro  ref.md"), "second file");
    ui.press(']');
    assert!(ui.screen().contains("coding  SKILL.md"), "third file");
    ui.press('[');
    assert!(ui.screen().contains("astro  ref.md"), "back to the second");

    ui.press('g').press('}');
    let second_skill = ui.screen();
    assert!(
        second_skill.contains("▌ coding") && !second_skill.contains("▌ astro"),
        "the head of the next skill is at the top: {second_skill}"
    );
    ui.press('{');
    assert_eq!(ui.screen(), top);
}

// Rule: `c` shows whole files or folds the lines far from a change, and `s` puts the old lines over the new
// instead of beside them; both can be switched back. 8/10: they are the two ways to read a long diff.
#[test]
fn folds_and_the_two_modes_can_be_switched_and_switched_back() {
    let mut ui = Harness::sized(long_world(), 100, 30);
    open_the_question_changes(&mut ui);
    ui.press('}');
    let folded = ui.screen();
    assert!(folded.contains("8 lines unchanged"), "{folded}");
    assert!(!folded.contains("line 12"), "{folded}");

    ui.press('c');
    let whole = ui.screen();
    assert!(
        whole.contains("line 12") && !whole.contains("lines unchanged"),
        "{whole}"
    );
    ui.press('c');
    assert_eq!(ui.screen(), folded, "folded again");

    ui.press('s');
    let inline = ui.screen();
    assert!(
        !row_with(&inline, "- line 5").contains("line 5 edited"),
        "one under the other: {inline}"
    );
    assert!(inline.contains("+ line 5 edited"), "{inline}");
    ui.press('s');
    assert_eq!(ui.screen(), folded, "side by side again");
}

// Rule: everything the keys do can be done with the mouse: the wheel scrolls, the header buttons jump and
// switch, a click on a fold opens that fold, a click on the ruler goes to that part. 7/10: each one is a hit
// target that the drawing registers and the handler must know.
#[test]
fn the_wheel_the_buttons_the_folds_and_the_ruler_answer_the_mouse() {
    let mut ui = Harness::sized(long_world(), 100, 30);
    open_the_question_changes(&mut ui);
    let top = ui.screen();

    ui.mouse(MouseKind::ScrollDown, 10, 10);
    assert_ne!(ui.screen(), top, "the wheel scrolls down");
    ui.mouse(MouseKind::ScrollUp, 10, 10);
    assert_eq!(ui.screen(), top, "and up");

    ui.click(Target::DiffButton(NEXT));
    let next = ui.screen();
    assert!(next.contains("change 2 of 5"), "the next button: {next}");
    ui.click(Target::DiffRuler);
    let middle = ui.screen();
    assert!(
        middle != next && !middle.contains("▌ astro"),
        "a click on the ruler goes to that part: {middle}"
    );

    ui.click(Target::DiffButton(MODE));
    let inline = ui.screen();
    assert!(inline.contains("split [inline]"), "{inline}");
    ui.click(Target::DiffButton(MODE));

    ui.press('g').press('}');
    let folded = ui.screen();
    let (y, _) = folded
        .lines()
        .enumerate()
        .find(|(_, row)| row.contains("8 lines unchanged"))
        .unwrap_or_else(|| panic!("no fold:\n{folded}"));
    ui.mouse(MouseKind::Down(Button::Left), 20, u16::try_from(y).unwrap());
    let opened = ui.screen();
    assert!(
        opened.contains("line 9") && opened.contains("line 16"),
        "the fold that was clicked is open: {opened}"
    );
}

// Rule: below 70 columns two columns are too narrow to read, so the sides go one under the other; when the
// terminal grows the two columns come back. 7/10: a resize while the page is open is where layouts break.
#[test]
fn a_narrow_terminal_puts_the_sides_one_under_the_other_and_a_wide_one_brings_them_back() {
    let mut ui = Harness::sized(world(), 60, 24);
    open_the_list_changes_of_coding(&mut ui);

    let narrow = ui.screen();
    assert!(
        !row_with(&narrow, "coding v1").contains("coding v2") && narrow.contains("coding v2"),
        "{narrow}"
    );

    ui.terminal.backend_mut().resize(100, 24);
    let wide = ui.screen();
    assert!(row_with(&wide, "coding v1").contains("coding v2"), "{wide}");
}

// Rule: the page draws at any size a terminal can have, in both modes, folded or whole, with long lines
// wrapped, and at every size it was opened or resized to; a size too small for it says so. 8/10: a panic here
// takes the whole program down on a resize.
#[test]
fn the_changes_page_survives_every_terminal_size() {
    let mut ui = Harness::sized(wide_lines_world(), 100, 30);
    open_the_question_changes(&mut ui);
    for (w, h) in [
        (100, 30),
        (1, 1),
        (10, 3),
        (29, 8),
        (30, 4),
        (45, 6),
        (69, 24),
        (70, 24),
        (71, 5),
        (200, 60),
        (31, 40),
    ] {
        ui.terminal.backend_mut().resize(w, h);
        for keys in [
            "", "s", "c", "n", "p", "G", "g", "}", "]", "{", "[", "s", "c",
        ] {
            for key in keys.chars() {
                ui.press(key);
            }
            ui.screen();
        }
        // Not on the first row, where the arrow of the header is: it leaves the page.
        if h >= 3 {
            ui.mouse(MouseKind::Down(Button::Left), w / 2, (h / 2).max(1));
        }
        ui.mouse(MouseKind::ScrollDown, w / 2, h / 2);
        ui.screen();
        assert_eq!(phase(&ui), "diff", "still on the page at {w}x{h}");
    }
    ui.terminal.backend_mut().resize(29, 10);
    let page = ui.screen();
    assert!(
        page.contains("narrow"),
        "a size that cannot hold the page says so: {page}"
    );
}

// Rule: a long page scrolls by pages and stops at both ends. 7/10: the bounds are the usual place for an
// off-by-one, and a page that scrolls past its text shows nothing.
#[test]
fn the_changes_page_scrolls_and_stays_inside_its_text() {
    let mut ui = Harness::sized(world(), 80, 12);
    open_the_question_changes(&mut ui);
    let top = ui.screen();

    ui.code(Code::PageDown);
    assert_ne!(ui.screen(), top, "page down moved the text");
    for _ in 0..40 {
        ui.code(Code::PageDown);
    }
    let bottom = ui.screen();
    assert!(bottom.contains("Changes"), "the heading stays: {bottom}");
    ui.code(Code::PageDown);
    assert_eq!(ui.screen(), bottom, "the end of the text stops the scroll");
    for _ in 0..40 {
        ui.code(Code::PageUp);
    }
    assert_eq!(ui.screen(), top);
}
