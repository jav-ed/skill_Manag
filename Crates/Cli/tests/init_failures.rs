#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
//! `init` must leave nothing behind when nothing could be installed, so the same command can be run again.
mod common;

use common::skillmirror;
use predicates::str::contains;
use skillmirror_testkit::World;

/// A vault whose only mandatory skill is not tracked by git, so every plan fails.
fn world_with_untracked_mandatory() -> World {
    let world = World::standard();
    world.vault_config(&["fresh"]).commit_vault();
    world.vault_file("fresh/SKILL.md", "not committed");
    world
}

#[test]
fn init_with_nothing_installable_creates_no_directory_and_can_be_repeated() {
    let world = world_with_untracked_mandatory();
    let dir = world.root().join("new1");

    for _ in 0..2 {
        skillmirror(&world)
            .args(["init", "--no-agents-md", "--yes"])
            .arg(&dir)
            .assert()
            .code(4)
            .stdout(contains("1 failed"));
        assert!(!dir.exists(), "the failed init created {}", dir.display());
    }
}

#[test]
fn init_with_git_and_nothing_installable_creates_no_repository() {
    let world = world_with_untracked_mandatory();
    let dir = world.root().join("new2");

    skillmirror(&world)
        .args(["init", "--no-agents-md", "--git", "--yes"])
        .arg(&dir)
        .assert()
        .code(4);

    assert!(!dir.exists(), "{} was left behind", dir.display());
}

#[test]
fn init_does_not_even_ask_when_there_is_nothing_to_write() {
    let world = world_with_untracked_mandatory();
    let dir = world.root().join("new3");

    // Without --yes and without a terminal a write would be refused (exit 2); a plan of failures only
    // is shown like a dry run instead.
    skillmirror(&world)
        .args(["init", "--no-agents-md"])
        .arg(&dir)
        .assert()
        .code(4)
        .stdout(contains("1 failed"));

    assert!(!dir.exists());
}

#[test]
fn init_removes_what_it_made_when_every_write_failed() {
    let world = World::standard();
    // The backup store path is a plain file, so every target fails after the plan was fine.
    let state = world.path().join("home/.local/state/skillmirror");
    std::fs::create_dir_all(&state).unwrap();
    std::fs::write(state.join("backups"), "in the way").unwrap();
    let dir = world.root().join("new4");

    skillmirror(&world)
        .args(["init", "--no-agents-md", "--git", "--yes"])
        .arg(&dir)
        .assert()
        .code(4);

    assert!(!dir.exists(), "{} was left behind", dir.display());
    std::fs::remove_file(state.join("backups")).unwrap();
    skillmirror(&world)
        .args(["init", "--no-agents-md", "--git", "--yes"])
        .arg(&dir)
        .assert()
        .success();
    assert!(dir.join(".git").exists());
    assert!(dir.join(".agents/skills/coding/SKILL.md").exists());
}

#[test]
fn init_keeps_an_existing_empty_directory_but_removes_the_repository_it_made() {
    let world = world_with_untracked_mandatory();
    let dir = world.root().join("new5");
    std::fs::create_dir_all(&dir).unwrap();

    skillmirror(&world)
        .args(["init", "--no-agents-md", "--git", "--yes"])
        .arg(&dir)
        .assert()
        .code(4);

    assert!(dir.is_dir(), "the user's empty directory is theirs");
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
}

#[test]
fn a_partial_failure_keeps_the_project_with_what_was_installed() {
    let world = World::standard();
    world.vault_config(&["coding", "fresh"]).commit_vault();
    world.vault_file("fresh/SKILL.md", "not committed");
    let dir = world.root().join("new6");

    skillmirror(&world)
        .args(["init", "--no-agents-md", "--yes"])
        .arg(&dir)
        .assert()
        .code(4);

    assert!(dir.join(".agents/skills/coding/SKILL.md").exists());
    assert!(!dir.join(".agents/skills/fresh").exists());
}

#[test]
fn init_git_ignores_a_git_dir_from_the_environment() {
    let world = World::standard();
    let dir = world.root().join("new7");
    let elsewhere = world.path().join("other.git");

    skillmirror(&world)
        .env("GIT_DIR", &elsewhere)
        .args(["init", "--no-agents-md", "--git", "--yes"])
        .arg(&dir)
        .assert()
        .success();

    assert!(dir.join(".git").exists(), "no repository in the project");
    assert!(!elsewhere.exists(), "git followed GIT_DIR");
}
