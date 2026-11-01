//! The ways into the changes page and out of it: from the list and from the question, back to where it was
//! opened from, and what it shows when there is nothing to show.

use skillmirror_core::ops::SideReport;

use super::{open_the_list_changes_of_coding, open_the_question_changes, phase, wait_diff};
use crate::event::{Event, Job};
use crate::hit::Target;
use crate::input::Code;
use crate::results::Kind;
use crate::screens::{Back, DiffPage, DiffView, Labels};
use crate::tests::{Harness, column, place_support, row_with, world};

// Rule: `v` on the sync list shows what the sync would change in the skill under the cursor, the project's
// text on the left and the text after the sync on the right; looking writes nothing, and esc comes back to
// the list as it was left, cursor and ticks. 9/10: this is the way in the page was built for.
#[test]
fn v_on_the_list_shows_the_changes_of_the_skill_under_the_cursor_and_esc_comes_back() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    // The cursor on `coding`, which is not ticked any more: the list is not in its first state.
    ui.press('j').press(' ');
    let list = ui.screen();

    ui.press('v');
    assert_eq!(phase(&ui), "diffing");
    wait_diff(&mut ui);

    let page = ui.screen();
    assert!(page.contains("1 file in 2 projects, +1 −1"), "{page}");
    assert!(page.contains("the same change in 2 projects"), "{page}");
    assert!(
        !page.contains("astro"),
        "only the skill under the cursor: {page}"
    );
    let row = row_with(&page, "coding v1");
    assert!(row.contains("coding v2"), "both sides on one row: {row}");
    assert!(
        column(row, "coding v1") < column(row, "coding v2"),
        "the project's text is on the left: {row}"
    );
    assert_eq!(
        ui.world.read("projects/one/.agents/skills/coding/SKILL.md"),
        "coding v1",
        "looking writes nothing"
    );

    ui.code(Code::Esc);
    assert_eq!(phase(&ui), "select");
    assert_eq!(ui.screen(), list, "the list is as it was left");
}

// Rule: the question of a run can show the changes it would make, and esc comes back to the same question
// with the same plan: what was shown is what runs. 9/10: a plan that changed behind the page would be a lie.
#[test]
fn the_question_can_show_the_changes_and_come_back_to_the_same_plan() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    ui.code(Code::Enter).wait_confirm();
    assert!(
        ui.screen().contains("v view changes"),
        "the footer names the key"
    );

    ui.press('v');
    assert_eq!(phase(&ui), "diffing");
    wait_diff(&mut ui);

    let page = ui.screen();
    assert!(page.contains("3 files in 2 projects, +3 −2"), "{page}");
    assert!(page.contains("▌ astro   in projects/one"), "{page}");
    assert!(
        page.contains("▌ coding   the same change in 2 projects"),
        "{page}"
    );
    assert!(row_with(&page, "astro v1").contains("astro v2"), "{page}");
    let new_file = row_with(&page, "ref.md");
    assert!(new_file.contains("new file"), "{new_file}");
    assert!(
        row_with(&page, "│  1 + ref").contains('╱'),
        "a new file has nothing on the left: {page}"
    );
    assert!(
        !ui.world.exists("projects/one/.agents/skills/astro/ref.md"),
        "looking writes nothing"
    );

    ui.code(Code::Esc);
    assert_eq!(phase(&ui), "confirm", "back to the question");
    ui.press('y').wait_done();
    assert_eq!(
        ui.world.read("projects/one/.agents/skills/coding/SKILL.md"),
        "coding v2",
        "the plan that was shown is the plan that ran"
    );
}

// Rule: add and init show the files they would create, with nothing on the left. 7/10: their question has
// no other way to say what a skill brings into a new project.
#[test]
fn add_and_init_show_what_they_would_create() {
    let mut ui = Harness::new(world());
    place_support::add_to(&mut ui, "plain");
    ui.press(' ').code(Code::Enter).wait_confirm().press('v');
    wait_diff(&mut ui);

    let page = ui.screen();
    assert!(page.contains("▌ astro   in projects/plain"), "{page}");
    assert!(row_with(&page, "SKILL.md").contains("new file"), "{page}");
    let row = row_with(&page, "astro v2");
    assert!(row.contains('╱'), "nothing on the left: {row}");
    assert!(page.contains("now") && page.contains("after"), "{page}");
}

// Rule: the arrow in the header leaves the changes page for the place it was opened from, the question or
// the list. 7/10: it is a second way out that the keys do not exercise.
#[test]
fn the_header_arrow_leaves_the_changes_for_where_they_were_opened_from() {
    let mut ui = Harness::new(world());
    open_the_question_changes(&mut ui);
    ui.click(Target::HeaderBack);
    assert_eq!(phase(&ui), "confirm");

    let mut ui = Harness::new(world());
    open_the_list_changes_of_coding(&mut ui);
    ui.click(Target::HeaderBack);
    assert_eq!(phase(&ui), "select");
}

// Rule: a skill with nothing to bring in says so instead of showing an empty page, and esc leaves it. 7/10.
#[test]
fn a_skill_that_is_up_to_date_has_nothing_to_show() {
    let world = world();
    world
        .project_file("one/.agents/skills/coding/SKILL.md", "coding v2")
        .project_file("two/.agents/skills/coding/SKILL.md", "coding v2");
    let mut ui = Harness::new(world);
    open_the_list_changes_of_coding(&mut ui);

    let page = ui.screen();
    assert!(page.contains("Nothing would change."), "{page}");
    ui.code(Code::Esc);
    assert_eq!(phase(&ui), "select");
}

// Rule: the report of a read that was left is dropped, not shown on the page the user went to. 8/10: jobs
// answer late, and a page that opens by itself is the worst surprise.
#[test]
fn leaving_while_the_changes_are_read_drops_them() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    ui.code(Code::Enter).wait_confirm().press('v');
    assert_eq!(phase(&ui), "diffing");

    ui.press('q');
    assert_eq!(phase(&ui), "menu");
    let late = ui.next_event();
    ui.app.handle(late);

    assert_eq!(
        phase(&ui),
        "menu",
        "the report of a page that was left is dropped"
    );
}

// Rule: only the answer of the read the page waits for counts; an answer or a failure of another job changes
// nothing. 7/10.
#[test]
fn a_report_of_another_read_is_ignored() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    ui.code(Code::Enter).wait_confirm().press('v');
    assert_eq!(phase(&ui), "diffing");
    let elsewhere = DiffPage {
        back: Back::List,
        view: DiffView::new(SideReport::default(), Labels::of(Kind::Sync)),
    };

    ui.app.handle(Event::Job {
        id: 9999,
        job: Job::Diffed(Box::new(elsewhere)),
    });
    ui.app.handle(Event::Job {
        id: 9999,
        job: Job::Failed("old".to_string()),
    });

    assert_eq!(phase(&ui), "diffing", "only the read it waits for counts");
    wait_diff(&mut ui);
}

// Rule: a delete has no changes page: it has no plan to show, and `v` does nothing there. 7/10.
#[test]
fn a_delete_has_nothing_to_show() {
    let mut ui = Harness::new(world());
    ui.open("Delete").wait_select();
    ui.press(' ').code(Code::Enter);
    assert_eq!(phase(&ui), "confirm");
    assert!(!ui.screen().contains("view changes"));

    ui.press('v');

    assert_eq!(phase(&ui), "confirm", "no plan, no changes page");
}
