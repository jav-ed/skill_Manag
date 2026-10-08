//! The history page: the backup runs, and undoing one of them.

use skillmirror_core::backup::Backups;
use skillmirror_core::config::Dirs;

use super::{Harness, world};
use crate::app::{App, Screen};
use crate::event::{Event, Job};
use crate::hit::Target;
use crate::input::{Button, Code, MouseKind};
use crate::screens::HistoryPhase;

const CODING: &str = "projects/one/.agents/skills/coding/SKILL.md";

fn phase(ui: &Harness) -> &'static str {
    match &ui.app.screen {
        Screen::History(h) => match h.phase {
            HistoryPhase::Loading => "loading",
            HistoryPhase::Failed(_) => "failed",
            HistoryPhase::List => "list",
            HistoryPhase::Planning => "planning",
            HistoryPhase::Confirm(_) => "confirm",
            HistoryPhase::Running { .. } => "running",
            HistoryPhase::Done(_) => "done",
        },
        Screen::Menu(_) => "menu",
        Screen::Work(_) => "work",
        Screen::Place(_) => "place",
        Screen::Setup(_) => "setup",
    }
}

fn in_phase(app: &App, wanted: fn(&HistoryPhase) -> bool) -> bool {
    matches!(&app.screen, Screen::History(h) if wanted(&h.phase))
}

impl Harness {
    fn wait_list(&mut self) -> &mut Self {
        self.wait_for("the list of runs", |app| {
            in_phase(app, |p| matches!(p, HistoryPhase::List))
        })
    }

    fn wait_history_confirm(&mut self) -> &mut Self {
        self.wait_for("the question about the undo", |app| {
            in_phase(app, |p| matches!(p, HistoryPhase::Confirm(_)))
        })
    }

    fn wait_history_done(&mut self) -> &mut Self {
        self.wait_for("the results of the undo", |app| {
            in_phase(app, |p| matches!(p, HistoryPhase::Done(_)))
        })
    }

    /// One sync through the sync page, and back to the menu.
    fn sync_once(&mut self) -> &mut Self {
        self.open("Sync").wait_select();
        self.run_confirmed();
        self.press('q')
    }
}

#[test]
fn an_empty_store_says_so() {
    let mut ui = Harness::new(world());

    ui.open("History").wait_list();

    let page = ui.screen();
    assert!(page.contains("0 runs"), "{page}");
    assert!(page.contains("No backups yet"), "{page}");
    ui.code(Code::Enter);
    assert_eq!(phase(&ui), "list", "nothing to undo: enter does nothing");
}

#[test]
fn the_runs_are_listed_and_undo_puts_the_old_copies_back() {
    let mut ui = Harness::new(world());
    ui.sync_once();
    assert_eq!(ui.world.read(CODING), "coding v2");

    ui.open("History").wait_list();
    let page = ui.screen();
    assert!(page.contains("1 run"), "{page}");
    assert!(page.contains("sync"), "{page}");

    ui.code(Code::Enter).wait_history_confirm();
    let page = ui.screen();
    assert!(page.contains("Undo the sync of"), "{page}");
    assert!(
        page.contains("Puts back 3 skill folders in 2 projects."),
        "{page}"
    );
    assert_eq!(ui.world.read(CODING), "coding v2", "nothing before yes");

    ui.press('y').wait_history_done();
    let page = ui.screen();
    assert!(page.contains("Undo results"), "{page}");
    assert!(page.contains("restored coding in"), "{page}");
    assert!(page.contains("undo it again to redo this undo"), "{page}");
    assert_eq!(ui.world.read(CODING), "coding v1");
}

#[test]
fn the_undo_is_a_run_of_its_own_and_undoing_it_redoes_the_sync() {
    let mut ui = Harness::new(world());
    ui.sync_once();
    ui.open("History").wait_list();
    ui.code(Code::Enter)
        .wait_history_confirm()
        .press('y')
        .wait_history_done();
    ui.press('q');
    assert_eq!(phase(&ui), "menu");

    ui.open("History").wait_list();
    let page = ui.screen();
    assert!(page.contains("undo"), "{page}");
    ui.code(Code::Enter)
        .wait_history_confirm()
        .press('y')
        .wait_history_done();

    assert_eq!(
        ui.world.read(CODING),
        "coding v2",
        "the second undo is a redo"
    );
}

#[test]
fn no_changes_nothing_and_leaves_the_run_where_it_was() {
    let mut ui = Harness::new(world());
    ui.sync_once();
    ui.open("History").wait_list();
    ui.code(Code::Enter).wait_history_confirm();

    ui.press('n');

    assert_eq!(phase(&ui), "list");
    assert_eq!(ui.world.read(CODING), "coding v2");
    let backups = Backups::in_dirs(&Dirs::under(&ui.world.path().join("home")));
    assert_eq!(backups.run_ids().unwrap().len(), 1, "no run was started");
}

