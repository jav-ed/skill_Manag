#![allow(clippy::unwrap_used, clippy::expect_used)]
//! The screens of the interactive interface that came after the first four, in a real terminal.
mod common;

use common::pty::tui;
use skillmirror_testkit::World;

#[test]
fn scan_problems_are_counted_and_listed() {
    let world = World::standard();
    world.project_file("one/.agents/.stage-4000000000-1-0/half/SKILL.md", "x");
    let mut term = tui(&world);
    term.wait_for("skillmirror");
    term.send("\r");

    term.wait_for("1 scan problem (i)");
    term.send("i");
    term.wait_for("Scan problems");
    term.wait_for("left over from an interrupted run");
    term.send("x");
    term.wait_gone("Scan problems");

    term.send("q");
    term.wait_for("Refresh the skills each project already has");
    term.send("q");
    assert_eq!(term.exit_code(), 0);
}

#[test]
fn history_undoes_a_sync() {
    let world = World::standard();
    let mut term = tui(&world);
    term.wait_for("skillmirror");
    term.send("\r");
    term.wait_for("2 / 2 selected");
    term.send("\r");
    term.wait_for("Sync 2 skills in 2 projects?");
    term.send("y");
    term.wait_for("Sync results");
    assert_eq!(
        world.read("projects/one/.agents/skills/coding/SKILL.md"),
        "coding v2"
    );
    term.send("q");
    term.wait_for("Refresh the skills each project already has");

    // History is the fifth entry of the menu.
    term.send("jjjj\r");
    term.wait_for("1 run");
    term.wait_for("sync");
    term.send("\r");
    term.wait_for("Undo the sync of");
    term.wait_for("Puts back 3 skill folders in 2 projects.");
    assert_eq!(
        world.read("projects/one/.agents/skills/coding/SKILL.md"),
        "coding v2",
        "the question is asked before anything is changed"
    );
    term.send("y");
    term.wait_for("Undo results");
    term.wait_for("restored coding in");
    assert_eq!(
        world.read("projects/one/.agents/skills/coding/SKILL.md"),
        "coding v1"
    );

    term.send("q");
    term.wait_for("Undo a sync, push or delete");
    term.send("q");
    assert_eq!(term.exit_code(), 0);
}
