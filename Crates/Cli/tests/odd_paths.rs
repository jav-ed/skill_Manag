#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
//! A project folder whose name is not valid UTF-8 is still a project: every command must work on it.
mod common;

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use common::{json_of, skillmirror};
use predicates::str::contains;
use skillmirror_testkit::World;

/// The standard world plus a project called `caf<0xE9>-app` (latin-1, not UTF-8) with an old `coding`.
fn world_with_odd_project() -> (World, PathBuf) {
    let world = World::standard();
    let project = world.root().join(OsStr::from_bytes(b"caf\xe9-app"));
    let skill = project.join(".agents/skills/coding");
    std::fs::create_dir_all(&skill).unwrap();
    std::fs::write(skill.join("SKILL.md"), "coding v1").unwrap();
    (world, skill)
}

#[test]
fn sync_undo_and_history_work_on_a_project_with_a_latin_1_name() {
    let (world, skill) = world_with_odd_project();

    skillmirror(&world)
        .args(["sync", "--yes"])
        .assert()
        .success()
        .stdout(contains("Backup: run "));
    assert_eq!(
        std::fs::read_to_string(skill.join("SKILL.md")).unwrap(),
        "coding v2"
    );

    skillmirror(&world).arg("history").assert().success();
    skillmirror(&world)
        .args(["undo", "--yes"])
        .assert()
        .success();
    assert_eq!(
        std::fs::read_to_string(skill.join("SKILL.md")).unwrap(),
        "coding v1"
    );
}

#[test]
fn the_json_documents_survive_such_a_project() {
    let (world, skill) = world_with_odd_project();

    let sync = json_of(skillmirror(&world).args(["sync", "--yes", "--json"]));
    assert!(sync["entries"].as_array().unwrap().len() >= 3);
    assert_eq!(
        std::fs::read_to_string(skill.join("SKILL.md")).unwrap(),
        "coding v2"
    );

    let listed = json_of(skillmirror(&world).args(["list", "--json"]));
    assert!(
        !listed["installed"].as_array().unwrap().is_empty(),
        "{listed}"
    );
    let undone = json_of(skillmirror(&world).args(["undo", "--yes", "--json"]));
    assert!(
        !undone["entries"].as_array().unwrap().is_empty(),
        "{undone}"
    );
    let deleted = json_of(skillmirror(&world).args(["delete", "coding", "--yes", "--json"]));
    assert!(
        !deleted["deleted"].as_array().unwrap().is_empty(),
        "{deleted}"
    );
}