#[test]
fn a_run_that_cannot_be_read_is_listed_with_the_reason_and_cannot_be_undone() {
    let mut ui = Harness::new(world());
    ui.sync_once();
    let backups = Backups::in_dirs(&Dirs::under(&ui.world.path().join("home")));
    let id = backups.run_ids().unwrap().remove(0);
    std::fs::write(backups.root().join(&id).join("0/entry.json"), "not json").unwrap();

    ui.open("History").wait_list();

    let page = ui.screen();
    assert!(page.contains("cannot be read"), "{page}");
    ui.code(Code::Enter);
    assert_eq!(phase(&ui), "list", "a broken run is not offered for undo");
}

#[test]
fn leaving_while_the_plan_is_made_drops_it() {
    let mut ui = Harness::new(world());
    ui.sync_once();
    ui.open("History").wait_list();
    ui.code(Code::Enter);
    assert_eq!(phase(&ui), "planning");

    ui.press('q');
    assert_eq!(phase(&ui), "menu");
    // The report of the plan arrives late and must not open a question on the menu.
    let late = ui.next_event();
    ui.app.handle(late);

    assert_eq!(phase(&ui), "menu");
}

#[test]
fn a_page_left_before_the_runs_arrive_ignores_them() {
    let mut ui = Harness::new(world());
    ui.open("History");
    assert_eq!(phase(&ui), "loading");
    ui.press('q');

    let late = ui.next_event();
    ui.app.handle(late);

    assert_eq!(phase(&ui), "menu");
}

#[test]
fn nothing_leaves_or_starts_while_the_undo_writes() {
    let mut ui = Harness::new(world());
    ui.sync_once();
    ui.open("History").wait_list();
    ui.code(Code::Enter).wait_history_confirm().press('y');
    assert_eq!(phase(&ui), "running");

    ui.press('q').code(Code::Enter).click(Target::HeaderBack);
    assert_eq!(phase(&ui), "running", "keys and the arrow do not leave it");
    ui.ctrl('c');
    assert!(!ui.app.should_quit(), "the first ctrl+c only warns");

    ui.wait_history_done();
    assert_eq!(ui.world.read(CODING), "coding v1");
}

#[test]
fn the_mouse_picks_a_run_and_a_second_click_asks_about_it() {
    let mut ui = Harness::new(world());
    ui.sync_once();
    ui.open("History").wait_list();

    ui.click(Target::Row(0));
    assert_eq!(
        phase(&ui),
        "planning",
        "the only run is under the cursor already"
    );
    ui.wait_history_confirm();
    ui.click(Target::Button(1));
    assert_eq!(phase(&ui), "list");

    ui.wait_list().click(Target::Row(0)).wait_history_confirm();
    // A click beside the dialog is a click behind it.
    ui.mouse(MouseKind::Down(Button::Left), 1, 5);
    assert_eq!(phase(&ui), "list");
    assert_eq!(ui.world.read(CODING), "coding v2");
}

#[test]
fn the_newest_run_comes_first() {
    let mut ui = Harness::new(world());
    ui.sync_once();
    ui.open("Delete").wait_select();
    ui.press(' ').code(Code::Enter).press('y').wait_done();
    ui.press('q');

    ui.open("History").wait_list();

    let page = ui.screen();
    let delete = page.find("delete").expect("the delete is listed");
    let sync = page.find("sync").expect("the sync is listed");
    assert!(delete < sync, "newest first: {page}");
    assert!(page.contains("2 runs"), "{page}");
}

#[test]
fn an_undo_that_cannot_restore_anything_is_reported_without_asking() {
    let mut ui = Harness::new(world());
    ui.sync_once();
    // Both projects are gone, so no saved folder has a place to go back to.
    std::fs::remove_dir_all(ui.world.root().join("one")).unwrap();
    std::fs::remove_dir_all(ui.world.root().join("two")).unwrap();
    ui.open("History").wait_list();

    ui.code(Code::Enter).wait_history_done();

    let page = ui.screen();
    assert!(page.contains("Nothing can be undone"), "{page}");
    assert!(page.contains("failed"), "{page}");
    assert!(
        !page.contains("Undo the sync"),
        "no question to ask: {page}"
    );
}

/// A report that carries the id of a job the page no longer waits for.
fn stale(job: Job) -> Event {
    Event::Job { id: 9999, job }
}

#[test]
fn a_report_of_a_job_the_page_no_longer_waits_for_is_ignored() {
    let mut ui = Harness::new(world());
    ui.sync_once();
    ui.open("History");
    assert_eq!(phase(&ui), "loading");
    ui.app.handle(stale(Job::Runs(Box::new(Ok(Vec::new())))));
    assert_eq!(phase(&ui), "loading", "a list from another read");

    ui.wait_list();
    ui.app
        .handle(stale(Job::UndoPlanned(Box::new(Err("old".to_string())))));
    assert_eq!(phase(&ui), "list", "a plan nobody asked for");

    ui.code(Code::Enter);
    assert_eq!(phase(&ui), "planning");
    ui.app
        .handle(stale(Job::UndoPlanned(Box::new(Err("old".to_string())))));
    ui.app
        .handle(stale(Job::Undone(Box::new(Err("old".to_string())))));
    ui.app.handle(stale(Job::Failed("old".to_string())));
    assert_eq!(
        phase(&ui),
        "planning",
        "the page still waits for its own plan"
    );

    ui.wait_history_confirm();
    assert_eq!(ui.world.read(CODING), "coding v2");
}
