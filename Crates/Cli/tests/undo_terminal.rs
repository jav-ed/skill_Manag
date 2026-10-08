#![allow(clippy::unwrap_used, clippy::expect_used)]
//! `undo` asks a person in a real terminal, and only a yes changes anything.
mod common;

use common::pty::Terminal;
use common::skillmirror;
use skillmirror_testkit::World;

const ONE_CODING: &str = "projects/one/.agents/skills/coding/SKILL.md";

fn synced_world() -> World {
    let world = World::standard();
    skillmirror(&world)
        .args(["sync", "--yes"])
        .assert()
        .success();
    world
}

fn undo_in_terminal(world: &World) -> Terminal {
    let vault = world.vault();
    let root = world.root();
    Terminal::spawn(
        world,
        &[
            "--vault",
            vault.to_str().unwrap(),
            "--root",
            root.to_str().unwrap(),
            "undo",
        ],
        24,
        140,
    )
}

#[test]
fn undo_shows_the_plan_and_a_no_changes_nothing() {
    let world = synced_world();
    let mut term = undo_in_terminal(&world);

    term.wait_for("would restore in");
    term.wait_for("Undo the sync of");
    term.wait_for("[y/N]");
    term.send("n\n");

    term.wait_for("Cancelled, nothing was changed.");
    assert_eq!(term.exit_code(), 0);
    assert_eq!(world.read(ONE_CODING), "coding v2");
}

#[test]
fn undo_after_a_yes_restores_the_old_copies() {
    let world = synced_world();
    let mut term = undo_in_terminal(&world);

    term.wait_for("[y/N]");
    term.send("y\n");

    term.wait_for("3 folders restored");
    assert_eq!(term.exit_code(), 0);
    assert_eq!(world.read(ONE_CODING), "coding v1");
}
