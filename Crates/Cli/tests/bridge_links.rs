#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
//! What the rest of the tool says and does about bridges: `status`, `doctor`, `add` and `init`.
mod common;

use std::path::Path;

use common::pty::Terminal;
use common::{claude_link as link_of, json_of, skillmirror, world_with_claude_target as world};
use predicates::prelude::*;
use predicates::str::contains;
use skillmirror_testkit::World;

#[test]
fn status_reports_a_missing_bridge_as_drift_until_it_is_made() {
    let world = World::standard();
    skillmirror(&world)
        .args(["sync", "--yes"])
        .assert()
        .success();
    skillmirror(&world)
        .args(["push", "--yes"])
        .assert()
        .success();
    skillmirror(&world).arg("status").assert().success();
    world
        .vault_config_with(&["coding", "tmux"], "targets: [claude]")
        .commit_vault();

    skillmirror(&world)
        .arg("status")
        .assert()
        .code(1)
        .stdout(contains("bridge to .agents/skills not made"))
        .stdout(contains("3 bridges missing"));
    let doc = json_of(skillmirror(&world).args(["status", "--json"]));
    assert_eq!(doc["summary"]["missing_bridges"], 3);
    assert_eq!(doc["projects"][0]["missing_bridges"][0], "claude");

    skillmirror(&world)
        .args(["bridge", "--yes"])
        .assert()
        .success();

    skillmirror(&world)
        .arg("status")
        .assert()
        .success()
        .stdout(contains("bridge").not());
}

#[test]
fn doctor_warns_about_a_missing_bridge() {
    let world = world();

    skillmirror(&world)
        .arg("doctor")
        .assert()
        .stdout(contains("bridge is missing"))
        .stdout(contains("skillmirror bridge"));
}

#[test]
fn add_links_the_project_it_wrote_to_and_leaves_the_others() {
    let world = world();

    skillmirror(&world)
        .args(["add", "tmux", "--yes", "--project"])
        .arg(world.root().join("one"))
        .assert()
        .success()
        .stdout(contains("linked"));

    assert!(link_of(&world, "one").is_some());
    assert!(
        link_of(&world, "two").is_none(),
        "other projects are not touched"
    );
}

#[test]
fn add_on_a_dry_run_links_nothing() {
    let world = world();

    skillmirror(&world)
        .args(["add", "tmux", "--dry-run", "--project"])
        .arg(world.root().join("one"))
        .assert()
        .success()
        .stdout(contains("linked").not());

    assert!(link_of(&world, "one").is_none());
}

#[test]
fn add_that_installs_nothing_makes_no_link() {
    let world = world();
    // Sync brings every installed skill up to the vault; it makes no bridge either.
    skillmirror(&world)
        .args(["sync", "--yes"])
        .assert()
        .success();
    assert!(link_of(&world, "two").is_none());

    skillmirror(&world)
        .args(["add", "coding", "--yes", "--project"])
        .arg(world.root().join("two"))
        .assert()
        .success()
        .stdout(contains("linked").not());

    let found = link_of(&world, "two");
    assert!(
        found.is_none(),
        "{found:?}: nothing was written, so nothing was linked"
    );
}

#[test]
fn add_reports_a_real_folder_in_the_way_without_failing_the_install() {
    let world = world();
    world.project_file("one/.claude/skills/mine/SKILL.md", "mine");

    skillmirror(&world)
        .args(["add", "tmux", "--yes", "--project"])
        .arg(world.root().join("one"))
        .assert()
        .success()
        .stdout(contains("real folder"));

    assert_eq!(
        world.read("projects/one/.claude/skills/mine/SKILL.md"),
        "mine"
    );
    assert_eq!(
        world.read("projects/one/.agents/skills/tmux/SKILL.md"),
        "tmux v2"
    );
}

#[test]
fn add_json_lists_the_bridges_and_sync_json_does_not() {
    let world = world();

    let doc = json_of(
        skillmirror(&world)
            .args(["add", "tmux", "--yes", "--json", "--project"])
            .arg(world.root().join("one")),
    );
    assert_eq!(doc["bridges"][0]["status"], "linked");

    let sync = json_of(skillmirror(&world).args(["sync", "--dry-run", "--json"]));
    assert!(sync.get("bridges").is_none());
}

#[test]
fn init_links_the_new_project() {
    let world = world();
    let dir = world.root().join("fresh");

    skillmirror(&world)
        .args(["init", "--no-agents-md"])
        .arg(&dir)
        .arg("--yes")
        .assert()
        .success();

    assert_eq!(
        link_of(&world, "fresh").as_deref(),
        Some(Path::new("../.agents/skills"))
    );
}

fn bridge_in_terminal(world: &World) -> Terminal {
    let vault = world.vault();
    let root = world.root();
    Terminal::spawn(
        world,
        &[
            "--vault",
            vault.to_str().unwrap(),
            "--root",
            root.to_str().unwrap(),
            "bridge",
        ],
        24,
        140,
    )
}

#[test]
fn bridge_asks_in_a_terminal_and_only_a_yes_links() {
    let world = world();

    let mut term = bridge_in_terminal(&world);
    term.wait_for("Link 3 bridge(s)? [y/N]");
    term.send("n\n");
    term.wait_for("Cancelled, nothing was linked.");
    assert_eq!(term.exit_code(), 0);
    assert!(link_of(&world, "one").is_none());

    let mut term = bridge_in_terminal(&world);
    term.wait_for("[y/N]");
    term.send("y\n");
    term.wait_for("3 bridges linked");
    assert_eq!(term.exit_code(), 0);
    assert!(link_of(&world, "one").is_some());
}

#[test]
fn a_link_in_the_way_is_a_problem_of_the_project_and_not_of_a_skill() {
    let world = world();
    // Everything in sync and linked except `one`, where a real folder is in the way.
    skillmirror(&world)
        .args(["sync", "--yes"])
        .assert()
        .success();
    skillmirror(&world)
        .args(["push", "--yes"])
        .assert()
        .success();
    world.project_file("one/.claude/skills/mine/SKILL.md", "mine");
    skillmirror(&world)
        .args(["bridge", "--yes"])
        .assert()
        .code(4);

    let out = skillmirror(&world)
        .args(["status", "--json"])
        .output()
        .unwrap();

    assert_eq!(out.status.code(), Some(4));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let one = doc["projects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["project"].as_str().unwrap().ends_with("/one"))
        .unwrap();
    assert_eq!(one["status"], "problem");
    assert_eq!(one["failed"].as_array().unwrap().len(), 0);
    assert_eq!(one["project_problems"][0]["skill"], "claude");
    assert!(
        one["project_problems"][0]["message"]
            .as_str()
            .unwrap()
            .contains("real folder")
    );
    assert_eq!(doc["summary"]["failed"], 1);
}
