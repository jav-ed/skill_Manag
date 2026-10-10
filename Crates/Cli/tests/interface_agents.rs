#![allow(clippy::unwrap_used, clippy::expect_used)]
//! The Agents page of the interactive interface, in a real terminal: pick projects, read the changes,
//! confirm, and see the file written.
mod common;

use common::pty::tui;
use common::skillmirror;
use skillmirror_testkit::World;

/// Two projects with a block written from an old text, one without a file, and a newer text in the vault.
fn world() -> World {
    let world = World::standard();
    world.vault_file("project-files/AGENTS.md", "# Rules\n\n1. old\n");
    for name in ["one", "two"] {
        skillmirror(&world)
            .args(["agents", "add", "--yes", "--project"])
            .arg(world.root().join(name))
            .assert()
            .success();
    }
    world.vault_file("project-files/AGENTS.md", "# Rules\n\n1. new\n");
    world
}

#[test]
fn agents_writes_the_new_text_into_the_ticked_projects_and_keeps_the_rest_of_the_file() {
    let world = world();
    std::fs::write(
        world.root().join("one/AGENTS.md"),
        format!(
            "{}\n## Mine\nkeep this\n",
            world.read("projects/one/AGENTS.md")
        ),
    )
    .unwrap();
    let mut term = tui(&world);
    term.wait_for("skillmirror");
    // Agents is the sixth entry of the menu.
    term.send("jjjjj\r");
    term.wait_for("Select projects to write the AGENTS.md block into");
    // The two outdated projects are ticked, the one without a file is not.
    term.wait_for("2 / 3 selected");
    term.wait_for("the vault text changed");
    term.wait_for("no AGENTS.md yet");

    term.send("\r");
    term.wait_for("Write AGENTS.md in 2 projects?");
    term.wait_for("Rewrites the block in 2 AGENTS.md files.");
    assert!(
        world.read("projects/one/AGENTS.md").contains("1. old"),
        "the question is asked before anything is written"
    );
    term.send("v");
    term.wait_for("AGENTS.md in projects/one");
    term.wait_for("-1. old");
    term.wait_for("+1. new");
    term.send("v");
    term.wait_for("Write AGENTS.md in 2 projects?");

    term.send("y");
    term.wait_for("Agents file results");
    term.wait_for("written to 2 projects");
    let one = world.read("projects/one/AGENTS.md");
    assert!(one.contains("1. new") && !one.contains("1. old"), "{one}");
    assert!(one.ends_with("\n## Mine\nkeep this\n"), "{one}");
    assert!(world.read("projects/two/AGENTS.md").contains("1. new"));
    assert!(
        !world.exists("projects/three/AGENTS.md"),
        "three was not ticked"
    );

    term.send("q");
    term.wait_for("Refresh the skills each project already has");
    term.send("q");
    assert_eq!(term.exit_code(), 0);
}

#[test]
fn agents_says_why_when_the_text_cannot_be_used() {
    let world = World::standard();
    world.vault_file("project-files/AGENTS.md", "   \n");
    let mut term = tui(&world);
    term.wait_for("skillmirror");
    term.send("jjjjj\r");

    term.wait_for("is empty");
    term.send("q");
    term.wait_for("Refresh the skills each project already has");
    term.send("q");
    assert_eq!(term.exit_code(), 0);
}
