//! Adopting a project's skill folder into the vault: copied, staged, never committed, project untouched.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use super::*;
use crate::testutil::Fixture;

fn world() -> Fixture {
    let world = Fixture::new(&[
        ("coding/SKILL.md", "coding"),
        ("web/astro/SKILL.md", "astro"),
        ("config.yaml", "mandatory: []\n"),
    ]);
    world.tree.write_all(&[
        (
            "proj/.agents/skills/tmux/SKILL.md",
            "---\nname: tmux\n---\n",
        ),
        ("proj/.agents/skills/tmux/refs/panes.md", "panes"),
        ("proj/.agents/skills/tmux/run.sh", "#!/bin/sh\n"),
    ]);
    std::fs::set_permissions(
        world.tree.path().join("proj/.agents/skills/tmux/run.sh"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    world
}

fn proj(world: &Fixture) -> PathBuf {
    world.tree.path().join("proj")
}

fn staged(world: &Fixture) -> String {
    world.tree.git("vault", &["status", "--porcelain"])
}

fn names(files: &[PathBuf]) -> Vec<String> {
    files.iter().map(|f| f.display().to_string()).collect()
}

#[test]
fn the_folder_is_copied_with_its_modes_and_staged_not_committed() {
    let world = world();
    let plan = plan_adopt(&world.vault, &proj(&world), "tmux", "tools").unwrap();
    assert_eq!(names(&plan.files), ["SKILL.md", "refs/panes.md", "run.sh"]);

    let made = adopt(&world.vault, &plan).unwrap();

    assert_eq!(made.dir, world.vault.path.join("tools/tmux"));
    assert!(
        made.left_out.is_empty(),
        "nothing was left out: {:?}",
        made.left_out
    );
    assert_eq!(
        staged(&world),
        "A  tools/tmux/SKILL.md\nA  tools/tmux/refs/panes.md\nA  tools/tmux/run.sh\n"
    );
    let bits = std::fs::metadata(made.dir.join("run.sh"))
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(bits, 0o755, "the executable bit travels");
    assert_eq!(
        std::fs::read_to_string(made.dir.join("refs/panes.md")).unwrap(),
        "panes"
    );
    assert!(
        proj(&world).join(".agents/skills/tmux/run.sh").exists(),
        "the project keeps its folder"
    );
}

#[test]
fn once_adopted_the_projects_folder_is_in_sync_with_the_vault() {
    let world = world();
    let plan = plan_adopt(&world.vault, &proj(&world), "tmux", "").unwrap();
    adopt(&world.vault, &plan).unwrap();

    let now = world.refresh();
    let target = now.target("proj", "tmux");
    let plan = now.plan(vec![target]);
    assert_eq!(plan.entries.len(), 1);
    let skill_plan = plan.entries[0].result.as_ref().unwrap();
    assert_eq!(
        skill_plan.kind,
        crate::plan::PlanKind::Unchanged,
        "a sync right after adopting has nothing to do"
    );
}

#[test]
fn what_cannot_be_copied_faithfully_is_refused_before_anything_is_written() {
    let world = world();
    std::os::unix::fs::symlink(
        "/etc/passwd",
        proj(&world).join(".agents/skills/tmux/passwd"),
    )
    .unwrap();

    let error = plan_adopt(&world.vault, &proj(&world), "tmux", "").unwrap_err();

    assert!(
        matches!(
            error,
            AuthorError::UnsupportedEntry {
                what: "symbolic link",
                ..
            }
        ),
        "{error:?}"
    );
    assert_eq!(staged(&world), "");
}

#[test]
fn a_nested_repository_is_refused() {
    let world = world();
    world.tree.write(
        "proj/.agents/skills/tmux/.git/HEAD",
        "ref: refs/heads/main\n",
    );
    let error = plan_adopt(&world.vault, &proj(&world), "tmux", "").unwrap_err();
    assert!(
        matches!(
            error,
            AuthorError::UnsupportedEntry {
                what: "git repository",
                ..
            }
        ),
        "{error:?}"
    );
}

#[test]
fn the_other_refusals_name_what_is_wrong() {
    let world = world();
    let vault = &world.vault;
    let project = proj(&world);

    assert!(matches!(
        plan_adopt(vault, &project, "ghost", ""),
        Err(AuthorError::NotInProject { .. })
    ));
    assert!(matches!(
        plan_adopt(vault, &project, "../x", ""),
        Err(AuthorError::InvalidName { .. })
    ));
    world
        .tree
        .write("proj/.agents/skills/bare/notes.md", "no header file");
    assert!(matches!(
        plan_adopt(vault, &project, "bare", ""),
        Err(AuthorError::NoSkillFile { .. })
    ));
    world
        .tree
        .write("proj/.agents/skills/coding/SKILL.md", "other coding");
    assert!(matches!(
        plan_adopt(vault, &project, "coding", ""),
        Err(AuthorError::NameTaken { .. })
    ));
}

#[test]
fn a_projects_skills_folder_that_is_a_link_is_not_read() {
    let world = world();
    let linked = world.tree.path().join("linked");
    std::fs::create_dir_all(linked.join(".agents")).unwrap();
    std::os::unix::fs::symlink(
        proj(&world).join(".agents/skills"),
        linked.join(".agents/skills"),
    )
    .unwrap();
    let error = plan_adopt(&world.vault, &linked, "tmux", "").unwrap_err();
    assert!(
        matches!(error, AuthorError::LinkedParent { .. }),
        "{error:?}"
    );
}

#[test]
fn a_destination_that_appeared_after_the_plan_is_never_overwritten() {
    let world = world();
    let plan = plan_adopt(&world.vault, &proj(&world), "tmux", "").unwrap();
    world.tree.write("vault/tmux/mine.md", "written meanwhile");

    let error = adopt(&world.vault, &plan).unwrap_err();

    assert!(matches!(error, AuthorError::Occupied { .. }), "{error:?}");
    assert_eq!(
        std::fs::read_to_string(world.vault.path.join("tmux/mine.md")).unwrap(),
        "written meanwhile",
        "what was there stays"
    );
    assert!(!Path::new(&world.vault.path.join("tmux/SKILL.md")).exists());
}

#[test]
fn when_git_refuses_the_copy_is_taken_back() {
    let world = world();
    world.tree.write("vault/.gitignore", "tmux/\n");
    world.tree.git("vault", &["add", ".gitignore"]);
    let plan = plan_adopt(&world.vault, &proj(&world), "tmux", "newgroup").unwrap();

    let error = adopt(&world.vault, &plan).unwrap_err();

    assert!(matches!(error, AuthorError::GitAdd { .. }), "{error:?}");
    assert!(!world.vault.path.join("newgroup").exists());
}

#[test]
fn a_name_with_glob_characters_stages_only_its_own_folder() {
    let world = world();
    world.tree.write("proj/.agents/skills/*/SKILL.md", "star");
    world
        .tree
        .write("vault/stray.md", "an unrelated file in the vault");

    let plan = plan_adopt(&world.vault, &proj(&world), "*", "").unwrap();
    adopt(&world.vault, &plan).unwrap();

    let status = staged(&world);
    assert!(status.contains("A  */SKILL.md"), "{status}");
    assert!(
        status.contains("?? stray.md"),
        "git add must not have taken the stray file: {status}"
    );
}
