//! `config show|path|root` and `mandatory list|add|remove`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod common;

use common::{bare, json_of, skillmirror};
use predicates::prelude::*;
use predicates::str::contains;
use skillmirror_testkit::World;

#[test]
fn show_names_every_setting_and_where_the_vault_and_root_came_from() {
    let world = World::standard();

    skillmirror(&world)
        .args(["config", "show"])
        .assert()
        .success()
        .stdout(contains("vault"))
        .stdout(contains("(flag)"))
        .stdout(contains("mandatory     coding, tmux"))
        .stdout(contains("targets       none"))
        .stdout(contains("config.yaml"));
}

#[test]
fn show_as_json_carries_the_same() {
    let world = World::standard();

    let doc = json_of(skillmirror(&world).args(["config", "show", "--json"]));

    assert_eq!(doc["vault"]["source"], "flag");
    assert!(doc["vault"]["path"].as_str().unwrap().ends_with("/vault"));
    assert_eq!(doc["mandatory"][0], "coding");
    assert!(
        doc["config_file"]
            .as_str()
            .unwrap()
            .ends_with("config.yaml")
    );
}

#[test]
fn show_with_nothing_set_says_so_instead_of_failing() {
    let world = World::new();

    bare(&world)
        .args(["config", "show"])
        .assert()
        .success()
        .stdout(contains("not set"));
}

#[test]
fn path_prints_the_config_file_alone() {
    let world = World::standard();

    let out = skillmirror(&world)
        .args(["config", "path"])
        .output()
        .unwrap();

    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        world.vault().join("config.yaml").display().to_string()
    );
}

#[test]
fn root_changes_only_the_root_line_and_keeps_the_rest_of_the_file() {
    let world = World::standard();
    let before = world.read("vault/config.yaml");
    let other = world.path().join("elsewhere");
    std::fs::create_dir(&other).unwrap();

    bare(&world)
        .arg("--vault")
        .arg(world.vault())
        .args(["config", "root"])
        .arg(&other)
        .assert()
        .success()
        .stdout(contains("Set root"));

    let after = world.read("vault/config.yaml");
    assert!(
        after.contains(&format!("root: \"{}\"", other.display())),
        "{after}"
    );
    assert!(after.contains("coding"), "mandatory is untouched: {after}");
    assert_ne!(before, after);
}

#[test]
fn root_refuses_a_folder_that_is_not_there_and_a_dry_run_changes_nothing() {
    let world = World::standard();
    let before = world.read("vault/config.yaml");

    skillmirror(&world)
        .args(["config", "root"])
        .arg(world.path().join("nope"))
        .assert()
        .code(2)
        .stderr(contains("nope"));
    skillmirror(&world)
        .args(["config", "root", "--dry-run"])
        .arg(world.path())
        .assert()
        .success()
        .stdout(contains("Would set"));

    assert_eq!(world.read("vault/config.yaml"), before);
}

#[test]
fn root_warns_when_a_flag_still_decides() {
    let world = World::standard();

    skillmirror(&world)
        .args(["config", "root"])
        .arg(world.path())
        .assert()
        .success()
        .stderr(contains("--root still decides"));
}

#[test]
fn mandatory_add_and_remove_edit_the_list() {
    let world = World::standard();

    skillmirror(&world)
        .args(["mandatory", "add", "astro"])
        .assert()
        .success()
        .stdout(contains("astro"))
        .stdout(contains("Saved in"));
    skillmirror(&world)
        .args(["mandatory", "list"])
        .assert()
        .success()
        .stdout(contains("coding\ntmux\nastro\n"));
    skillmirror(&world)
        .args(["mandatory", "remove", "tmux", "astro"])
        .assert()
        .success();
    skillmirror(&world)
        .args(["mandatory", "list", "--json"])
        .assert()
        .success()
        .stdout(contains("\"coding\""));
    let doc = json_of(skillmirror(&world).args(["mandatory", "list", "--json"]));
    assert_eq!(doc["mandatory"].as_array().unwrap().len(), 1);
}

#[test]
fn adding_twice_changes_nothing_the_second_time() {
    let world = World::standard();
    let before = world.read("vault/config.yaml");

    skillmirror(&world)
        .args(["mandatory", "add", "coding"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Saved in").not());

    assert_eq!(
        world.read("vault/config.yaml"),
        before,
        "an unchanged list does not rewrite the file"
    );
}

#[test]
fn an_unknown_skill_and_an_unlisted_name_are_hard_errors_and_a_dry_run_writes_nothing() {
    let world = World::standard();
    let before = world.read("vault/config.yaml");

    skillmirror(&world)
        .args(["mandatory", "add", "ghost"])
        .assert()
        .code(3)
        .stderr(contains("ghost"))
        .stderr(contains("hint:"));
    skillmirror(&world)
        .args(["mandatory", "remove", "astro"])
        .assert()
        .code(3)
        .stderr(contains("not in the mandatory list"));
    skillmirror(&world)
        .args(["mandatory", "add", "astro", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("dry run"));

    assert_eq!(world.read("vault/config.yaml"), before);
}
