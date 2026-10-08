#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod common;

use common::{json_of, skillmirror};
use predicates::prelude::*;
use predicates::str::contains;
use skillmirror_testkit::World;

const ONE_CODING: &str = "projects/one/.agents/skills/coding/SKILL.md";
const TWO_CODING: &str = "projects/two/.agents/skills/coding/SKILL.md";
const ONE_ASTRO: &str = "projects/one/.agents/skills/astro/SKILL.md";

fn synced() -> World {
    let world = World::standard();
    skillmirror(&world)
        .args(["sync", "--yes"])
        .assert()
        .success()
        .stdout(contains("Backup: run "));
    world
}

fn runs(world: &World) -> Vec<serde_json::Value> {
    let doc = json_of(skillmirror(world).args(["history", "--json"]));
    doc["runs"].as_array().unwrap().clone()
}

#[test]
fn undo_after_sync_brings_back_every_old_byte() {
    let world = synced();
    assert_eq!(world.read(ONE_CODING), "coding v2");

    skillmirror(&world)
        .args(["undo", "--yes"])
        .assert()
        .success()
        .stdout(contains("3 folders restored"))
        .stdout(contains("undo again to redo this undo"))
        .stdout(contains("would restore").not());

    assert_eq!(world.read(ONE_CODING), "coding v1");
    assert_eq!(world.read(TWO_CODING), "coding v1");
    assert_eq!(world.read(ONE_ASTRO), "astro v1");
    assert!(!world.exists("projects/one/.agents/skills/astro/ref.md"));
    assert_eq!(
        world.read("projects/one/.agents/skills/local-only/SKILL.md"),
        "mine"
    );
}

#[test]
fn undoing_twice_redoes_the_sync() {
    let world = synced();
    skillmirror(&world)
        .args(["undo", "--yes"])
        .assert()
        .success();

    skillmirror(&world)
        .args(["undo", "--yes"])
        .assert()
        .success();

    assert_eq!(world.read(ONE_CODING), "coding v2");
    assert_eq!(world.read(ONE_ASTRO), "astro v2");
    assert!(world.exists("projects/one/.agents/skills/astro/ref.md"));
}

#[test]
fn a_dry_run_undo_reports_and_changes_nothing() {
    let world = synced();

    skillmirror(&world)
        .args(["undo", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("would restore in"))
        .stdout(contains("(dry run)"));

    assert_eq!(world.read(ONE_CODING), "coding v2");
    assert_eq!(runs(&world).len(), 1, "the run is still there");
}

#[test]
fn undo_without_yes_and_without_a_terminal_is_refused() {
    let world = synced();

    skillmirror(&world)
        .arg("undo")
        .assert()
        .code(2)
        .stderr(contains("refusing to undo without confirmation"))
        .stderr(contains("--yes"));

    assert_eq!(world.read(ONE_CODING), "coding v2");
}

#[test]
fn a_filter_undoes_one_project_or_one_skill() {
    let world = synced();
    let sync_run = runs(&world)[0]["id"].as_str().unwrap().to_string();

    skillmirror(&world)
        .args(["undo", "--yes", "--project"])
        .arg(world.root().join("two"))
        .assert()
        .success();

    assert_eq!(world.read(TWO_CODING), "coding v1");
    assert_eq!(world.read(ONE_CODING), "coding v2", "other projects stay");

    // The newest run is now the undo itself, so the sync run is named.
    skillmirror(&world)
        .args(["undo", sync_run.as_str(), "--yes", "--skill", "astro"])
        .assert()
        .success();

    assert_eq!(world.read(ONE_ASTRO), "astro v1");
    assert_eq!(world.read(ONE_CODING), "coding v2", "other skills stay");
}

#[test]
fn a_filter_that_matches_nothing_says_so_and_changes_nothing() {
    let world = synced();

    skillmirror(&world)
        .args(["undo", "--yes", "--skill", "ghost"])
        .assert()
        .success()
        .stdout(contains("Nothing in run"));

    assert_eq!(world.read(ONE_CODING), "coding v2");
}

#[test]
fn delete_keeps_the_folder_and_undo_restores_it() {
    let world = World::standard();
    skillmirror(&world)
        .args(["delete", "coding", "--yes"])
        .assert()
        .success()
        .stdout(contains("Backup: run "));
    assert!(!world.exists("projects/one/.agents/skills/coding"));

    skillmirror(&world)
        .args(["undo", "--yes"])
        .assert()
        .success();

    assert_eq!(world.read(ONE_CODING), "coding v1");
    assert_eq!(world.read(TWO_CODING), "coding v1");
}

#[test]
fn undo_after_push_removes_what_push_installed() {
    let world = World::standard();
    skillmirror(&world)
        .args(["push", "--yes"])
        .assert()
        .success();
    assert!(world.exists("projects/three/.agents/skills/tmux/SKILL.md"));

    skillmirror(&world)
        .args(["undo", "--yes"])
        .assert()
        .success()
        .stdout(contains("removed"));

    assert!(!world.exists("projects/three/.agents/skills/tmux"));
    assert!(
        world.exists("projects/three/.agents/skills"),
        "skills/ stays"
    );
    assert_eq!(world.read(ONE_CODING), "coding v1");
}

#[test]
fn history_lists_runs_newest_first_as_text_and_json() {
    let world = synced();
    skillmirror(&world)
        .args(["delete", "astro", "--yes"])
        .assert()
        .success();

    let listed = runs(&world);

    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0]["command"], "delete");
    assert_eq!(listed[1]["command"], "sync");
    assert_eq!(listed[1]["skills"], 3);
    assert_eq!(listed[1]["projects"], 2);
    assert!(listed[0]["date"].as_str().unwrap().ends_with(" UTC"));
    skillmirror(&world)
        .arg("history")
        .assert()
        .success()
        .stdout(contains("sync"))
        .stdout(contains("delete"));
}

