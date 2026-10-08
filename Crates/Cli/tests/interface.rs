#![allow(clippy::unwrap_used, clippy::expect_used)]
mod common;

use std::thread;
use std::time::Duration;

use common::pty::Terminal;
use common::skillmirror;
use predicates::str::contains;
use skillmirror_testkit::World;

fn start(world: &World) -> Terminal {
    let vault = world.vault();
    let root = world.root();
    Terminal::spawn(
        world,
        &[
            "--vault",
            vault.to_str().unwrap(),
            "--root",
            root.to_str().unwrap(),
        ],
        30,
        100,
    )
}

#[test]
fn a_session_syncs_through_a_real_terminal_and_quits_cleanly() {
    let world = World::standard();
    let mut term = start(&world);
    term.wait_for("skillmirror");
    term.send("\r");
    term.wait_for("Select skills to sync");
    term.wait_for("2 / 2 selected");
    term.send("\r");
    // The page says what would be written and waits for a yes before it writes anything.
    term.wait_for("Sync 2 skills in 2 projects?");
    assert_eq!(
        world.read("projects/one/.agents/skills/coding/SKILL.md"),
        "coding v1"
    );
    term.send("y");
    term.wait_for("Sync results");
    term.wait_for("synced to 2 projects");
    assert_eq!(
        world.read("projects/one/.agents/skills/coding/SKILL.md"),
        "coding v2"
    );
    term.send("q");
    term.wait_for("Refresh the skills each project already has");
    term.send("q");
    assert_eq!(term.exit_code(), 0);
}

#[test]
fn the_mouse_opens_an_entry_and_an_idle_interface_uses_no_cpu() {
    let world = World::standard();
    let mut term = start(&world);
    term.wait_for("skillmirror");
    // Menu rows: header 0, gap 1, tagline 2, gap 3, then one row per entry from 4 (Sync, List, ...): eight
    // entries with two lines each would leave no room for the long text under them.
    term.click(8, 5);
    term.wait_for("Skills — 4 installed");
    term.send("q");
    term.wait_for("Refresh the skills each project already has");
    let before = term.cpu_ticks();
    thread::sleep(Duration::from_secs(2));
    let idle = term.cpu_ticks() - before;
    assert!(idle <= 1, "an idle interface used {idle} CPU ticks in 2 s");
    term.send("\x03");
    assert_eq!(term.exit_code(), 0);
}

#[test]
fn the_tui_command_without_a_terminal_is_a_usage_error() {
    let world = World::standard();
    skillmirror(&world)
        .arg("tui")
        .assert()
        .code(2)
        .stderr(contains("needs a terminal"))
        .stderr(contains("hint:"));
}
