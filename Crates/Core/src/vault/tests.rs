use std::path::PathBuf;

use super::*;
use crate::testutil::TempTree;

fn vault_with(files: &[(&str, &str)]) -> TempTree {
    let tree = TempTree::new();
    tree.write_all(files);
    tree
}

fn names(vault: &Vault) -> Vec<&str> {
    vault.skills.keys().map(String::as_str).collect()
}

#[test]
fn skills_are_folders_with_skill_md_and_groups_are_the_folders_above() {
    let tree = vault_with(&[
        ("coding/SKILL.md", "x"),
        ("web/astro/SKILL.md", "x"),
        ("web/seo/schema/SKILL.md", "x"),
        ("web/README.md", "x"),
        ("docs/notes.md", "x"),
        (".git/HEAD", "x"),
        (".hidden/SKILL.md", "x"),
        ("config.yaml", "x"),
    ]);
    let vault = discover(tree.path()).unwrap();
    assert_eq!(names(&vault), ["astro", "coding", "schema"]);
    assert_eq!(vault.skills["schema"].group, ["web", "seo"]);
    assert_eq!(vault.skills["schema"].rel, PathBuf::from("web/seo/schema"));
    assert!(vault.skills["coding"].group.is_empty());
    assert_eq!(
        vault
            .groups
            .iter()
            .map(|g| g.to_str().unwrap())
            .collect::<Vec<_>>(),
        ["web", "web/seo"]
    );
    assert_eq!(vault.ignored.len(), 1);
    assert_eq!(vault.ignored[0].reason, IgnoredReason::NoSkillInside);
}

#[test]
fn descent_stops_at_the_first_skill_md() {
    let tree = vault_with(&[("outer/SKILL.md", "x"), ("outer/inner/SKILL.md", "x")]);
    assert_eq!(names(&discover(tree.path()).unwrap()), ["outer"]);
}

#[test]
fn duplicate_names_across_groups_are_a_hard_error() {
    let tree = vault_with(&[("a/tool/SKILL.md", "x"), ("b/tool/SKILL.md", "x")]);
    let err = discover(tree.path()).unwrap_err();
    assert!(
        matches!(err, VaultError::DuplicateSkill { ref name, .. } if name == "tool"),
        "{err}"
    );
}

#[test]
fn a_group_named_like_a_skill_is_a_hard_error() {
    let tree = vault_with(&[("web/SKILL.md", "x"), ("android/web/inner/SKILL.md", "x")]);
    assert!(matches!(
        discover(tree.path()),
        Err(VaultError::GroupSkillClash { .. })
    ));
}

#[test]
fn nesting_beyond_the_cap_is_a_hard_error() {
    let ok = vault_with(&[("a/b/c/d/skill/SKILL.md", "x")]);
    assert!(discover(ok.path()).is_ok());
    let deep = vault_with(&[("a/b/c/d/e/skill/SKILL.md", "x")]);
    assert!(matches!(
        discover(deep.path()),
        Err(VaultError::TooDeep { .. })
    ));
}

#[test]
fn symlinked_folders_are_ignored_and_listed() {
    let tree = vault_with(&[("real/SKILL.md", "x")]);
    std::os::unix::fs::symlink(tree.path().join("real"), tree.path().join("alias")).unwrap();
    let vault = discover(tree.path()).unwrap();
    assert_eq!(names(&vault), ["real"]);
    assert_eq!(vault.ignored[0].reason, IgnoredReason::Symlink);
}

#[test]
fn missing_or_non_directory_vault_is_a_hard_error() {
    let tree = vault_with(&[("file", "x")]);
    assert!(matches!(
        discover(&tree.path().join("nope")),
        Err(VaultError::Missing { .. })
    ));
    assert!(matches!(
        discover(&tree.path().join("file")),
        Err(VaultError::NotADirectory { .. })
    ));
}

