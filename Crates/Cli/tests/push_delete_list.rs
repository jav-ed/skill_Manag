#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod common;

use common::{json_of, skillmirror};
use predicates::str::contains;
use skillmirror_testkit::World;

#[test]
fn push_installs_mandatory_skills_into_every_project_with_a_skills_directory() {
    let world = World::standard();
    skillmirror(&world)
        .args(["push", "--yes"])
        .assert()
        .success()
        .stdout(contains("added"));
    for project in ["one", "two", "three"] {
        assert_eq!(
            world.read(&format!("projects/{project}/.agents/skills/tmux/SKILL.md")),
            "tmux v2"
        );
        assert_eq!(
            world.read(&format!(
                "projects/{project}/.agents/skills/coding/SKILL.md"
            )),
            "coding v2"
        );
    }
    assert!(!world.exists("projects/plain/.agents"));
    assert!(
        !world.exists("projects/one/.agents/skills/astro/ref.md"),
        "push installs mandatory skills only"
    );
}

#[test]
fn push_with_an_unknown_mandatory_name_is_a_hard_error_and_writes_nothing() {
    let world = World::standard();
    world.vault_config(&["coding", "ghost"]).commit_vault();
    skillmirror(&world)
        .args(["push", "--yes"])
        .assert()
        .code(3)
        .stderr(contains("ghost"))
        .stderr(contains("hint:"));
    assert!(!world.exists("projects/three/.agents/skills/coding"));
}

#[test]
fn delete_rejects_names_that_could_escape_the_skills_directory() {
    let world = World::standard();
    for bad in ["..", "a/b", ".agents"] {
        skillmirror(&world)
            .args(["delete", bad, "--yes"])
            .assert()
            .code(2)
            .stderr(contains("not a valid skill name"));
    }
    skillmirror(&world)
        .arg("delete")
        .arg("..")
        .arg("--project")
        .arg(world.root().join("one"))
        .arg("--yes")
        .assert()
        .code(2);
    assert!(world.exists("projects/one/.agents/skills/coding"));
}

#[test]
fn delete_by_name_removes_only_that_skill_everywhere() {
    let world = World::standard();
    skillmirror(&world)
        .args(["delete", "coding", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("would delete from"))
        .stdout(contains("2 folders would be removed (dry run)"));
    assert!(world.exists("projects/one/.agents/skills/coding"));
    skillmirror(&world)
        .args(["delete", "coding", "--yes"])
        .assert()
        .success()
        .stdout(contains("2 folders removed"));
    assert!(!world.exists("projects/one/.agents/skills/coding"));
    assert!(!world.exists("projects/two/.agents/skills/coding"));
    assert!(world.exists("projects/one/.agents/skills/astro"));
}

#[test]
fn delete_in_one_project_and_missing_names() {
    let world = World::standard();
    skillmirror(&world)
        .args(["delete", "ghost", "--yes"])
        .assert()
        .success()
        .stdout(contains("ghost not found in any project."));
    skillmirror(&world)
        .arg("delete")
        .arg("coding")
        .arg("--project")
        .arg(world.root().join("one"))
        .arg("--yes")
        .assert()
        .success();
    assert!(!world.exists("projects/one/.agents/skills/coding"));
    assert!(world.exists("projects/two/.agents/skills/coding"));
    skillmirror(&world)
        .arg("delete")
        .arg("nope")
        .arg("--project")
        .arg(world.root().join("one"))
        .arg("--yes")
        .assert()
        .code(3)
        .stderr(contains("is not an installed skill folder"));
}

#[test]
fn delete_without_yes_and_without_a_terminal_is_refused() {
    let world = World::standard();
    skillmirror(&world)
        .args(["delete", "coding"])
        .write_stdin("")
        .assert()
        .code(2)
        .stderr(contains("refusing to delete"));
    assert!(world.exists("projects/one/.agents/skills/coding"));
}

#[test]
fn list_shows_every_installed_skill_and_marks_those_missing_from_the_vault() {
    let world = World::standard();
    skillmirror(&world)
        .arg("list")
        .assert()
        .success()
        .stdout(contains("local-only"))
        .stdout(contains("(not in vault)"))
        .stdout(contains("4 skills installed in 2 projects"));
    let doc = json_of(skillmirror(&world).args(["list", "--json"]));
    assert_eq!(doc["installed"].as_array().unwrap().len(), 4);
}
