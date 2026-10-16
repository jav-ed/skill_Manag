//! `changes` and `promote`: see what a project changed, put it into the vault, and spread it.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod common;

use common::{json_of, skillmirror};
use predicates::str::contains;
use skillmirror_testkit::World;

fn vault_status(world: &World) -> String {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(world.vault())
        .args(["status", "--porcelain"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// The standard world, whose project `one` has an older coding and astro and a skill the vault lacks.
fn in_project(world: &World, rel: &str) -> assert_cmd::Command {
    let mut cmd = skillmirror(world);
    cmd.current_dir(world.root().join(rel));
    cmd
}

#[test]
fn changes_names_the_project_it_found_and_what_differs_and_exits_1() {
    let world = World::standard();

    in_project(&world, "one")
        .arg("changes")
        .assert()
        .code(1)
        .stdout(contains("projects/one"))
        .stdout(contains("this folder"))
        .stdout(contains("coding"))
        .stdout(contains("edited here"))
        .stdout(contains("local-only"))
        .stdout(contains("not in the vault"));
    // Standing inside the project, and then from the folder above all projects.
    let inside = json_of(in_project(&world, "one/.agents/skills").args(["changes", "--json"]));
    assert_eq!(inside["found"], "above");
    let states: Vec<(&str, &str)> = inside["skills"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| (s["name"].as_str().unwrap(), s["state"].as_str().unwrap()))
        .collect();
    assert_eq!(
        states,
        [
            ("astro", "edited"),
            ("coding", "edited"),
            ("local-only", "new")
        ]
    );
    in_project(&world, "two/.agents")
        .arg("changes")
        .assert()
        .code(1);
}

#[test]
fn promote_puts_the_projects_copy_into_the_vault_and_undo_takes_it_back() {
    let world = World::standard();

    in_project(&world, "one")
        .args(["promote", "coding", "--yes"])
        .assert()
        .success()
        .stdout(contains("Replaced"))
        .stdout(contains("Staged in git, not committed"))
        .stdout(contains("Backup: run "));

    assert_eq!(world.read("vault/coding/SKILL.md"), "coding v1");
    assert_eq!(vault_status(&world), "M  coding/SKILL.md\n");
    // The vault has what the project has now, so a sync would not take it back.
    let after = json_of(in_project(&world, "one").args(["changes", "--json"]));
    let coding = after["skills"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["name"] == "coding")
        .unwrap();
    assert_eq!(coding["state"], "same");
    skillmirror(&world)
        .args(["undo", "--yes"])
        .assert()
        .success();
    assert_eq!(world.read("vault/coding/SKILL.md"), "coding v2");
    assert_eq!(vault_status(&world), "");
}

#[test]
fn promote_with_sync_updates_the_other_projects_that_have_the_skill() {
    let world = World::standard();
    world.project_file("one/.agents/skills/coding/SKILL.md", "coding v3");

    in_project(&world, "one")
        .args(["promote", "coding", "--sync", "--yes"])
        .assert()
        .success();

    assert_eq!(world.read("vault/coding/SKILL.md"), "coding v3");
    assert_eq!(
        world.read("projects/two/.agents/skills/coding/SKILL.md"),
        "coding v3"
    );
    // Only what was promoted is spread: project one's astro is still its own old one.
    assert_eq!(
        world.read("projects/one/.agents/skills/astro/SKILL.md"),
        "astro v1"
    );
}

#[test]
fn a_skill_that_cannot_be_promoted_is_a_hard_error_and_a_dry_run_writes_nothing() {
    let world = World::standard();

    in_project(&world, "one")
        .args(["promote", "local-only", "--yes"])
        .assert()
        .code(3)
        .stderr(contains("adopt"));
    in_project(&world, "one")
        .args(["promote", "coding", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("Would replace"));
    in_project(&world, "one")
        .args(["promote", "coding"])
        .assert()
        .code(2)
        .stderr(contains("--yes"));
    in_project(&world, "one").arg("promote").assert().code(2);
    assert_eq!(world.read("vault/coding/SKILL.md"), "coding v2");
    assert_eq!(vault_status(&world), "");
}
