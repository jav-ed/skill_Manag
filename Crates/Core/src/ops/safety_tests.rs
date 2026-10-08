//! Delete must never reach through links and never leave half a skill behind.

use std::os::unix::fs::PermissionsExt;

use super::*;
use crate::events::ignore_events;
use crate::scan::Target;
use crate::testutil::TempTree;

fn project_with_skill(tree: &TempTree, skill: &str) -> std::path::PathBuf {
    tree.write(&format!("p/.agents/skills/{skill}/SKILL.md"), "x");
    tree.path().join("p")
}

#[test]
fn delete_through_a_symlinked_agents_directory_is_refused() {
    let tree = TempTree::new();
    tree.write("shared/skills/coding/SKILL.md", "shared by other projects");
    std::fs::create_dir_all(tree.path().join("proj")).unwrap();
    std::os::unix::fs::symlink(tree.path().join("shared"), tree.path().join("proj/.agents"))
        .unwrap();
    let err = target_in_project(&tree.path().join("proj"), "coding").unwrap_err();
    assert!(matches!(err, DeleteError::LinkedParent { .. }), "{err}");
    assert!(tree.path().join("shared/skills/coding/SKILL.md").exists());
}

#[test]
fn delete_through_a_symlinked_skills_directory_is_refused_even_for_a_ready_made_target() {
    let tree = TempTree::new();
    tree.write("shared/coding/SKILL.md", "shared");
    std::fs::create_dir_all(tree.path().join("proj/.agents")).unwrap();
    std::os::unix::fs::symlink(
        tree.path().join("shared"),
        tree.path().join("proj/.agents/skills"),
    )
    .unwrap();
    let target = Target {
        project: tree.path().join("proj"),
        skill: "coding".into(),
        path: tree.path().join("proj/.agents/skills/coding"),
    };
    let report = delete(vec![target], false, &ignore_events);
    assert_eq!(report.failed(), 1);
    assert!(tree.path().join("shared/coding/SKILL.md").exists());
}

#[test]
fn a_delete_leaves_no_trash_folder_behind() {
    let tree = TempTree::new();
    let project = project_with_skill(&tree, "coding");
    let target = target_in_project(&project, "coding").unwrap();
    assert_eq!(delete(vec![target], false, &ignore_events).failed(), 0);
    let names: Vec<_> = std::fs::read_dir(project.join(".agents"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["skills"]);
}

#[test]
fn a_delete_that_cannot_finish_still_takes_the_skill_out_of_the_skills_directory() {
    use std::os::unix::fs::MetadataExt;
    if std::fs::metadata("/proc/self").unwrap().uid() == 0 {
        return; // root ignores the permission bits this test relies on
    }
    let tree = TempTree::new();
    let project = project_with_skill(&tree, "coding");
    tree.write("p/.agents/skills/coding/locked/inner.md", "x");
    let locked = project.join(".agents/skills/coding/locked");
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o555)).unwrap();
    let target = target_in_project(&project, "coding").unwrap();
    let report = delete(vec![target], false, &ignore_events);
    let trash = std::fs::read_dir(project.join(".agents"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.to_string_lossy().contains(".trash-"))
        .expect("the remains are named");
    std::fs::set_permissions(trash.join("locked"), std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(matches!(
        report.deleted[0].result,
        Err(DeleteError::RemainsKept { .. })
    ));
    assert!(
        !project.join(".agents/skills/coding").exists(),
        "never half deleted inside skills/"
    );
}

#[test]
fn an_explicit_project_may_delete_a_symlinked_skill_folder_and_only_the_link_goes() {
    let tree = TempTree::new();
    tree.write("elsewhere/keep.md", "precious");
    std::fs::create_dir_all(tree.path().join("p/.agents/skills")).unwrap();
    std::os::unix::fs::symlink(
        tree.path().join("elsewhere"),
        tree.path().join("p/.agents/skills/linked"),
    )
    .unwrap();
    let target = target_in_project(&tree.path().join("p"), "linked").unwrap();
    assert_eq!(delete(vec![target], false, &ignore_events).failed(), 0);
    assert!(tree.path().join("elsewhere/keep.md").exists());
    assert!(!tree.path().join("p/.agents/skills/linked").exists());
}
