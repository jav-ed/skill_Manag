//! `new`, `adopt` and `vault init`: growing the vault from the command line.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod common;

use common::{bare, json_of, skillmirror};
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

#[test]
fn a_new_skill_is_staged_and_can_be_added_to_a_project_at_once() {
    let world = World::standard();

    skillmirror(&world)
        .args([
            "new",
            "notes",
            "--group",
            "docs",
            "--description",
            "Keep notes",
        ])
        .assert()
        .success()
        .stdout(contains("notes"))
        .stdout(contains("SKILL.md"))
        .stdout(contains("not committed"));

    assert_eq!(vault_status(&world), "A  docs/notes/SKILL.md\n");
    skillmirror(&world)
        .args(["add", "notes", "--project"])
        .arg(world.root().join("three"))
        .arg("--yes")
        .assert()
        .success();
    let installed = world.read("projects/three/.agents/skills/notes/SKILL.md");
    assert!(
        installed.contains("description: \"Keep notes\""),
        "{installed}"
    );
}

#[test]
fn a_dry_run_creates_nothing_and_says_where() {
    let world = World::standard();

    skillmirror(&world)
        .args(["new", "later", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("Would create"));
    let doc = json_of(skillmirror(&world).args(["new", "later", "--dry-run", "--json"]));

    assert_eq!(doc["dry_run"], true);
    assert!(doc["dir"].as_str().unwrap().ends_with("/later"));
    assert!(!world.exists("vault/later"));
}

#[test]
fn a_taken_name_is_a_hard_error_with_a_hint_and_nothing_changes() {
    let world = World::standard();

    skillmirror(&world)
        .args(["new", "coding"])
        .assert()
        .code(3)
        .stderr(contains("already in the vault"))
        .stderr(contains("hint:"));
    skillmirror(&world)
        .args(["new", "Bad Name"])
        .assert()
        .code(3)
        .stderr(contains("not a usable skill name"));
    assert_eq!(vault_status(&world), "");
}

#[test]
fn adopting_takes_a_projects_skill_into_the_vault_and_the_project_stays_in_sync() {
    let world = World::standard();

    skillmirror(&world)
        .args(["adopt", "local-only", "--from"])
        .arg(world.root().join("one"))
        .args(["--group", "mine"])
        .assert()
        .success()
        .stdout(contains("SKILL.md"));

    assert_eq!(vault_status(&world), "A  mine/local-only/SKILL.md\n");
    skillmirror(&world)
        .args(["sync", "local-only", "--check"])
        .assert()
        .success();
    skillmirror(&world)
        .args(["info", "local-only"])
        .assert()
        .success()
        .stdout(contains("mine"))
        .stdout(contains("up to date"));
}

#[test]
fn adopting_refuses_what_the_vault_has_and_what_the_project_lacks() {
    let world = World::standard();
    let one = world.root().join("one");

    skillmirror(&world)
        .args(["adopt", "coding", "--from"])
        .arg(&one)
        .assert()
        .code(3)
        .stderr(contains("already in the vault"));
    skillmirror(&world)
        .args(["adopt", "ghost", "--from"])
        .arg(&one)
        .assert()
        .code(3)
        .stderr(contains("no skill folder"))
        .stderr(contains("hint:"));
    assert_eq!(vault_status(&world), "");
}

#[test]
fn an_adopt_dry_run_copies_nothing() {
    let world = World::standard();

    skillmirror(&world)
        .args(["adopt", "local-only", "--dry-run", "--from"])
        .arg(world.root().join("one"))
        .assert()
        .success()
        .stdout(contains("Would create"));

    assert!(!world.exists("vault/local-only"));
}

#[test]
fn vault_init_makes_a_repository_and_becomes_the_default_vault() {
    let world = World::new();
    let dir = world.path().join("fresh-vault");

    bare(&world)
        .args(["vault", "init"])
        .arg(&dir)
        .arg("--root")
        .arg(world.root())
        .assert()
        .success()
        .stdout(contains("Created"))
        .stdout(contains("now your default vault"));

    assert!(dir.join(".git").is_dir());
    let config = std::fs::read_to_string(dir.join("config.yaml")).unwrap();
    assert!(config.contains("root:"), "{config}");
    // The new vault is found without any flag.
    bare(&world)
        .arg("skills")
        .assert()
        .success()
        .stdout(contains("0 skills"));
    bare(&world).args(["new", "first"]).assert().success();
    assert!(dir.join("first/SKILL.md").is_file());
}

#[test]
fn vault_init_leaves_another_default_vault_alone_unless_told() {
    let world = World::standard();
    let dir = world.path().join("second-vault");
    // The standard world's flags name its vault; make that vault the default by pointer too.
    let pointer = world.path().join("home/.config/skillmirror/vault");
    std::fs::create_dir_all(pointer.parent().unwrap()).unwrap();
    std::fs::write(&pointer, format!("{}\n", world.vault().display())).unwrap();

    bare(&world)
        .args(["vault", "init"])
        .arg(&dir)
        .assert()
        .success()
        .stdout(contains("still"))
        .stdout(contains("--use"));
    assert_eq!(
        std::fs::read_to_string(&pointer).unwrap().trim(),
        world.vault().display().to_string()
    );

    let third = world.path().join("third-vault");
    bare(&world)
        .args(["vault", "init"])
        .arg(&third)
        .arg("--use")
        .assert()
        .success();
    assert_eq!(
        std::fs::read_to_string(&pointer).unwrap().trim(),
        third.display().to_string()
    );
}

#[test]
fn vault_init_refuses_a_folder_that_has_files_in_it() {
    let world = World::new();
    let dir = world.path().join("busy");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("notes.md"), "mine").unwrap();

    bare(&world)
        .args(["vault", "init"])
        .arg(&dir)
        .assert()
        .code(3)
        .stderr(contains("not empty"));
    assert!(!dir.join(".git").exists());
    assert!(!world.path().join("home/.config/skillmirror/vault").exists());
}

#[test]
fn a_vault_init_dry_run_creates_nothing_and_writes_no_pointer() {
    let world = World::new();
    let dir = world.path().join("dry");

    bare(&world)
        .args(["vault", "init"])
        .arg(&dir)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(contains("Would create"));

    assert!(!dir.exists());
    assert!(!world.path().join("home/.config/skillmirror/vault").exists());
}

#[test]
fn a_dry_run_checks_the_folder_too() {
    let world = World::new();
    let dir = world.path().join("busy");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("notes.md"), "mine").unwrap();

    bare(&world)
        .args(["vault", "init", "--dry-run"])
        .arg(&dir)
        .assert()
        .code(3)
        .stderr(contains("not empty"));
}

#[test]
fn a_file_git_does_not_take_is_a_partial_result_not_a_quiet_success() {
    let world = World::standard();
    world.vault_file(".gitignore", "SKILL.md\n");
    world.commit_vault();

    skillmirror(&world)
        .args(["new", "ignored"])
        .assert()
        .code(4)
        .stdout(contains("not taken by git"));
}
