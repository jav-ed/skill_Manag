#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod common;

use common::{bare, skillmirror};
use predicates::str::contains;
use skillmirror_testkit::World;

#[test]
fn no_arguments_prints_help() {
    let world = World::standard();
    bare(&world)
        .assert()
        .failure()
        .stderr(contains("Usage:"))
        .stderr(contains("sync"));
}

#[test]
fn a_missing_vault_is_a_hard_error_with_a_hint() {
    let world = World::standard();
    bare(&world)
        .args(["sync", "--dry-run"])
        .assert()
        .code(3)
        .stderr(contains("no vault configured"))
        .stderr(contains("hint:"));
}

#[test]
fn legacy_environment_variables_are_refused_and_name_their_replacement() {
    let world = World::standard();
    skillmirror(&world)
        .env("SKILL_MANAG_VAULT", "/somewhere")
        .args(["sync", "--dry-run"])
        .assert()
        .code(3)
        .stderr(contains("SKILL_MANAG_VAULT -> SKILLMIRROR_VAULT"));
}

#[test]
fn environment_variables_supply_vault_and_root() {
    let world = World::standard();
    bare(&world)
        .env("SKILLMIRROR_VAULT", world.vault())
        .env("SKILLMIRROR_ROOT", world.root())
        .args(["sync", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("3 to update"));
}

#[test]
fn a_vault_that_is_not_a_git_repository_is_a_hard_error() {
    let world = World::new();
    world.vault_file("coding/SKILL.md", "x").vault_config(&[]);
    skillmirror(&world)
        .args(["sync", "--dry-run"])
        .assert()
        .code(3)
        .stderr(contains("not a git repository"))
        .stderr(contains("git init"));
}

#[test]
fn a_malformed_vault_config_is_a_hard_error_naming_the_file() {
    let world = World::standard();
    world
        .vault_file("config.yaml", "rot: /typo\n")
        .commit_vault();
    bare(&world)
        .arg("--vault")
        .arg(world.vault())
        .arg("--root")
        .arg(world.root())
        .args(["sync", "--dry-run"])
        .assert()
        .code(3)
        .stderr(contains("config.yaml"));
}

#[test]
fn a_missing_root_is_a_hard_error() {
    let world = World::standard();
    bare(&world)
        .arg("--vault")
        .arg(world.vault())
        .arg("--root")
        .arg("/does/not/exist")
        .args(["sync", "--dry-run"])
        .assert()
        .code(3)
        .stderr(contains("does not exist"));
}

#[test]
fn migrate_carries_the_old_pointer_over_and_unblocks_the_tool() {
    let world = World::standard();
    let old = world.path().join("home/.config/skill_Manag");
    std::fs::create_dir_all(&old).unwrap();
    std::fs::write(old.join("vault"), format!("{}\n", world.vault().display())).unwrap();

    bare(&world)
        .args(["sync", "--dry-run"])
        .assert()
        .code(3)
        .stderr(contains("migrate"));
    bare(&world)
        .arg("migrate")
        .assert()
        .success()
        .stdout(contains("Copied the vault pointer"));
    bare(&world)
        .args(["sync", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("3 to update"));
    bare(&world)
        .arg("migrate")
        .assert()
        .success()
        .stdout(contains("Already migrated"));
    bare(&world)
        .args(["migrate", "--retire"])
        .assert()
        .success()
        .stdout(contains("Removed the old pointer"));
    assert!(!old.exists());
}

#[test]
fn completions_print_a_script() {
    let world = World::new();
    bare(&world)
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(contains("skillmirror"));
}
