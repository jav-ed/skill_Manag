//! A new vault: a git repository with a staged `config.yaml`.

use super::*;
use crate::config::VaultConfig;
use crate::testutil::TempTree;

#[test]
fn the_folder_becomes_a_git_repository_with_a_staged_config() {
    let tree = TempTree::new();
    let dir = tree.path().join("skills");
    let root = tree.path().join("projects");

    let made = init_vault(&dir, Some(&root)).unwrap();

    assert_eq!(made.dir, dir);
    assert_eq!(
        tree.git("skills", &["status", "--porcelain"]),
        "A  config.yaml\n"
    );
    let config = VaultConfig::load(&dir).unwrap();
    assert_eq!(config.root.as_deref(), Some(root.as_path()));
    assert!(config.mandatory.is_empty(), "no skill is mandatory yet");
    assert_eq!(
        tree.git("skills", &["rev-list", "--all", "--count"]).trim(),
        "0",
        "nothing is committed"
    );
}

#[test]
fn without_a_root_the_config_has_no_root_line() {
    let tree = TempTree::new();
    let dir = tree.path().join("skills");
    init_vault(&dir, None).unwrap();
    assert!(VaultConfig::load(&dir).unwrap().root.is_none());
}

#[test]
fn a_folder_that_is_not_empty_is_refused_and_left_alone() {
    let tree = TempTree::new();
    tree.write("skills/notes.md", "mine");
    let error = init_vault(&tree.path().join("skills"), None).unwrap_err();
    assert!(
        matches!(error, VaultInitError::Folder(ProjectError::NotEmpty { .. })),
        "{error:?}"
    );
    assert_eq!(tree.read("skills/notes.md"), "mine");
    assert!(!tree.path().join("skills/.git").exists());
}

#[test]
fn an_empty_existing_folder_is_used_and_kept_when_taken_back() {
    let tree = TempTree::new();
    std::fs::create_dir(tree.path().join("skills")).unwrap();
    let made = init_vault(&tree.path().join("skills"), None).unwrap();

    made.take_back().unwrap();

    assert!(
        tree.path().join("skills").is_dir(),
        "the folder was there before"
    );
    assert_eq!(
        std::fs::read_dir(tree.path().join("skills"))
            .unwrap()
            .count(),
        0
    );
}

#[test]
fn taking_back_a_folder_the_call_made_removes_it() {
    let tree = TempTree::new();
    let made = init_vault(&tree.path().join("a/b"), None).unwrap();
    made.take_back().unwrap();
    assert!(!tree.path().join("a/b").exists());
}
