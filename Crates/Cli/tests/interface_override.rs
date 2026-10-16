#![allow(clippy::unwrap_used, clippy::expect_used)]
//! The Override page in a real terminal, started inside a project.
mod common;

use common::pty::tui_in;
use skillmirror_testkit::World;

#[test]
fn override_replaces_the_vault_copy_of_a_skill_changed_in_the_project_it_was_started_in() {
    let world = World::standard();
    let mut term = tui_in(&world, Some(&world.root().join("one")));
    term.wait_for("skillmirror");
    // Override is the second entry of the menu.
    term.send("j\r");
    term.wait_for("the project in this folder");
    term.wait_for("2 skills can be replaced");
    term.wait_for("local-only");
    // The cursor starts on the first edited skill, astro.
    term.send(" \r");
    term.wait_for("Replace 1 skill in the vault?");
    assert_eq!(world.read("vault/web/astro/SKILL.md"), "astro v2");
    term.send("y");
    term.wait_for("astro replaced in the vault");
    assert_eq!(world.read("vault/web/astro/SKILL.md"), "astro v1");

    term.send("q");
    term.wait_for("Put the skills you changed in this project into the vault");
    term.send("q");
    assert_eq!(term.exit_code(), 0);
}
