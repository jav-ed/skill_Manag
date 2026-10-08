//! `sync`, `push` and `status` limited to named skills, a group, a profile or one project.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod common;

use common::{json_of, skillmirror, world_with_claude_target};
use predicates::prelude::*;
use predicates::str::contains;
use skillmirror_testkit::World;

#[test]
fn sync_of_one_skill_updates_that_skill_everywhere_and_nothing_else() {
    let world = World::standard();

    skillmirror(&world)
        .args(["sync", "coding", "--yes"])
        .assert()
        .success()
        .stdout(contains("2 updated"));

    assert_eq!(
        world.read("projects/one/.agents/skills/coding/SKILL.md"),
        "coding v2"
    );
    assert_eq!(
        world.read("projects/two/.agents/skills/coding/SKILL.md"),
        "coding v2"
    );
    assert_eq!(
        world.read("projects/one/.agents/skills/astro/SKILL.md"),
        "astro v1",
        "astro was not named"
    );
}

#[test]
fn sync_of_one_project_leaves_the_other_projects_alone() {
    let world = World::standard();

    skillmirror(&world)
        .args(["sync", "--project"])
        .arg(world.root().join("two"))
        .arg("--yes")
        .assert()
        .success()
        .stdout(contains("1 updated"));

    assert_eq!(
        world.read("projects/two/.agents/skills/coding/SKILL.md"),
        "coding v2"
    );
    assert_eq!(
        world.read("projects/one/.agents/skills/coding/SKILL.md"),
        "coding v1"
    );
}

#[test]
fn sync_by_group_takes_the_skills_below_the_folder() {
    let world = World::standard();

    skillmirror(&world)
        .args(["sync", "--group", "web", "--yes"])
        .assert()
        .success()
        .stdout(contains("1 updated"));

    assert_eq!(
        world.read("projects/one/.agents/skills/astro/ref.md"),
        "ref"
    );
    assert_eq!(
        world.read("projects/one/.agents/skills/coding/SKILL.md"),
        "coding v1"
    );
}

#[test]
fn a_named_skill_is_still_never_added_to_a_project_that_lacks_it() {
    let world = World::standard();

    skillmirror(&world)
        .args(["sync", "tmux", "--yes"])
        .assert()
        .success()
        .stdout(contains("No matching skills"));

    assert!(!world.exists("projects/one/.agents/skills/tmux"));
}

#[test]
fn an_unknown_skill_or_project_is_an_error_with_a_hint() {
    let world = World::standard();

    skillmirror(&world)
        .args(["sync", "ghost", "--dry-run"])
        .assert()
        .code(3)
        .stderr(contains("ghost"))
        .stderr(contains("hint:"));
    skillmirror(&world)
        .args(["sync", "--project"])
        .arg(world.root().join("plain"))
        .arg("--dry-run")
        .assert()
        .code(3)
        .stderr(contains("not a project with a skills folder"))
        .stderr(contains("hint:"));
}

#[test]
fn check_with_a_scope_only_looks_inside_it() {
    let world = World::standard();
    skillmirror(&world)
        .args(["sync", "coding", "--yes"])
        .assert()
        .success();

    // coding is current everywhere now; astro still differs, but it is outside the scope.
    skillmirror(&world)
        .args(["sync", "coding", "--check"])
        .assert()
        .success();
    skillmirror(&world)
        .args(["sync", "astro", "--check"])
        .assert()
        .code(1);
}

#[test]
fn push_of_one_project_installs_the_mandatory_skills_only_there() {
    let world = World::standard();

    skillmirror(&world)
        .args(["push", "--project"])
        .arg(world.root().join("three"))
        .arg("--yes")
        .assert()
        .success();

    assert!(world.exists("projects/three/.agents/skills/coding/SKILL.md"));
    assert!(world.exists("projects/three/.agents/skills/tmux/SKILL.md"));
    assert!(!world.exists("projects/two/.agents/skills/tmux"));
}

#[test]
fn push_refuses_a_skill_that_is_not_mandatory() {
    let world = World::standard();

    skillmirror(&world)
        .args(["push", "astro", "--dry-run"])
        .assert()
        .code(3)
        .stderr(contains("not a mandatory skill"))
        .stderr(contains("hint:"));
    assert!(!world.exists("projects/three/.agents/skills/astro"));
}

#[test]
fn status_of_one_skill_lists_that_skill_and_its_summary_counts_it_alone() {
    let world = World::standard();

    let doc = json_of(skillmirror(&world).args(["status", "astro", "--json"]));

    let rows = doc["projects"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "only project one has astro: {doc}");
    assert_eq!(rows[0]["outdated"][0]["skill"], "astro");
    assert_eq!(doc["summary"]["outdated_skills"], 1);
    assert_eq!(doc["summary"]["missing_mandatory"], 0);
}

#[test]
fn status_of_one_project_names_only_that_project() {
    let world = World::standard();

    skillmirror(&world)
        .args(["status", "--project"])
        .arg(world.root().join("two"))
        .assert()
        .code(1)
        .stdout(contains("two"))
        .stdout(contains("one").not());
}

#[test]
fn a_missing_link_belongs_to_the_project_and_is_not_reported_for_a_named_skill() {
    let world = world_with_claude_target();

    let whole = json_of(skillmirror(&world).args(["status", "--json"]));
    assert!(
        whole["summary"]["missing_bridges"].as_u64().unwrap() > 0,
        "without a scope the missing links are counted: {whole}"
    );

    let scoped = json_of(skillmirror(&world).args(["status", "coding", "--json"]));
    assert_eq!(scoped["summary"]["missing_bridges"], 0, "{scoped}");
}
