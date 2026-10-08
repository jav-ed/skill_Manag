#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod common;

use common::skillmirror;
use predicates::str::contains;
use skillmirror_testkit::World;

const HEADER: &str = "---\nname: NAME\ndescription: Use it for things.\n---\nBody\n";

/// A world whose skills all have a proper header, so nothing needs attention.
fn healthy() -> World {
    let world = World::new();
    world
        .vault_file("coding/SKILL.md", &HEADER.replace("NAME", "coding"))
        .vault_file("tmux/SKILL.md", &HEADER.replace("NAME", "tmux"))
        .project_file(
            "one/.agents/skills/coding/SKILL.md",
            &HEADER.replace("NAME", "coding"),
        );
    world.vault_config(&["coding"]).commit_vault();
    world
}

#[test]
fn a_healthy_world_gets_ticks_and_exit_zero() {
    let world = healthy();

    skillmirror(&world)
        .arg("doctor")
        .assert()
        .success()
        .stdout(contains("✓ git"))
        .stdout(contains("✓ skill-files"))
        .stdout(contains("✓ mandatory"))
        .stdout(contains("nothing needs attention"));
}

#[test]
fn skills_without_a_header_are_warnings_and_exit_one() {
    // The standard world's SKILL.md files are one plain line each.
    let world = World::standard();

    skillmirror(&world)
        .arg("doctor")
        .assert()
        .code(1)
        .stdout(contains("does not start with a --- header"))
        .stdout(contains("0 errors, "));
}

#[test]
fn an_unknown_mandatory_skill_is_an_error_and_exit_three() {
    let world = healthy();
    world.vault_config(&["coding", "ghost"]).commit_vault();

    skillmirror(&world)
        .arg("doctor")
        .assert()
        .code(3)
        .stdout(contains("✗ mandatory"))
        .stdout(contains("ghost"));
}

#[test]
fn a_legacy_variable_is_reported_instead_of_stopping_the_command() {
    let world = healthy();

    skillmirror(&world)
        .env("SKILL_MANAG_VAULT", "/somewhere")
        .arg("doctor")
        .assert()
        .code(3)
        .stdout(contains("SKILL_MANAG_VAULT"))
        .stdout(contains("✓ git"));
}

#[test]
fn the_json_document_lists_checks_findings_and_a_summary() {
    let world = World::standard();

    let mut cmd = skillmirror(&world);
    cmd.args(["doctor", "--json"]);
    let doc = {
        let out = cmd.output().unwrap();
        assert_eq!(out.status.code(), Some(1));
        serde_json::from_slice::<serde_json::Value>(&out.stdout).unwrap()
    };

    let checks: Vec<_> = doc["checks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap())
        .collect();
    assert!(checks.contains(&"skill-files"), "{checks:?}");
    let findings = doc["findings"].as_array().unwrap();
    assert!(
        findings
            .iter()
            .all(|f| f["severity"] == "warning" || f["severity"] == "note")
    );
    assert!(doc["summary"]["warnings"].as_u64().unwrap() >= 3);
    assert_eq!(doc["summary"]["errors"], 0);
}

#[test]
fn doctor_writes_nothing() {
    let world = World::standard();

    skillmirror(&world).arg("doctor").assert().code(1);

    assert_eq!(
        world.read("projects/one/.agents/skills/coding/SKILL.md"),
        "coding v1"
    );
    assert!(!world.exists("home/.local"), "no state directory was made");
    assert!(
        skillmirror(&world)
            .arg("doctor")
            .output()
            .unwrap()
            .stderr
            .is_empty(),
        "findings go to stdout"
    );
}
