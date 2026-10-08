#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
//! `report` writes one self-contained HTML file and never touches the vault or the projects.
mod common;

use common::skillmirror;
use predicates::prelude::*;
use predicates::str::contains;
use skillmirror_testkit::World;
use std::path::Path;

fn listing(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.file_name().is_some_and(|n| n == ".git") {
                continue;
            }
            out.push(format!(
                "{} {}",
                path.display(),
                entry.metadata().unwrap().len()
            ));
            if path.is_dir() {
                stack.push(path);
            }
        }
    }
    out.sort();
    out
}

#[test]
fn report_writes_one_html_file_and_names_it() {
    let world = World::standard();
    let out = world.path().join("out.html");

    skillmirror(&world)
        .arg("report")
        .arg("-o")
        .arg(&out)
        .assert()
        .success()
        .stdout(contains("Wrote the report to"))
        .stdout(contains("3 skills, 3 projects"));

    let html = std::fs::read_to_string(&out).unwrap();
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<!-- skillmirror report -->"));
    for needle in ["coding", "astro", "tmux", "local-only", "projects/one"] {
        assert!(html.contains(needle), "{needle} is in the report");
    }
    assert!(html.contains("-coding v1"), "the diff of the outdated copy");
    assert!(!html.contains("http://") && !html.contains("https://"));
}

#[test]
fn a_dash_prints_the_report_and_writes_no_file() {
    let world = World::standard();
    let before = listing(world.path());

    skillmirror(&world)
        .args(["report", "-o", "-"])
        .assert()
        .success()
        .stdout(contains("<!-- skillmirror report -->"))
        .stdout(contains("Wrote").not());

    assert_eq!(listing(world.path()), before);
}

#[test]
fn the_default_name_is_made_in_the_current_directory() {
    let world = World::standard();
    let here = world.path().join("here");
    std::fs::create_dir(&here).unwrap();

    skillmirror(&world)
        .current_dir(&here)
        .arg("report")
        .assert()
        .success();

    assert!(here.join("skillmirror-report.html").is_file());
}

#[test]
fn a_file_that_is_not_a_report_is_never_replaced() {
    let world = World::standard();
    let out = world.path().join("notes.html");
    std::fs::write(&out, "<h1>my notes</h1>").unwrap();

    skillmirror(&world)
        .arg("report")
        .arg("-o")
        .arg(&out)
        .assert()
        .code(3)
        .stderr(contains("is not a skillmirror report"))
        .stderr(contains("--output"));

    assert_eq!(std::fs::read_to_string(&out).unwrap(), "<h1>my notes</h1>");
}

#[test]
fn an_earlier_report_is_replaced_and_no_temporary_file_is_left() {
    let world = World::standard();
    let dir = world.path().join("reports");
    std::fs::create_dir(&dir).unwrap();
    let out = dir.join("r.html");

    skillmirror(&world)
        .arg("report")
        .arg("-o")
        .arg(&out)
        .assert()
        .success();
    skillmirror(&world)
        .arg("report")
        .arg("-o")
        .arg(&out)
        .assert()
        .success();

    let names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["r.html"], "{names:?}");
}

#[test]
fn a_missing_directory_is_a_hard_error() {
    let world = World::standard();

    skillmirror(&world)
        .arg("report")
        .arg("-o")
        .arg(world.path().join("no/such/dir/r.html"))
        .assert()
        .code(3);
}

#[test]
fn the_report_changes_nothing_in_the_vault_or_the_projects() {
    let world = World::standard();
    let before = listing(world.path());
    let out = world.path().join("r.html");

    skillmirror(&world)
        .arg("report")
        .arg("-o")
        .arg(&out)
        .assert()
        .success();

    let mut after = listing(world.path());
    after.retain(|line| !line.contains("r.html"));
    assert_eq!(after, before);
}

#[test]
fn an_unknown_mandatory_skill_is_a_hard_error_and_writes_no_file() {
    let world = World::standard();
    world.vault_config(&["coding", "nope"]).commit_vault();
    let out = world.path().join("r.html");

    skillmirror(&world)
        .arg("report")
        .arg("-o")
        .arg(&out)
        .assert()
        .code(3)
        .stderr(contains("nope"));

    assert!(!out.exists());
}
