//! Vault discovery around odd names, deep folders and links that are skills themselves.

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

use super::*;
use crate::testutil::TempTree;

#[test]
fn a_skill_folder_with_a_non_utf8_name_is_a_hard_error() {
    let tree = TempTree::new();
    tree.write("coding/SKILL.md", "x");
    let odd = tree.path().join(OsStr::from_bytes(b"bad\xffname"));
    std::fs::create_dir_all(&odd).unwrap();
    std::fs::write(odd.join("SKILL.md"), "x").unwrap();
    let err = discover(tree.path()).unwrap_err();
    assert!(matches!(err, VaultError::NonUtf8Name { .. }), "{err}");
}

#[test]
fn a_deep_folder_without_any_skill_does_not_break_the_vault() {
    let tree = TempTree::new();
    tree.write("coding/SKILL.md", "x");
    tree.write("notes/a/b/c/d/e/readme.txt", "x");
    let vault = discover(tree.path()).unwrap();
    assert_eq!(vault.skills.len(), 1);
    assert!(vault.ignored.iter().any(|i| i.path.ends_with("notes")));
}

#[test]
fn a_deep_folder_that_does_hold_a_skill_is_still_too_deep() {
    let tree = TempTree::new();
    tree.write("coding/SKILL.md", "x");
    tree.write("a/b/c/d/e/deep/SKILL.md", "x");
    let err = discover(tree.path()).unwrap_err();
    assert!(matches!(err, VaultError::TooDeep { .. }), "{err}");
}

#[test]
fn a_submodule_that_is_the_skill_folder_itself_is_named_as_such() {
    let tree = TempTree::new();
    tree.write("vault/coding/SKILL.md", "x");
    tree.write("vault/sub/SKILL.md", "inside the submodule");
    tree.git_init_commit("vault");
    // Replace the tracked folder `sub` by a gitlink, as `git submodule add` would.
    tree.git("vault", &["rm", "-rq", "--cached", "sub"]);
    tree.git(
        "vault",
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            "160000,1111111111111111111111111111111111111111,sub",
        ],
    );
    let vault = discover(&tree.path().join("vault")).unwrap();
    let files = read_files(&vault).unwrap();
    let problems = &files.get("sub").unwrap().problems;
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].kind, ProblemKind::Submodule);
    assert_eq!(problems[0].rel, std::path::Path::new(SELF_PATH));
}
