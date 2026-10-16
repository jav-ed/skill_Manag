#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
//! `init` makes a git repository on `main` (unless the folder is inside one) and runs the doc-start skill's own
//! script for the scaffold. The skill in these vaults has a small stand-in for the script.
mod common;

use common::{json_of, skillmirror};
use predicates::str::contains;
use skillmirror_testkit::World;

/// Makes `Scratch/` and `.gitignore` like the real script does, and says so.
const BOOTSTRAP: &str = "set -e\ncd \"$1\"\nmkdir -p Scratch Repos\nprintf 'Repos/\\nScratch/\\n' >> .gitignore\necho '  created: Scratch/'\n";

fn doc_start_world(bootstrap: &str) -> World {
    let world = World::standard();
    world
        .vault_file("doc-start/SKILL.md", "doc-start v1")
        .vault_file("doc-start/Code/bootstrap.sh", bootstrap)
        .vault_file("doc-start/Code/install_Scratch.sh", "#!/usr/bin/env bash\n")
        .vault_file(
            "doc-start/Templates/clean_Scratch.sh",
            "#!/usr/bin/env bash\n",
        );
    world.vault_config(&["coding", "doc-start"]).commit_vault();
    world
}

fn init(world: &World, name: &str) -> assert_cmd::Command {
    let mut cmd = skillmirror(world);
    cmd.args(["init", "--no-agents-md", "--yes"])
        .arg(world.root().join(name));
    cmd
}

#[test]
fn a_new_project_is_a_repository_on_main_with_the_scaffold() {
    let world = doc_start_world(BOOTSTRAP);

    init(&world, "fresh")
        .assert()
        .success()
        .stdout(contains("Project setup"))
        .stdout(contains("created: Scratch/"));

    assert_eq!(
        world.read("projects/fresh/.git/HEAD").trim(),
        "ref: refs/heads/main"
    );
    assert!(world.exists("projects/fresh/Scratch"));
    assert_eq!(
        world.read("projects/fresh/.gitignore"),
        "Repos/\nScratch/\n"
    );
    assert!(world.exists("projects/fresh/.agents/skills/doc-start/SKILL.md"));
}

#[test]
fn no_git_and_no_scaffold_switch_each_part_off() {
    let world = doc_start_world(BOOTSTRAP);

    init(&world, "no_repo").arg("--no-git").assert().success();
    assert!(!world.exists("projects/no_repo/.git"));
    assert!(
        world.exists("projects/no_repo/Scratch"),
        "the scaffold still ran"
    );

    init(&world, "no_scaffold")
        .arg("--no-scaffold")
        .assert()
        .success();
    assert!(world.exists("projects/no_scaffold/.git"));
    assert!(!world.exists("projects/no_scaffold/Scratch"));
}

#[test]
fn a_folder_inside_a_repository_gets_no_second_one() {
    let world = doc_start_world(BOOTSTRAP);
    let outer = world.root().join("outer");
    std::fs::create_dir_all(&outer).unwrap();
    let made = std::process::Command::new("git")
        .args(["init", "-q", "--initial-branch=main"])
        .arg(&outer)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .status()
        .unwrap();
    assert!(made.success());

    init(&world, "outer/inner")
        .assert()
        .success()
        .stdout(contains("none made, the folder is inside the repository"));

    assert!(!world.exists("projects/outer/inner/.git"));
    assert!(world.exists("projects/outer/inner/Scratch"));
}

#[test]
fn the_dry_run_lists_the_steps_and_makes_nothing() {
    let world = doc_start_world(BOOTSTRAP);

    init(&world, "ghost")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(contains("git: a new repository on the branch main"))
        .stdout(contains(
            "scaffold: the doc-start skill's Code/bootstrap.sh",
        ));

    assert!(!world.exists("projects/ghost"));
}

#[test]
fn the_json_has_a_setup_row() {
    let world = doc_start_world(BOOTSTRAP);

    let doc = json_of(init(&world, "fresh").arg("--json"));

    assert_eq!(doc["setup"]["state"], "done");
    assert_eq!(doc["setup"]["steps"].as_array().unwrap().len(), 2);
    assert_eq!(doc["setup"]["script"][0], "created: Scratch/");
}

#[test]
fn without_the_doc_start_skill_it_is_a_hard_error_before_anything_is_made() {
    let world = World::standard();

    init(&world, "fresh")
        .assert()
        .code(3)
        .stderr(contains("doc-start"))
        .stderr(contains("--no-scaffold"));

    assert!(!world.exists("projects/fresh"));
}

#[test]
fn a_doc_start_skill_without_its_script_in_git_is_a_hard_error_too() {
    let world = World::standard();
    world.vault_file("doc-start/SKILL.md", "doc-start v1");
    world.vault_config(&["coding", "doc-start"]).commit_vault();

    init(&world, "fresh")
        .assert()
        .code(3)
        .stderr(contains("Code/bootstrap.sh"));

    assert!(!world.exists("projects/fresh"));
}

#[test]
fn a_failing_script_keeps_the_skills_and_says_what_to_run_again() {
    let world = doc_start_world("echo 'no space left' >&2\nexit 7\n");

    init(&world, "fresh")
        .assert()
        .code(4)
        .stdout(contains("no space left"))
        .stdout(contains("run `bash"));

    assert!(
        world.exists("projects/fresh/.agents/skills/coding/SKILL.md"),
        "the skills are installed"
    );
}
