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

    // History is the seventh entry of the menu.
    term.send("jjjjjj\r");
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

#[test]
fn add_installs_a_skill_into_a_project() {
    let world = World::standard();
    let mut term = tui(&world);
    term.wait_for("skillmirror");
    // Add is the fifth entry of the menu.
    term.send("jjjj\r");
    term.wait_for("The project that gets the skills.");
    // The picker starts in the root: one, plain, three, two. Walk into `two` and choose it.
    term.send("jjjl");
    term.wait_for("projects/two");
    term.send("\r");
    term.wait_for("Select skills to add to");
    term.wait_for("0 / 3 selected");
    // astro, coding, tmux: tick tmux.
    term.send("jj ");
    term.wait_for("1 / 3 selected");
    term.send("\r");
    term.wait_for("Add 1 skill to");
    assert!(!world.exists("projects/two/.agents/skills/tmux"));
    term.send("y");
    term.wait_for("Add results");
    assert_eq!(
        world.read("projects/two/.agents/skills/tmux/SKILL.md"),
        "tmux v2"
    );
    term.send("q");
    term.wait_for("Install chosen vault skills into one project");
    term.send("q");
    assert_eq!(term.exit_code(), 0);
}

#[test]
fn init_makes_a_new_project() {
    let world = World::standard();
    let mut term = tui(&world);
    term.wait_for("skillmirror");
    // Init is the sixth entry of the menu.
    term.send("jjjjj\r");
    term.wait_for("The folder the new project goes in.");
    term.send("\r");
    term.wait_for("The name of the new project folder:");
    // `q` and `j` are letters here.
    term.send("quick\r");
    term.wait_for("Select skills for");
    term.wait_for("2 / 3 selected");
    term.send("g\r");
    term.wait_for("Create ");
    term.wait_for("It becomes a git repository.");
    assert!(!world.exists("projects/quick"), "nothing before yes");
    term.send("y");
    term.wait_for("Init results");
    assert!(world.exists("projects/quick/.git"));
    assert_eq!(
        world.read("projects/quick/.agents/skills/coding/SKILL.md"),
        "coding v2"
    );
    term.send("q");
    term.wait_for("Make a new project folder with your skills in it");
    term.send("q");
    assert_eq!(term.exit_code(), 0);
}

#[test]
fn the_question_shows_the_changes_before_anything_is_written() {
    let world = World::standard();
    let mut term = tui(&world);
    term.wait_for("skillmirror");
    term.send("\r");
    term.wait_for("2 / 2 selected");
    term.send("\r");
    term.wait_for("Sync 2 skills in 2 projects?");
    term.wait_for("v view changes");

    term.send("v");
    term.wait_for("is taken from the project");
    term.wait_for("-coding v1");
    term.wait_for("+coding v2");
    assert_eq!(
        world.read("projects/one/.agents/skills/coding/SKILL.md"),
        "coding v1",
        "looking writes nothing"
    );

    // `q` leaves the changes and returns to the same question, which `y` then answers.
    term.send("q");
    term.wait_for("Sync 2 skills in 2 projects?");
    term.send("y");
    term.wait_for("Sync results");
    assert_eq!(
        world.read("projects/one/.agents/skills/coding/SKILL.md"),
        "coding v2"
    );
    term.send("q");
    term.wait_for("Refresh the skills each project already has");
    term.send("q");
    assert_eq!(term.exit_code(), 0);
}
