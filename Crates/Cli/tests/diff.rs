#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod common;

use common::{json_of, skillmirror};
use predicates::prelude::*;
use predicates::str::contains;
use skillmirror_testkit::World;

#[test]
fn diff_shows_the_lines_a_sync_would_bring_in_and_take_away() {
    let world = World::standard();

    skillmirror(&world)
        .arg("diff")
        .assert()
        .code(1)
        .stdout(contains("--- a/SKILL.md"))
        .stdout(contains("+++ b/SKILL.md"))
        .stdout(contains("-coding v1"))
        .stdout(contains("+coding v2"))
        .stdout(contains("+++ b/ref.md"))
        .stdout(contains("3 skill folders differ, 4 files would change"))
        .stdout(contains("\u{1b}[").not());
}

#[test]
fn a_skill_name_and_a_project_narrow_the_diff() {
    let world = World::standard();

    skillmirror(&world)
        .args(["diff", "astro"])
        .assert()
        .code(1)
        .stdout(contains("+astro v2"))
        .stdout(contains("coding v2").not());
    skillmirror(&world)
        .args(["diff", "coding", "--project"])
        .arg(world.root().join("two"))
        .assert()
        .code(1)
        .stdout(contains("two"))
        .stdout(contains("1 skill folder differs, 1 file would change"));
}

#[test]
fn stat_lists_the_files_with_their_line_counts() {
    let world = World::standard();

    skillmirror(&world)
        .args(["diff", "astro", "--stat"])
        .assert()
        .code(1)
        .stdout(contains("SKILL.md"))
        .stdout(contains("modified"))
        .stdout(contains("ref.md"))
        .stdout(contains("added"))
        .stdout(contains("2 files changed, +2 -1"))
        .stdout(contains("+astro v2").not());
}

#[test]
fn an_unknown_skill_is_a_hard_error_and_not_a_clean_diff() {
    let world = World::standard();

    skillmirror(&world)
        .args(["diff", "ghost"])
        .assert()
        .code(3)
        .stderr(contains("ghost"))
        .stderr(contains("hint:"));
}

#[test]
fn when_everything_is_in_sync_the_diff_is_empty_and_the_exit_is_zero() {
    let world = World::standard();
    skillmirror(&world)
        .args(["sync", "--yes"])
        .assert()
        .success();

    skillmirror(&world)
        .arg("diff")
        .assert()
        .success()
        .stdout(contains("No differences"));
}

#[test]
fn the_json_document_carries_the_unified_diff_per_file() {
    let world = World::standard();

    let doc = json_of(skillmirror(&world).args(["diff", "astro", "--json"]));

    let skills = doc["skills"].as_array().unwrap();
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0]["skill"], "astro");
    let files = skills[0]["files"].as_array().unwrap();
    let skill_md = files.iter().find(|f| f["path"] == "SKILL.md").unwrap();
    assert_eq!(skill_md["change"], "modified");
    assert_eq!(skill_md["lines_added"], 1);
    assert_eq!(skill_md["lines_removed"], 1);
    assert!(skill_md["diff"].as_str().unwrap().contains("+astro v2"));
    let added = files.iter().find(|f| f["path"] == "ref.md").unwrap();
    assert_eq!(added["change"], "added");
    assert!(doc["failed"].as_array().unwrap().is_empty(), "{doc}");
}

#[test]
fn diff_writes_nothing() {
    let world = World::standard();

    skillmirror(&world).arg("diff").assert().code(1);

    assert_eq!(
        world.read("projects/one/.agents/skills/coding/SKILL.md"),
        "coding v1"
    );
    assert!(!world.exists("home/.local"), "no state directory was made");
}
