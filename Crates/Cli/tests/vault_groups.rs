//! A vault is sorted into folders as the owner likes: a folder with `SKILL.md` is a skill, any other folder
//! that holds skills is a group, and groups nest without a limit. Projects never see the groups.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod common;

use common::{json_of, skillmirror};
use predicates::str::contains;
use skillmirror_testkit::World;

const DEEP: &str = "Web_Skills/seo/schema/audits/weekly/checks/seo-audit";

fn sorted_world() -> World {
    let world = World::new();
    world
        .vault_file("Default_Skills/coding/SKILL.md", "coding v2")
        .vault_file("Default_Skills/tmux/SKILL.md", "tmux v2")
        .vault_file("Web_Skills/astro/SKILL.md", "astro v2")
        .vault_file(&format!("{DEEP}/SKILL.md"), "audit v2")
        .project_file("one/.agents/skills/coding/SKILL.md", "coding v1")
        .project_file("one/.agents/skills/seo-audit/SKILL.md", "audit v1")
        .project_file("one/.agents/skills/astro/SKILL.md", "astro v1");
    world.vault_config(&["coding"]).commit_vault();
    world
}

#[test]
fn groups_of_any_depth_install_flat_under_the_name_of_the_skill() {
    let world = sorted_world();

    skillmirror(&world)
        .args(["sync", "--yes"])
        .assert()
        .success()
        .stdout(contains("3 updated"));

    assert_eq!(
        world.read("projects/one/.agents/skills/seo-audit/SKILL.md"),
        "audit v2"
    );
    assert_eq!(
        world.read("projects/one/.agents/skills/coding/SKILL.md"),
        "coding v2"
    );
    assert!(!world.exists("projects/one/.agents/skills/Web_Skills"));
    assert!(!world.exists("projects/one/.agents/skills/Default_Skills"));
}

#[test]
fn a_group_is_named_by_its_path_at_any_level() {
    let world = sorted_world();

    // A level in the middle takes everything below it and nothing beside it.
    skillmirror(&world)
        .args(["sync", "--group", "Web_Skills/seo", "--yes"])
        .assert()
        .success()
        .stdout(contains("1 updated"));

    assert_eq!(
        world.read("projects/one/.agents/skills/seo-audit/SKILL.md"),
        "audit v2"
    );
    assert_eq!(
        world.read("projects/one/.agents/skills/astro/SKILL.md"),
        "astro v1",
        "astro sits in Web_Skills, above the group that was named"
    );
}

#[test]
fn the_listing_gives_every_folder_of_the_group() {
    let world = sorted_world();

    let doc = json_of(skillmirror(&world).args(["skills", "--json"]));

    let deep = doc["skills"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["name"] == "seo-audit")
        .expect("the deep skill is listed");
    assert_eq!(
        deep["group"],
        serde_json::json!(["Web_Skills", "seo", "schema", "audits", "weekly", "checks"])
    );
}

#[test]
fn moving_a_skill_into_a_group_changes_nothing_in_the_projects() {
    let world = World::standard();
    skillmirror(&world)
        .args(["sync", "--yes"])
        .assert()
        .success();
    let vault = world.vault();
    std::fs::create_dir_all(vault.join("Default/basics")).unwrap();
    std::fs::rename(vault.join("coding"), vault.join("Default/basics/coding")).unwrap();
    world.commit_vault();

    skillmirror(&world)
        .args(["sync", "--check"])
        .assert()
        .success();
}

#[test]
fn a_skill_moved_but_not_yet_added_to_git_is_refused_and_nothing_is_removed() {
    let world = World::standard();
    skillmirror(&world)
        .args(["sync", "--yes"])
        .assert()
        .success();
    let vault = world.vault();
    std::fs::create_dir_all(vault.join("Default")).unwrap();
    std::fs::rename(vault.join("coding"), vault.join("Default/coding")).unwrap();

    skillmirror(&world)
        .args(["sync", "--yes"])
        .assert()
        .code(4)
        .stdout(contains("no git-tracked files"))
        .stdout(contains("git add"));

    assert_eq!(
        world.read("projects/one/.agents/skills/coding/SKILL.md"),
        "coding v2",
        "the copy in the project is still there"
    );
}
