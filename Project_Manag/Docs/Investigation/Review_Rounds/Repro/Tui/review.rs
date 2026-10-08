//! Review round 2, series U (interface). They print what they observe and assert nothing.
//!
//! Findings: M1 (u8, u2b), M2 (u1), M5 (u5), L5 (u2), L6 (u3), L7 (u6). `review_draw.rs` holds the draw fuzz (u4).
//! See `Project_Manag/Docs/Investigation/Review_Rounds/round_2_Full.md`.
#![allow(clippy::print_stdout)]

use std::time::Duration;

use super::{Harness, world};
use crate::app::Screen;
use crate::hit::Target;
use crate::input::Code;
use crate::screens::{Phase, Step};

pub(super) fn phase(app: &crate::app::App) -> String {
    match &app.screen {
        Screen::Menu(_) => "menu".to_string(),
        Screen::Setup(_) => "setup".to_string(),
        Screen::Work(w) => format!(
            "{:?}/{}",
            w.mode,
            match &w.phase {
                Phase::Loading => "loading".to_string(),
                Phase::Failed(_) => "failed".to_string(),
                Phase::Select => "select".to_string(),
                Phase::Confirm(_) => "confirm".to_string(),
                Phase::Running { done, total, .. } => format!("running {done}/{total}"),
                Phase::Done(r) => format!("done(results of {:?})", r.kind),
            }
        ),
    }
}

#[test]
fn u1_a_running_job_reports_into_whatever_screen_is_open() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    ui.code(Code::Enter); // sync starts at once: no confirmation
    println!("U1 after Enter on the Sync page: {}", phase(&ui.app));
    ui.click(Target::HeaderBack); // the arrow in the header is clickable while the job runs
    println!("U1 after clicking the header arrow: {}", phase(&ui.app));
    ui.open("Delete");
    println!(
        "U1 Delete page opened from the stale session: {}",
        phase(&ui.app)
    );
    // now the events of the first job arrive
    while !matches!(&ui.app.screen, Screen::Work(w) if matches!(w.phase, Phase::Done(_))) {
        let event = ui
            .rx
            .recv_timeout(Duration::from_secs(5))
            .expect("job events");
        ui.app.handle(event);
    }
    println!("U1 after the first job reported: {}", phase(&ui.app));
    println!(
        "U1 page text:\n{}",
        ui.screen().lines().take(4).collect::<Vec<_>>().join("\n")
    );
}

#[test]
fn u2_enter_after_a_filter_acts_on_rows_that_are_not_visible() {
    let mut ui = Harness::new(world());
    ui.open("Delete").wait_select();
    ui.press('a'); // select every row
    ui.press('/').type_text("astro").code(Code::Enter);
    let page = ui.screen();
    let selected = page
        .lines()
        .find(|l| l.contains("selected"))
        .unwrap_or("")
        .trim()
        .to_string();
    let visible = page.lines().filter(|l| l.contains("[✗]")).count();
    println!("U2 header: {selected:?}; rows shown with a mark: {visible}");
    ui.code(Code::Enter);
    let dialog = ui.screen();
    println!(
        "U2 dialog title: {:?}",
        dialog
            .lines()
            .find(|l| l.contains("Delete ") && l.contains('?'))
            .unwrap_or("")
            .trim()
    );
    println!("U2 phase: {}", phase(&ui.app));
}

#[test]
fn u2b_sync_and_push_have_no_confirmation_and_no_file_list() {
    let mut ui = Harness::new(world());
    ui.open("Push").wait_select();
    println!(
        "U2b push page selected: {}",
        ui.screen()
            .lines()
            .find(|l| l.contains("selected"))
            .unwrap_or("")
            .trim()
    );
    ui.code(Code::Enter);
    println!("U2b one Enter later: {}", phase(&ui.app));
}

#[test]
fn u3_ctrl_c_during_a_job_ends_the_program() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    ui.code(Code::Enter);
    ui.ctrl('c');
    println!(
        "U3 running: {}, should_quit: {}",
        phase(&ui.app),
        ui.app.should_quit()
    );
}

pub(super) fn walk_wizard(ui: &mut Harness, steps: usize) {
    ui.press('h');
    let Screen::Setup(setup) = &ui.app.screen else {
        panic!("wizard")
    };
    let index = setup
        .picker
        .names
        .iter()
        .position(|n| n == "vault")
        .unwrap();
    for _ in 0..index {
        ui.press('j');
    }
    ui.press('l').code(Code::Enter); // vault chosen -> root step
    if steps >= 2 {
        ui.code(Code::Enter); // root chosen -> mandatory
    }
    if steps >= 3 {
        ui.code(Code::Enter); // -> save
    }
    if steps >= 4 {
        ui.press('y');
    }
}

#[test]
fn u5_setup_writes_the_pointer_before_it_knows_the_config_can_be_saved() {
    let w = world();
    // a valid config that the text rewrite cannot edit (comment at column 0 inside the list)
    w.vault_file(
        "config.yaml",
        &format!(
            "root: {}\nmandatory:\n  - coding\n# keep this\n  - tmux\n",
            w.root().display()
        ),
    );
    let mut ui = Harness::unconfigured(w);
    walk_wizard(&mut ui, 4);
    let Screen::Setup(setup) = &ui.app.screen else {
        panic!("wizard")
    };
    let step = match &setup.step {
        Step::Saved(Ok(())) => "saved ok".to_string(),
        Step::Saved(Err(m)) => format!("saved with error: {}", m.lines().next().unwrap_or("")),
        _ => "not saved".to_string(),
    };
    println!("U5 wizard result: {step}");
    println!(
        "U5 pointer file exists: {}",
        ui.world.exists("home/config/skillmirror/vault")
    );
    if ui.world.exists("home/config/skillmirror/vault") {
        println!(
            "U5 pointer now points at: {:?}",
            ui.world.read("home/config/skillmirror/vault").trim()
        );
    }
}

#[test]
fn u6_a_stale_load_from_before_a_setup_change_is_adopted() {
    // open a page (scan starts with the old settings), leave, change settings, open again before the first scan reports
    let mut ui = Harness::new(world());
    ui.open("Sync"); // spawn_load #1 (old settings), loading = true
    ui.code(Code::Esc); // Loading page goes back
    println!("U6 after Esc on the loading page: {}", phase(&ui.app));
    ui.open("List");
    println!(
        "U6 second open while the first scan is in flight: {}",
        phase(&ui.app)
    );
    ui.wait_select();
    println!(
        "U6 first scan's result populated the second page: {}",
        phase(&ui.app)
    );
}

#[test]
fn u8_the_job_plans_again_after_the_user_pressed_enter() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    println!(
        "U8 page when loaded: {}",
        ui.screen()
            .lines()
            .find(|l| l.contains("to update"))
            .unwrap_or("")
            .trim()
    );
    // while the page is open somebody adds a file to a project's copy
    std::fs::write(
        ui.world
            .path()
            .join("projects/one/.agents/skills/coding/my_notes.md"),
        "mine",
    )
    .unwrap();
    ui.code(Code::Enter).wait_done();
    println!(
        "U8 my_notes.md survived the sync: {}",
        ui.world
            .exists("projects/one/.agents/skills/coding/my_notes.md")
    );
    let page = ui.screen();
    println!("U8 results page mentions it: {}", page.contains("my_notes"));
}
