//! Bridges: a link is made only where nothing is, and never through or over anything.

use std::os::unix::fs::symlink;
use std::path::Path;

use super::*;
use crate::testutil::TempTree;

fn claude() -> Vec<String> {
    vec!["claude".to_string()]
}

/// A project with a skill in `.agents/skills`.
fn project(tree: &TempTree) -> std::path::PathBuf {
    tree.write("p/.agents/skills/coding/SKILL.md", "coding\n");
    tree.path().join("p")
}

fn only(bridges: Vec<Bridge>) -> Bridge {
    assert_eq!(bridges.len(), 1, "{bridges:?}");
    bridges.into_iter().next().unwrap()
}

#[test]
fn a_missing_bridge_is_made_as_a_relative_link_and_then_is_in_place() {
    let tree = TempTree::new();
    let project = project(&tree);
    let bridge = only(plan_project_bridges(&claude(), &project));
    assert_eq!(bridge.state, BridgeState::Missing);
    assert_eq!(bridge.points_to, Path::new("../.agents/skills"));
    assert_eq!(bridge.link, project.join(".claude/skills"));

    create_bridge(&bridge).unwrap();

    assert_eq!(
        std::fs::read_link(project.join(".claude/skills")).unwrap(),
        Path::new("../.agents/skills"),
        "the link is relative, so it survives moving the project"
    );
    assert_eq!(
        std::fs::read_to_string(project.join(".claude/skills/coding/SKILL.md")).unwrap(),
        "coding\n",
        "reading through the link finds the skills"
    );
    assert_eq!(
        only(plan_project_bridges(&claude(), &project)).state,
        BridgeState::InPlace
    );
}

#[test]
fn a_bridge_is_never_made_twice_or_over_something() {
    let tree = TempTree::new();
    let project = project(&tree);
    let bridge = only(plan_project_bridges(&claude(), &project));
    create_bridge(&bridge).unwrap();

    let again = create_bridge(&bridge);

    assert!(
        matches!(again, Err(BridgeError::NotFree { .. })),
        "{again:?}"
    );
}

#[test]
fn a_real_folder_is_occupied_and_keeps_every_file() {
    let tree = TempTree::new();
    let project = project(&tree);
    tree.write("p/.claude/skills/mine/SKILL.md", "my own skill\n");

    let bridge = only(plan_project_bridges(&claude(), &project));
    let result = create_bridge(&bridge);

    assert_eq!(bridge.state, BridgeState::Occupied);
    assert!(bridge.state.is_problem());
    assert!(matches!(result, Err(BridgeError::NotFree { .. })));
    assert_eq!(
        std::fs::read_to_string(project.join(".claude/skills/mine/SKILL.md")).unwrap(),
        "my own skill\n"
    );
}

#[test]
fn a_link_to_somewhere_else_is_reported_and_left() {
    let tree = TempTree::new();
    let project = project(&tree);
    tree.write("elsewhere/x.md", "x");
    std::fs::create_dir_all(project.join(".claude")).unwrap();
    symlink(
        tree.path().join("elsewhere"),
        project.join(".claude/skills"),
    )
    .unwrap();

    let bridge = only(plan_project_bridges(&claude(), &project));

    assert!(
        matches!(bridge.state, BridgeState::Elsewhere(_)),
        "{:?}",
        bridge.state
    );
    assert!(create_bridge(&bridge).is_err());
    assert_eq!(
        std::fs::read_link(project.join(".claude/skills")).unwrap(),
        tree.path().join("elsewhere")
    );
}

#[test]
fn an_absolute_link_to_the_same_folder_counts_as_in_place() {
    let tree = TempTree::new();
    let project = project(&tree);
    std::fs::create_dir_all(project.join(".claude")).unwrap();
    symlink(
        project.join(".agents/skills"),
        project.join(".claude/skills"),
    )
    .unwrap();

    assert_eq!(
        only(plan_project_bridges(&claude(), &project)).state,
        BridgeState::InPlace
    );
}

#[test]
fn nothing_is_written_through_a_claude_folder_that_is_a_link() {
    let tree = TempTree::new();
    let project = project(&tree);
    tree.write("shared/.keep", "");
    symlink(tree.path().join("shared"), project.join(".claude")).unwrap();

    let bridge = only(plan_project_bridges(&claude(), &project));
    let result = create_bridge(&bridge);

    assert!(
        matches!(bridge.state, BridgeState::ParentIsLink(_)),
        "{:?}",
        bridge.state
    );
    assert!(result.is_err());
    assert!(
        !tree.path().join("shared/skills").exists(),
        "nothing was written behind the link"
    );
}

#[test]
fn a_project_without_skills_gets_no_dangling_link() {
    let tree = TempTree::new();
    tree.write("p/src/main.rs", "");
    let project = tree.path().join("p");

    let bridge = only(plan_project_bridges(&claude(), &project));

    assert!(create_bridge(&bridge).is_err());
    assert!(!project.join(".claude").exists());
}

#[test]
fn no_targets_means_no_bridges() {
    let tree = TempTree::new();
    let project = project(&tree);

    assert!(plan_project_bridges(&[], &project).is_empty());
    assert!(plan_project_bridges(&["unknown".to_string()], &project).is_empty());
}

#[test]
fn a_dangling_link_beside_missing_skills_is_not_in_place() {
    let tree = TempTree::new();
    tree.write("p/readme.md", "x\n");
    let project = tree.path().join("p");
    std::fs::create_dir_all(project.join(".claude")).unwrap();
    // Neither the link nor `.agents/skills` can be resolved; two failed lookups must not count as equal.
    symlink("/nowhere/at/all", project.join(".claude/skills")).unwrap();

    let bridge = only(plan_project_bridges(&claude(), &project));

    assert!(
        matches!(bridge.state, BridgeState::Elsewhere(_)),
        "{:?}",
        bridge.state
    );
}
