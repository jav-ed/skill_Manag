//! `info SKILL`: one skill from the vault side and from every project's side.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod common;

use common::{json_of, skillmirror};
use predicates::str::contains;
use skillmirror_testkit::World;

#[test]
fn the_text_names_group_files_and_the_state_in_each_project() {
    let world = World::standard();

    skillmirror(&world)
        .args(["info", "astro"])
        .assert()
        .success()
        .stdout(contains("astro"))
        .stdout(contains("web"))
        .stdout(contains("2 copied"))
        .stdout(contains("ref.md"))
        .stdout(contains("Installed in 1 of 3 projects"))
        .stdout(contains("outdated (1 file added, 1 file changed)"));
}

#[test]
fn a_mandatory_skill_says_so_and_names_the_projects_that_lack_it() {
    let world = World::standard();

    skillmirror(&world)
        .args(["info", "tmux"])
        .assert()
        .success()
        .stdout(contains("mandatory"))
        .stdout(contains("Installed in 0 of 3 projects"))
        .stdout(contains("mandatory, not installed"));
}

#[test]
fn the_json_document_carries_the_same_facts() {
    let world = World::standard();

    let doc = json_of(skillmirror(&world).args(["info", "coding", "--json"]));

    assert_eq!(doc["name"], "coding");
    assert_eq!(doc["in_vault"], true);
    assert_eq!(doc["mandatory"], true);
    assert_eq!(doc["files"][0], "SKILL.md");
    assert_eq!(doc["total_projects"], 3);
    let projects = doc["projects"].as_array().unwrap();
    assert_eq!(projects.len(), 3, "one, two and three(missing): {doc}");
    let states: Vec<&str> = projects
        .iter()
        .map(|p| p["state"].as_str().unwrap())
        .collect();
    assert_eq!(states.iter().filter(|s| **s == "outdated").count(), 2);
    assert_eq!(
        states.iter().filter(|s| **s == "mandatory_missing").count(),
        1
    );
}

#[test]
fn a_folder_only_projects_have_is_described_as_not_in_the_vault() {
    let world = World::standard();

    skillmirror(&world)
        .args(["info", "local-only"])
        .assert()
        .success()
        .stdout(contains("not in the vault"));
    let doc = json_of(skillmirror(&world).args(["info", "local-only", "--json"]));
    assert_eq!(doc["in_vault"], false);
    assert_eq!(doc["projects"][0]["state"], "not_in_vault");
}

#[test]
fn a_file_git_does_not_track_is_called_out() {
    let world = World::standard();
    world.vault_file("coding/draft.md", "half finished");

    skillmirror(&world)
        .args(["info", "coding"])
        .assert()
        .success()
        .stdout(contains("not copied"))
        .stdout(contains("draft.md"));
}

#[test]
fn an_unknown_skill_is_a_hard_error_with_a_hint() {
    let world = World::standard();

    skillmirror(&world)
        .args(["info", "ghost"])
        .assert()
        .code(3)
        .stderr(contains("ghost"))
        .stderr(contains("hint:"));
}
