#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
//! The `bridge` command.
mod common;

use std::path::Path;

use common::{claude_link as link_of, json_of, skillmirror, world_with_claude_target as world};
use predicates::str::contains;
use skillmirror_testkit::World;

#[test]
fn a_dry_run_names_every_link_it_would_make_and_writes_nothing() {
    let world = world();

    skillmirror(&world)
        .args(["bridge", "--dry-run"])
        .assert()
        .code(1)
        .stdout(contains("would link"))
        .stdout(contains("3 bridges to link"))
        .stdout(contains("(dry run)"));

    for project in ["one", "two", "three"] {
        assert!(
            !world.exists(&format!("projects/{project}/.claude")),
            "{project}"
        );
    }
}

#[test]
fn bridge_links_every_project_with_skills_and_is_quiet_the_second_time() {
    let world = world();

    skillmirror(&world)
        .args(["bridge", "--yes"])
        .assert()
        .success()
        .stdout(contains("3 bridges linked"));

    for project in ["one", "two", "three"] {
        assert_eq!(
            link_of(&world, project).as_deref(),
            Some(Path::new("../.agents/skills")),
            "{project}"
        );
    }
    assert!(
        !world.exists("projects/plain/.claude"),
        "a folder without skills gets no link"
    );
    assert_eq!(
        world.read("projects/one/.claude/skills/coding/SKILL.md"),
        "coding v1",
        "the link leads to the skills"
    );
    skillmirror(&world)
        .args(["bridge", "--yes"])
        .assert()
        .success()
        .stdout(contains("0 bridges linked, 3 already in place"));
}

#[test]
fn a_real_claude_skills_folder_is_kept_and_the_rest_are_still_linked() {
    let world = world();
    world.project_file("one/.claude/skills/mine/SKILL.md", "mine");

    skillmirror(&world)
        .args(["bridge", "--yes"])
        .assert()
        .code(4)
        .stdout(contains("real folder"))
        .stdout(contains("hint: move what it holds"))
        .stdout(contains("2 bridges linked"))
        .stdout(contains("1 need attention"));

    assert_eq!(
        world.read("projects/one/.claude/skills/mine/SKILL.md"),
        "mine"
    );
    assert!(
        link_of(&world, "one").is_none(),
        "the folder was not replaced"
    );
    assert!(link_of(&world, "two").is_some());
    assert!(link_of(&world, "three").is_some());
}

#[test]
fn a_claude_folder_that_is_a_link_is_never_written_through() {
    let world = world();
    let elsewhere = world.path().join("elsewhere");
    std::fs::create_dir_all(&elsewhere).unwrap();
    std::os::unix::fs::symlink(&elsewhere, world.root().join("one/.claude")).unwrap();

    skillmirror(&world)
        .args(["bridge", "--yes"])
        .assert()
        .code(4)
        .stdout(contains("is a link, so nothing is written through it"));

    assert_eq!(
        std::fs::read_dir(&elsewhere).unwrap().count(),
        0,
        "nothing appeared behind the link"
    );
}

#[test]
fn a_link_that_points_elsewhere_is_reported_and_left() {
    let world = world();
    std::fs::create_dir_all(world.root().join("one/.claude")).unwrap();
    std::os::unix::fs::symlink("../other", world.root().join("one/.claude/skills")).unwrap();

    skillmirror(&world)
        .args(["bridge", "--yes"])
        .assert()
        .code(4)
        .stdout(contains("is a link to ../other"));

    assert_eq!(
        link_of(&world, "one").as_deref(),
        Some(Path::new("../other"))
    );
}

#[test]
fn without_targets_there_is_nothing_to_do() {
    let world = World::standard();

    skillmirror(&world)
        .args(["bridge", "--yes"])
        .assert()
        .success()
        .stdout(contains("No targets in the vault config"));
    let doc = json_of(skillmirror(&world).args(["bridge", "--json"]));
    assert_eq!(doc["bridges"].as_array().unwrap().len(), 0);
    assert!(!world.exists("projects/one/.claude"));
}

#[test]
fn an_unknown_target_is_a_config_error() {
    let world = World::standard();
    world
        .vault_config_with(&["coding"], "targets: [cursor]")
        .commit_vault();

    skillmirror(&world)
        .args(["bridge", "--dry-run"])
        .assert()
        .code(3)
        .stderr(contains("cursor"));
}

#[test]
fn linking_without_a_terminal_needs_yes() {
    let world = world();

    skillmirror(&world)
        .arg("bridge")
        .assert()
        .code(2)
        .stderr(contains("--yes"));
    assert!(link_of(&world, "one").is_none());
}

#[test]
fn the_json_document_has_one_row_per_project_and_a_status() {
    let world = world();
    world.project_file("two/.claude/skills/mine/SKILL.md", "mine");

    let cmd_out = skillmirror(&world)
        .args(["bridge", "--yes", "--json"])
        .output()
        .unwrap();
    assert_eq!(cmd_out.status.code(), Some(4));
    let doc: serde_json::Value = serde_json::from_slice(&cmd_out.stdout).unwrap();

    assert_eq!(doc["dry_run"], false);
    let rows = doc["bridges"].as_array().unwrap();
    assert_eq!(rows.len(), 3);
    let status_of = |project: &str| {
        rows.iter()
            .find(|r| r["project"].as_str().unwrap().ends_with(project))
            .unwrap()["status"]
            .clone()
    };
    assert_eq!(status_of("/one"), "linked");
    assert!(
        status_of("/two")["blocked"]["message"]
            .as_str()
            .unwrap()
            .contains("real folder")
    );
    assert_eq!(rows[0]["target"], "claude");
}