fn rels(files: &SkillFiles) -> Vec<String> {
    files
        .tracked
        .iter()
        .map(|f| f.rel.to_string_lossy().into_owned())
        .collect()
}

#[test]
fn only_tracked_files_are_listed_and_untracked_ones_are_reported() {
    let tree = vault_with(&[
        ("coding/SKILL.md", "x"),
        ("coding/languages/js.md", "x"),
        ("coding-extra/SKILL.md", "x"),
        ("web/astro/SKILL.md", "x"),
        ("config.yaml", "x"),
        (".gitignore", "*.log\n"),
    ]);
    tree.git_init_commit(".");
    tree.write_all(&[
        ("coding/draft.md", "x"),
        ("coding/debug.log", "x"),
        ("web/astro/new.md", "x"),
    ]);
    let vault = discover(tree.path()).unwrap();
    let files = read_files(&vault).unwrap();

    assert_eq!(
        rels(files.get("coding").unwrap()),
        ["SKILL.md", "languages/js.md"]
    );
    assert_eq!(rels(files.get("coding-extra").unwrap()), ["SKILL.md"]);
    assert_eq!(
        files.get("coding").unwrap().untracked,
        [PathBuf::from("draft.md")]
    );
    assert_eq!(
        files.get("astro").unwrap().untracked,
        [PathBuf::from("new.md")]
    );
}

#[test]
fn non_ascii_file_names_come_through_unquoted() {
    let tree = vault_with(&[("coding/SKILL.md", "x"), ("coding/Grüße 日本.md", "x")]);
    tree.git_init_commit(".");
    let files = read_files(&discover(tree.path()).unwrap()).unwrap();
    assert_eq!(
        rels(files.get("coding").unwrap()),
        ["Grüße 日本.md", "SKILL.md"]
    );
}

#[test]
fn a_vault_that_is_not_a_git_repository_is_a_hard_error() {
    let tree = vault_with(&[("coding/SKILL.md", "x")]);
    let vault = discover(tree.path()).unwrap();
    let err = read_files(&vault).unwrap_err();
    assert!(matches!(err, VaultError::NotAGitRepo { .. }), "{err}");
    assert!(crate::Hint::hint(&err).unwrap().contains("git init"));
}

#[test]
fn tracked_symlinks_are_reported_as_problems() {
    let tree = vault_with(&[("coding/SKILL.md", "x"), ("coding/real.md", "x")]);
    std::os::unix::fs::symlink("real.md", tree.path().join("coding/link.md")).unwrap();
    tree.git_init_commit(".");
    let files = read_files(&discover(tree.path()).unwrap()).unwrap();
    let coding = files.get("coding").unwrap();
    assert_eq!(rels(coding), ["SKILL.md", "real.md"]);
    assert_eq!(
        coding.problems,
        [FileProblem {
            rel: PathBuf::from("link.md"),
            kind: ProblemKind::Symlink
        }]
    );
}

#[test]
fn a_skill_with_no_tracked_files_is_visible_as_empty() {
    let tree = vault_with(&[("coding/SKILL.md", "x"), ("fresh/SKILL.md", "x")]);
    tree.git(".", &["init", "-q"]);
    tree.git(".", &["add", "coding"]);
    tree.git(".", &["commit", "-q", "-m", "x"]);
    let files = read_files(&discover(tree.path()).unwrap()).unwrap();
    assert!(files.get("fresh").unwrap().tracked.is_empty());
    assert_eq!(files.get("fresh").unwrap().untracked.len(), 1);
}

#[test]
fn a_vault_below_the_repository_root_lists_paths_relative_to_itself() {
    let tree = vault_with(&[
        ("repo/README.md", "x"),
        ("repo/skills/coding/SKILL.md", "x"),
    ]);
    tree.git_init_commit("repo");
    let vault = discover(&tree.path().join("repo/skills")).unwrap();
    let files = read_files(&vault).unwrap();
    assert_eq!(rels(files.get("coding").unwrap()), ["SKILL.md"]);
}
