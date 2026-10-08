//! Review round 2, series U, test u4: every screen at every small terminal size, to find panics.
//!
//! Result on the reviewed tip: 15,200 draws, 0 panics. See `Project_Manag/Docs/Investigation/Review_Rounds/review_Method.md`.

use std::panic::{AssertUnwindSafe, catch_unwind};

use super::review::{phase, walk_wizard};
use super::{Harness, world};
use crate::app::Screen;
use crate::input::Code;
use crate::results::Kind;
use crate::screens::Phase;

fn states() -> Vec<(&'static str, Harness)> {
    let mut out: Vec<(&'static str, Harness)> = Vec::new();
    out.push(("menu", Harness::new(world())));
    let mut ui = Harness::new(world());
    ui.press('?');
    out.push(("menu+help", ui));
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    out.push(("sync select", ui));
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    ui.press('/').type_text("co");
    out.push(("sync filtering", ui));
    let mut ui = Harness::new(world());
    ui.open("Delete").wait_select();
    ui.press('a');
    out.push(("delete select", ui));
    let mut ui = Harness::new(world());
    ui.open("Delete").wait_select();
    ui.press('a').code(Code::Enter);
    out.push(("delete confirm", ui));
    let mut ui = Harness::new(world());
    ui.open("Delete").wait_select();
    ui.press('a').code(Code::Enter).press('?');
    out.push(("confirm+help", ui));
    let mut ui = Harness::new(world());
    ui.open("List").wait_select();
    out.push(("list select", ui));
    let mut ui = Harness::new(world());
    ui.open("Push").wait_select();
    out.push(("push select", ui));
    let mut ui = Harness::new(world());
    ui.open("List").wait_select();
    ui.press('a').press('s').wait_done();
    out.push(("results", ui));
    let mut ui = Harness::new(world());
    ui.open("List").wait_select();
    ui.press('a').press('s').wait_done();
    ui.press('d');
    out.push(("results details", ui));
    let w = world();
    std::fs::remove_dir_all(w.vault().join(".git")).unwrap();
    let mut ui = Harness::new(w);
    ui.open("Sync")
        .wait_for("failed", |app| phase(app).ends_with("failed"));
    out.push(("failed page", ui));
    let mut ui = Harness::new(world());
    ui.open("Sync");
    out.push(("loading", ui));
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    if let Screen::Work(w) = &mut ui.app.screen {
        w.phase = Phase::Running {
            kind: Kind::Sync,
            done: 1,
            total: 3,
        };
    }
    out.push(("running", ui));
    let mut ui = Harness::unconfigured(world());
    out.push(("setup vault", ui_take(&mut ui)));
    let mut ui = Harness::unconfigured(world());
    walk_wizard(&mut ui, 1);
    out.push(("setup root", ui));
    let mut ui = Harness::unconfigured(world());
    walk_wizard(&mut ui, 2);
    out.push(("setup mandatory", ui));
    let mut ui = Harness::unconfigured(world());
    walk_wizard(&mut ui, 3);
    out.push(("setup save", ui));
    let mut ui = Harness::unconfigured(world());
    walk_wizard(&mut ui, 4);
    out.push(("setup saved", ui));
    out
}

fn ui_take(ui: &mut Harness) -> Harness {
    std::mem::replace(ui, Harness::new(world()))
}

#[test]
fn u4_every_screen_at_every_small_size() {
    let mut panics = Vec::new();
    let mut draws = 0usize;
    for (name, mut ui) in states() {
        for width in 1..=50u16 {
            for height in 1..=16u16 {
                ui.terminal.backend_mut().resize(width, height);
                draws += 1;
                let outcome = catch_unwind(AssertUnwindSafe(|| ui.draw()));
                if outcome.is_err() {
                    panics.push(format!("{name} at {width}x{height}"));
                }
            }
        }
    }
    println!("U4 {draws} draws, {} panics", panics.len());
    for p in panics.iter().take(25) {
        println!("U4 PANIC {p}");
    }
}