#[test]
fn json_documents_name_the_backup_run() {
    let world = World::standard();
    let sync = json_of(skillmirror(&world).args(["sync", "--yes", "--json"]));
    let id = sync["backup"].as_str().unwrap().to_string();
    assert_eq!(runs(&world)[0]["id"], id.as_str());

    let again = json_of(skillmirror(&world).args(["sync", "--yes", "--json"]));
    assert!(again["backup"].is_null(), "nothing changed, nothing kept");

    let undone = json_of(skillmirror(&world).args(["undo", id.as_str(), "--yes", "--json"]));
    assert_eq!(undone["run"], id.as_str());
    assert_eq!(undone["entries"].as_array().unwrap().len(), 3);
    assert_eq!(undone["entries"][0]["status"], "restored");
    assert!(
        undone["saved_as"].as_str().is_some(),
        "the undo is a run too"
    );
}

#[test]
fn a_run_that_changed_nothing_leaves_no_backup() {
    let world = World::standard();
    skillmirror(&world)
        .args(["sync", "--yes"])
        .assert()
        .success();
    let before = runs(&world).len();

    skillmirror(&world)
        .args(["sync", "--yes"])
        .assert()
        .success()
        .stdout(contains("Backup:").not());

    assert_eq!(runs(&world).len(), before);
}

#[test]
fn nothing_to_undo_and_unknown_runs_are_clear_errors() {
    let world = World::standard();

    skillmirror(&world)
        .args(["undo", "--yes"])
        .assert()
        .code(3)
        .stderr(contains("no backup to restore"))
        .stderr(contains("hint:"));
    skillmirror(&world)
        .args(["undo", "../nope", "--yes"])
        .assert()
        .code(2)
        .stderr(contains("no backup run named"))
        .stderr(contains("skillmirror history"));
    skillmirror(&world)
        .arg("history")
        .assert()
        .success()
        .stdout(contains("No backups yet"));
}

#[test]
fn a_dry_run_sync_or_delete_keeps_nothing() {
    let world = World::standard();
    skillmirror(&world)
        .args(["sync", "--dry-run"])
        .assert()
        .success();
    skillmirror(&world)
        .args(["delete", "coding", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("Backup:").not());
    assert!(runs(&world).is_empty());
}
