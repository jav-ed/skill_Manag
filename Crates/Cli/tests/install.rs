#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod common;

use common::{bare, json_of, skillmirror};
use predicates::str::contains;
use skillmirror_testkit::World;

const PROFILES: &str = "profiles:\n  websites:\n    description: Websites\n    groups: [web]\n    skills: [tmux]\n  site:\n    extends: [websites]\n    exclude: [tmux]\n";

fn world() -> World {
    let world = World::standard();
    world
        .vault_file("web/seo/SKILL.md", "seo v1")
        .vault_config_with(&["coding"], PROFILES)
        .commit_vault();
    world
}

#[test]
fn skills_shows_the_vault_as_a_tree_with_mandatory_marks() {
    let world = world();
    skillmirror(&world)
        .arg("skills")
        .assert()
        .success()
        .stdout(contains("web/"))
        .stdout(contains("astro"))
        .stdout(contains("coding  (mandatory)"))
        .stdout(contains("4 skills in 1 group"));
}

#[test]
fn skills_can_be_limited_to_a_group_and_rejects_unknown_groups() {
    let world = world();
    let json = json_of(skillmirror(&world).args(["skills", "--group", "web", "--json"]));
    let names: Vec<_> = json["skills"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["name"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(names, ["astro", "seo"]);
    skillmirror(&world)
        .args(["skills", "--group", "nope"])
        .assert()
        .code(3)
        .stderr(contains("nope"));
}

#[test]
fn add_installs_a_group_into_the_given_project() {
    let world = world();
    skillmirror(&world)
        .args(["add", "--group", "web", "--yes", "--project"])
        .arg(world.root().join("one"))
        .assert()
        .success();
    assert_eq!(
        world.read("projects/one/.agents/skills/seo/SKILL.md"),
        "seo v1"
    );
    assert_eq!(
        world.read("projects/one/.agents/skills/astro/ref.md"),
        "ref"
    );
    assert!(!world.exists("projects/two/.agents/skills/seo"));
}

#[test]
fn add_writes_nothing_on_a_dry_run() {
    let world = world();
    skillmirror(&world)
        .args(["add", "seo", "--dry-run", "--project"])
        .arg(world.root().join("one"))
        .assert()
        .success()
        .stdout(contains("seo"));
    assert!(!world.exists("projects/one/.agents/skills/seo"));
}

#[test]
fn add_refuses_a_directory_that_does_not_exist() {
    let world = world();
    skillmirror(&world)
        .args(["add", "seo", "--yes", "--project"])
        .arg(world.root().join("missing"))
        .assert()
        .code(3);
}

#[test]
fn add_without_a_selection_is_an_error() {
    let world = world();
    skillmirror(&world)
        .args(["add", "--yes", "--project"])
        .arg(world.root().join("one"))
        .assert()
        .failure();
}

#[test]
fn init_creates_the_project_with_mandatory_skills_and_the_profile() {
    let world = world();
    let dir = world.root().join("fresh");
    skillmirror(&world)
        .args(["init", "--no-agents-md"])
        .arg(&dir)
        .args(["--profile", "site", "--yes", "--git"])
        .assert()
        .success();
    for skill in ["coding", "astro", "seo"] {
        assert!(
            world.exists(&format!("projects/fresh/.agents/skills/{skill}/SKILL.md")),
            "{skill} is installed"
        );
    }
    assert!(
        !world.exists("projects/fresh/.agents/skills/tmux"),
        "the profile excludes tmux"
    );
    assert!(world.exists("projects/fresh/.git"));
}

#[test]
fn init_with_no_mandatory_installs_only_the_selection() {
    let world = world();
    skillmirror(&world)
        .args(["init", "--no-agents-md"])
        .arg(world.root().join("lean"))
        .args(["seo", "--no-mandatory", "--yes"])
        .assert()
        .success();
    assert!(world.exists("projects/lean/.agents/skills/seo/SKILL.md"));
    assert!(!world.exists("projects/lean/.agents/skills/coding"));
}

#[test]
fn init_dry_run_creates_no_directory() {
    let world = world();
    skillmirror(&world)
        .args(["init", "--no-agents-md"])
        .arg(world.root().join("ghost"))
        .args(["--dry-run"])
        .assert()
        .success();
    assert!(!world.exists("projects/ghost"));
}

#[test]
fn init_refuses_a_non_empty_directory_and_creates_nothing() {
    let world = world();
    skillmirror(&world)
        .args(["init", "--no-agents-md"])
        .arg(world.root().join("one"))
        .arg("--yes")
        .assert()
        .code(3);
}

#[test]
fn init_with_an_unknown_profile_creates_no_directory() {
    let world = world();
    skillmirror(&world)
        .args(["init", "--no-agents-md"])
        .arg(world.root().join("never"))
        .args(["--profile", "nope", "--yes"])
        .assert()
        .code(3)
        .stderr(contains("nope"));
    assert!(!world.exists("projects/never"));
}

#[test]
fn init_without_yes_and_without_a_terminal_is_a_usage_error() {
    let world = world();
    skillmirror(&world)
        .args(["init", "--no-agents-md"])
        .arg(world.root().join("unasked"))
        .assert()
        .code(2);
    assert!(!world.exists("projects/unasked"));
}

#[test]
fn init_works_without_a_vault_root_flag() {
    let world = world();
    bare(&world)
        .arg("--vault")
        .arg(world.vault())
        .args(["init", "--no-agents-md"])
        .arg(world.root().join("solo"))
        .args(["--yes"])
        .assert()
        .success();
    assert!(
        world.exists("projects/solo/.agents/skills/tmux/SKILL.md")
            || world.exists("projects/solo/.agents/skills/coding/SKILL.md")
    );
}
