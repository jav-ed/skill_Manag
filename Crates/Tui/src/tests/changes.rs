//! The changes page behind the question of a sync, push, add or init.

use super::{Harness, phase_name, world};
use crate::app::Screen;
use crate::event::{Event, Job};
use crate::input::Code;
use crate::results::Kind;
use crate::screens::{DiffPage, Pending, Phase};

fn phase(ui: &Harness) -> &'static str {
    phase_name(&ui.app)
}

fn wait_diff(ui: &mut Harness) {
    ui.wait_for(
        "the changes",
        |app| matches!(&app.screen, Screen::Work(w) if matches!(w.phase, Phase::Diff(_))),
    );
}

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
    assert!(page.contains("Changes"), "{page}");
    assert!(page.contains("coding in projects/one"), "{page}");
    assert!(page.contains("-coding v1"), "{page}");
    assert!(page.contains("+coding v2"), "{page}");
    assert!(page.contains("ref.md (new file"), "{page}");
    assert!(
        !ui.world.exists("projects/two/.agents/skills/tmux"),
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

#[test]
fn the_changes_page_scrolls_and_stays_inside_its_text() {
    let mut ui = Harness::sized(world(), 80, 12);
    ui.open("Sync").wait_select();
    ui.code(Code::Enter).wait_confirm().press('v');
    wait_diff(&mut ui);
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

#[test]
fn a_report_of_another_read_is_ignored() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    ui.code(Code::Enter).wait_confirm().press('v');
    assert_eq!(phase(&ui), "diffing");
    let elsewhere = DiffPage {
        pending: Pending {
            kind: Kind::Sync,
            targets: Vec::new(),
            skills: 0,
            plan: None,
            preview: None,
            install: None,
            agents: None,
        },
        lines: Vec::new(),
        scroll: 0,
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

#[test]
fn add_and_init_show_what_they_would_create() {
    let mut ui = Harness::new(world());
    super::place_support::add_to(&mut ui, "plain");
    ui.press(' ').code(Code::Enter).wait_confirm().press('v');
    wait_diff(&mut ui);

    let page = ui.screen();
    assert!(page.contains("astro in projects/plain"), "{page}");
    assert!(page.contains("SKILL.md (new file"), "{page}");
    assert!(page.contains("+astro v2"), "{page}");
}

#[test]
fn the_header_arrow_comes_back_from_the_changes_to_the_question() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    ui.code(Code::Enter).wait_confirm().press('v');
    wait_diff(&mut ui);

    ui.click(crate::hit::Target::HeaderBack);

    assert_eq!(phase(&ui), "confirm");
}
