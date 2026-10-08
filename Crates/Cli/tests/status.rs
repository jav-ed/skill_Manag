#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod common;

use common::{json_of, skillmirror};
use predicates::prelude::*;
use predicates::str::contains;
use skillmirror_testkit::World;

#[test]
fn status_names_what_differs_in_each_project_and_exits_one() {
    let world = World::standard();

    skillmirror(&world)
        .arg("status")
        .assert()
        .code(1)
        .stdout(contains("one"))
        .stdout(contains("outdated (1 file changed)"))
        .stdout(contains("outdated (1 file added, 1 file changed)"))
        .stdout(contains("mandatory, not installed"))
        .stdout(contains("local-only"))
        .stdout(contains("not in the vault"))
        .stdout(contains(
            "3 projects: 0 in sync, 3 differ (3 skill folders outdated, 4 mandatory missing)",
        ));
}

#[test]
fn the_json_document_has_one_row_per_project_and_a_summary() {
    let world = World::standard();

    let doc = json_of(skillmirror(&world).args(["status", "--json"]));

    let summary = &doc["summary"];
    assert_eq!(summary["projects"], 3);
    assert_eq!(summary["drift"], 3);
    assert_eq!(summary["outdated_skills"], 3);
    assert_eq!(summary["missing_mandatory"], 4);
    assert_eq!(summary["not_in_vault"], 1);
    assert_eq!(summary["failed"], 0);
    let rows = doc["projects"].as_array().unwrap();
    assert_eq!(rows.len(), 3);
    let one = rows
        .iter()
        .find(|r| r["project"].as_str().unwrap().ends_with("/one"))
        .unwrap();
    assert_eq!(one["status"], "drift");
    assert_eq!(one["not_in_vault"][0], "local-only");
    let astro = one["outdated"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["skill"] == "astro")
        .unwrap();
    assert_eq!(astro["files_added"], 1);
    assert_eq!(astro["files_changed"], 1);
}

#[test]
fn after_sync_and_push_everything_is_in_sync_and_the_exit_is_zero() {
    let world = World::standard();
    skillmirror(&world)
        .args(["sync", "--yes"])
        .assert()
        .success();
    skillmirror(&world)
        .args(["push", "--yes"])
        .assert()
        .success();

    skillmirror(&world)
        .arg("status")
        .assert()
        .success()
        .stdout(contains("3 projects: 3 in sync"))
        .stdout(contains("outdated").not());
    skillmirror(&world)
        .args(["status", "--all"])
        .assert()
        .success()
        .stdout(contains("up to date"));
}

#[test]
fn status_writes_nothing_not_even_a_backup_folder() {
    let world = World::standard();
    let tree = |w: &World| -> Vec<String> {
        let mut out = Vec::new();
        let mut stack = vec![w.path().to_path_buf()];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                out.push(format!(
                    "{} {}",
                    path.display(),
                    std::fs::metadata(&path).map_or(0, |m| m.len())
                ));
                if path.is_dir() {
                    stack.push(path);
                }
            }
        }
        out.sort();
        out
    };
    let before = tree(&world);

    skillmirror(&world).arg("status").assert().code(1);
    skillmirror(&world)
        .args(["status", "--json"])
        .assert()
        .code(1);

    assert_eq!(tree(&world), before);
}

#[test]
fn an_unknown_mandatory_name_is_a_hard_error() {
    let world = World::standard();
    world.vault_config(&["coding", "ghost"]).commit_vault();

    skillmirror(&world)
        .arg("status")
        .assert()
        .code(3)
        .stderr(contains("ghost"));
}

#[test]
fn a_skill_that_cannot_be_compared_is_a_partial_result() {
    let world = World::standard();
    skillmirror(&world)
        .args(["sync", "--yes"])
        .assert()
        .success();
    skillmirror(&world)
        .args(["push", "--yes"])
        .assert()
        .success();
    // The project copy of tmux becomes a link: it is reported, never followed.
    let skill = world.root().join("one/.agents/skills/tmux");
    std::fs::remove_dir_all(&skill).unwrap();
    std::os::unix::fs::symlink(world.root().join("two/.agents/skills/tmux"), &skill).unwrap();

    skillmirror(&world)
        .arg("status")
        .assert()
        .code(4)
        .stdout(contains("1 could not be compared"));
}
