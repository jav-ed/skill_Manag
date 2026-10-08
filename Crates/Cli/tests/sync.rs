#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod common;

use common::{json_of, skillmirror};
use predicates::str::contains;
use skillmirror_testkit::World;

#[test]
fn dry_run_lists_changes_and_writes_nothing() {
    let world = World::standard();
    skillmirror(&world)
        .args(["sync", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("coding"))
        .stdout(contains("update (1 changed)"))
        .stdout(contains("3 skills in 2 projects: 3 to update (dry run)"));
    assert_eq!(
        world.read("projects/one/.agents/skills/coding/SKILL.md"),
        "coding v1"
    );
    assert!(!world.exists("projects/one/.agents/skills/astro/ref.md"));
}

#[test]
fn sync_yes_updates_only_installed_skills_and_a_second_run_has_nothing_to_do() {
    let world = World::standard();
    skillmirror(&world)
        .args(["sync", "--yes"])
        .assert()
        .success()
        .stdout(contains("3 updated"));
    assert_eq!(
        world.read("projects/one/.agents/skills/coding/SKILL.md"),
        "coding v2"
    );
    assert_eq!(
        world.read("projects/one/.agents/skills/astro/SKILL.md"),
        "astro v2"
    );
    assert_eq!(
        world.read("projects/one/.agents/skills/astro/ref.md"),
        "ref"
    );
    assert_eq!(
        world.read("projects/one/.agents/skills/local-only/SKILL.md"),
        "mine"
    );
    assert!(
        !world.exists("projects/two/.agents/skills/tmux"),
        "sync must never add a skill"
    );
    assert!(!world.exists("projects/one/.agents/.stage-leftover"));
    skillmirror(&world)
        .args(["sync", "--yes"])
        .assert()
        .success()
        .stdout(contains("3 up to date"));
}

#[test]
fn check_exits_one_on_drift_and_zero_when_in_sync() {
    let world = World::standard();
    skillmirror(&world)
        .args(["sync", "--check"])
        .assert()
        .code(1);
    skillmirror(&world)
        .args(["sync", "--yes"])
        .assert()
        .success();
    skillmirror(&world)
        .args(["sync", "--check"])
        .assert()
        .code(0);
}

#[test]
fn writing_without_yes_and_without_a_terminal_is_refused() {
    let world = World::standard();
    skillmirror(&world)
        .arg("sync")
        .write_stdin("")
        .assert()
        .code(2)
        .stderr(contains("refusing to write without confirmation"))
        .stderr(contains("--yes"));
    assert_eq!(
        world.read("projects/one/.agents/skills/coding/SKILL.md"),
        "coding v1"
    );
}

#[test]
fn json_output_is_one_document_with_per_file_changes() {
    let world = World::standard();
    let doc = json_of(skillmirror(&world).args(["sync", "--dry-run", "--json"]));
    assert_eq!(doc["command"], "sync");
    assert_eq!(doc["mode"], "dry-run");
    assert_eq!(doc["summary"]["update"], 3);
    let astro = doc["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["skill"] == "astro")
        .unwrap();
    assert_eq!(astro["status"], "update");
    assert_eq!(astro["files"], 2);
    let kinds: Vec<_> = astro["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| (c["path"].as_str().unwrap(), c["kind"].as_str().unwrap()))
        .collect();
    assert_eq!(kinds, [("SKILL.md", "modified"), ("ref.md", "added")]);
}

#[test]
fn nested_vault_groups_install_flat() {
    let world = World::standard();
    skillmirror(&world)
        .args(["sync", "--yes"])
        .assert()
        .success();
    assert!(world.exists("projects/one/.agents/skills/astro/SKILL.md"));
    assert!(!world.exists("projects/one/.agents/skills/web"));
}

#[test]
fn an_empty_vault_or_root_says_so_and_succeeds() {
    let world = World::new();
    world.vault_config(&[]).commit_vault();
    skillmirror(&world)
        .args(["sync", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("No skills found in vault."));
    let world = World::new();
    world
        .vault_file("coding/SKILL.md", "x")
        .vault_config(&[])
        .commit_vault();
    skillmirror(&world)
        .args(["sync", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("No matching skills found in any project."));
}

#[test]
fn a_skill_without_tracked_files_fails_that_skill_and_leaves_the_project_alone() {
    let world = World::standard();
    world.vault_file("fresh/SKILL.md", "untracked");
    world.project_file("one/.agents/skills/fresh/SKILL.md", "precious");
    skillmirror(&world)
        .args(["sync", "--yes"])
        .assert()
        .code(4)
        .stdout(contains("no git-tracked files"));
    assert_eq!(
        world.read("projects/one/.agents/skills/fresh/SKILL.md"),
        "precious"
    );
    assert_eq!(
        world.read("projects/one/.agents/skills/coding/SKILL.md"),
        "coding v2"
    );
}
