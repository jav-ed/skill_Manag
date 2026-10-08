#![allow(clippy::unwrap_used, clippy::expect_used)]
//! `--json` promises one JSON document on stdout, so it cannot also ask a question in a terminal.
mod common;

use common::pty::Terminal;
use skillmirror_testkit::World;

fn run_in_terminal(world: &World, args: &[&str]) -> Terminal {
    let vault = world.vault();
    let root = world.root();
    let mut all = vec![
        "--vault",
        vault.to_str().unwrap(),
        "--root",
        root.to_str().unwrap(),
    ];
    all.extend_from_slice(args);
    Terminal::spawn(world, &all, 24, 120)
}

#[test]
fn a_json_write_in_a_terminal_without_yes_is_refused_before_anything_is_printed_or_written() {
    let world = World::standard();
    let mut term = run_in_terminal(&world, &["sync", "--json"]);

    term.wait_for("--json cannot ask for confirmation");
    assert_eq!(term.exit_code(), 2);

    let screen = term.text();
    assert!(!screen.contains("[y/N]"), "{screen}");
    assert!(
        !screen.contains("coding"),
        "no rows before the error: {screen}"
    );
    assert_eq!(
        world.read("projects/one/.agents/skills/coding/SKILL.md"),
        "coding v1"
    );
}
