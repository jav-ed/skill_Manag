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
