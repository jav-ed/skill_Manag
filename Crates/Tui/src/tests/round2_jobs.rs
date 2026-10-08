//! Review round 2 for the work pages: plan before asking (M1), one job at a time (M2), visible rows only
//! (L5), Ctrl-C during a job (L6), a dead keyboard (L8).

use super::{Harness, world};
use crate::event::{Event, Job};
use crate::hit::Target;
use crate::input::Code;
use crate::results::{Kind, Results};
use crate::screens::Phase;

const NOTES: &str = "projects/one/.agents/skills/astro/my_notes.md";

fn phase(ui: &Harness) -> &'static str {
    match &ui.app.screen {
        crate::app::Screen::Menu(_) => "menu",
        crate::app::Screen::Setup(_) => "setup",
        crate::app::Screen::History(_) => "history",
        crate::app::Screen::Place(_) => "place",
        crate::app::Screen::Work(w) => match w.phase {
            Phase::Loading => "loading",
            Phase::Failed(_) => "failed",
            Phase::Select => "select",
            Phase::Planning(_) => "planning",
            Phase::Confirm(_) => "confirm",
            Phase::Running { .. } => "running",
            Phase::Done(_) => "done",
        },
    }
}

fn results() -> Results {
    Results {
        kind: Kind::Sync,
        skills: Vec::new(),
        warnings: Vec::new(),
        backup: None,
        notes: Vec::new(),
    }
}

#[test]
fn sync_names_the_files_it_would_remove_and_writes_nothing_until_yes() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    ui.world
        .project_file("one/.agents/skills/astro/my_notes.md", "my notes");

    ui.code(Code::Enter).wait_confirm();

    let page = ui.screen();
    assert!(page.contains("Sync 2 skills in 2 projects?"), "{page}");
    assert!(page.contains("removes astro/my_notes.md"), "{page}");
    assert!(ui.world.exists(NOTES), "nothing is written before yes");
    ui.press('n');
    assert_eq!(phase(&ui), "select");
    assert!(ui.world.exists(NOTES), "no keeps everything");
}

#[test]
fn a_file_added_after_the_page_opened_is_never_deleted() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    ui.code(Code::Enter).wait_confirm();
    // The user saves a file into the project while the confirmation page is open.
    ui.world
        .project_file("one/.agents/skills/astro/late.md", "written meanwhile");

    ui.press('y').wait_done();

    assert!(
        ui.world.exists("projects/one/.agents/skills/astro/late.md"),
        "the plan the user saw did not contain this file, so it must survive"
    );
    let page = ui.screen();
    assert!(page.contains("1 failed"), "{page}");
}

#[test]
fn push_asks_as_well_and_a_clean_plan_goes_straight_through() {
    let mut ui = Harness::new(world());
    ui.open("Push").wait_select();
    ui.code(Code::Enter).wait_confirm();
    assert!(ui.screen().contains("Push 2 skills in 3 projects?"));
    ui.press('y').wait_done();

    ui.press('q');
    ui.open("Push").wait_select();
    ui.press('a').code(Code::Enter).wait_done();
    assert_eq!(phase(&ui), "done", "nothing to write: no question");
}

#[test]
fn a_second_enter_while_the_plan_is_made_does_not_cancel_it() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();

    ui.code(Code::Enter);
    assert_eq!(phase(&ui), "planning");
    ui.code(Code::Enter);

    assert_eq!(phase(&ui), "planning", "a double tap is not a way back");
    ui.wait_confirm();
    let crate::app::Screen::Work(work) = &ui.app.screen else {
        panic!("work page")
    };
    let Phase::Confirm(pending) = &work.phase else {
        panic!("confirm")
    };
    assert!(
        pending.preview.is_some(),
        "the numbers are worked out once, by the job"
    );
}

#[test]
fn the_header_arrow_does_not_leave_a_job_that_is_writing() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    ui.code(Code::Enter).wait_confirm().press('y');
    assert_eq!(phase(&ui), "running");

    ui.click(Target::HeaderBack);
    assert_eq!(phase(&ui), "running", "the arrow is ignored while writing");
    ui.wait_done();

    assert!(ui.screen().contains("Sync results"));
}

#[test]
fn leaving_while_the_plan_is_made_drops_the_plan() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    ui.code(Code::Enter);
    assert_eq!(phase(&ui), "planning");

    ui.click(Target::HeaderBack);
    assert_eq!(phase(&ui), "menu");
    ui.open("Delete");
    // The plan of the first page arrives now; it belongs to nobody any more.
    ui.wait_select();

    assert_eq!(phase(&ui), "select", "the delete page is not overwritten");
}

#[test]
fn reports_of_jobs_nobody_waits_for_change_nothing() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();

    for job in [
        Job::Finished(Box::new(results())),
        Job::Failed("late".to_string()),
        Job::Progress { done: 1, total: 2 },
    ] {
        ui.app.handle(Event::Job { id: 4242, job });
    }

    assert_eq!(phase(&ui), "select");
}

#[test]
fn enter_acts_on_the_selected_rows_that_are_visible_and_says_how_many_are_hidden() {
    let mut ui = Harness::new(world());
    ui.open("Delete").wait_select();
    ui.press('a');
    ui.press('/').type_text("astro").code(Code::Enter);

    let page = ui.screen();
    assert!(
        page.contains("3 / 3 selected (2 hidden by the filter)"),
        "{page}"
    );
    ui.code(Code::Enter);
    let dialog = ui.screen();
    assert!(dialog.contains("Delete 1 skill in 1 project?"), "{dialog}");
    ui.press('y').wait_done();

    assert!(!ui.world.exists("projects/one/.agents/skills/astro"));
    assert!(ui.world.exists("projects/one/.agents/skills/coding"));
    assert!(ui.world.exists("projects/one/.agents/skills/local-only"));
}

#[test]
fn ctrl_c_during_a_job_asks_once_before_it_quits() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    ui.code(Code::Enter).wait_confirm().press('y');
    assert_eq!(phase(&ui), "running");

    ui.ctrl('c');
    assert!(!ui.app.should_quit(), "the first press only warns");
    assert!(ui.screen().contains("ctrl+c again"), "{}", ui.screen());
    ui.ctrl('c');
    assert!(ui.app.should_quit(), "the second press quits");
}

#[test]
fn ctrl_c_quits_at_once_when_no_job_is_writing() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    ui.code(Code::Enter).wait_confirm().press('y').wait_done();

    ui.ctrl('c');

    assert!(ui.app.should_quit());
}

#[test]
fn a_keyboard_that_stopped_answering_ends_the_interface_with_the_reason() {
    let mut ui = Harness::new(world());

    ui.app.handle(Event::InputFailed("EIO".to_string()));

    assert!(ui.app.should_quit());
    assert_eq!(ui.app.failure(), Some("EIO"));
}

#[test]
fn a_scan_that_arrives_after_the_settings_changed_is_not_adopted() {
    let mut ui = Harness::new(world());
    ui.open("Sync"); // scan number 1 is on its way
    let late = ui.next_event(); // its report, kept back until after the change
    ui.press('q');
    ui.open("Setup");
    super::round2_wizard::save_unchanged_setup(&mut ui);

    ui.app.handle(late);
    ui.open("Sync");

    assert_eq!(phase(&ui), "loading", "a fresh scan, not the old one");
}
