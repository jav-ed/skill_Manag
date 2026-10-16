//! Local changes: the state of every skill of a project against the vault, and finding the project.

use std::path::PathBuf;

use super::*;
use crate::ops::inspect::status_tests::open;
use crate::scan::ScanOptions;
use crate::testutil::TempTree;

fn edit(rel: &str, kind: EditKind) -> Edit {
    Edit {
        rel: PathBuf::from(rel),
        kind,
    }
}

fn state<'a>(changes: &'a LocalChanges, name: &str) -> &'a LocalState {
    &changes
        .skills
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("no skill {name}"))
        .state
}

fn vault_config(tree: &TempTree) {
    tree.write(
        "vault/config.yaml",
        &format!("root: {}/projects\n", tree.path().display()),
    );
}

#[test]
fn every_skill_of_a_project_is_same_edited_behind_or_new() {
    let tree = TempTree::new();
    vault_config(&tree);
    tree.write_all(&[
        ("vault/coding/SKILL.md", "coding v1"),
        ("vault/tmux/SKILL.md", "tmux v1"),
        ("vault/web/astro/SKILL.md", "astro v1"),
        ("vault/web/astro/ref.md", "ref"),
    ]);
    tree.git_init_commit("vault");
    tree.write("vault/coding/SKILL.md", "coding v2");
    tree.git("vault", &["commit", "-q", "-am", "coding v2"]);
    tree.write_all(&[
        ("projects/p/.agents/skills/tmux/SKILL.md", "tmux v1"),
        // an old version of the vault's: the vault went on, the project did not change anything
        ("projects/p/.agents/skills/coding/SKILL.md", "coding v1"),
        // changed here: a file changed, one added, one dropped
        ("projects/p/.agents/skills/astro/SKILL.md", "astro mine"),
        ("projects/p/.agents/skills/astro/notes.md", "mine"),
        ("projects/p/.agents/skills/local-only/SKILL.md", "mine"),
    ]);
    let ws = open(&tree);

    let changes = local_changes(&ws, &tree.path().join("projects/p")).unwrap();

    assert_eq!(state(&changes, "tmux"), &LocalState::Same);
    assert_eq!(
        state(&changes, "coding"),
        &LocalState::Behind { commits: 1 }
    );
    assert_eq!(
        state(&changes, "astro"),
        &LocalState::Edited(vec![
            edit("SKILL.md", EditKind::Modified),
            edit("notes.md", EditKind::Added),
            edit("ref.md", EditKind::Removed),
        ])
    );
    assert_eq!(state(&changes, "local-only"), &LocalState::New);
    let astro = changes.skills.iter().find(|s| s.name == "astro").unwrap();
    assert_eq!(astro.group, ["web"]);
    assert_eq!(changes.edited().count(), 1);
}

#[test]
fn an_old_version_is_found_even_after_the_skill_moved_to_another_group() {
    let tree = TempTree::new();
    vault_config(&tree);
    tree.write("vault/old/coding/SKILL.md", "coding v1");
    tree.git_init_commit("vault");
    tree.write("vault/new/placeholder/SKILL.md", "x");
    tree.git("vault", &["mv", "old/coding", "new/coding"]);
    tree.write("vault/new/coding/SKILL.md", "coding v2");
    tree.git("vault", &["add", "-A"]);
    tree.git("vault", &["commit", "-q", "-m", "moved and changed"]);
    tree.write("projects/p/.agents/skills/coding/SKILL.md", "coding v1");
    let ws = open(&tree);

    let changes = local_changes(&ws, &tree.path().join("projects/p")).unwrap();

    assert_eq!(
        state(&changes, "coding"),
        &LocalState::Behind { commits: 1 }
    );
}

#[test]
fn the_search_goes_up_to_the_top_of_the_repository_and_not_further() {
    let tree = TempTree::new();
    tree.write_all(&[
        ("outer/.agents/skills/x/SKILL.md", "x"),
        ("outer/repo/src/main.rs", ""),
        ("outer/repo/.agents/skills/y/SKILL.md", "y"),
        ("outer/plain/src/lib.rs", ""),
    ]);
    tree.git("outer/repo", &["init", "-q"]);
    tree.git("outer/plain", &["init", "-q"]);
    let options = ScanOptions::default();
    let at = |rel: &str| tree.path().join(rel);

    assert_eq!(
        locate(&at("outer/repo"), &options).unwrap(),
        Located::One {
            root: at("outer/repo"),
            found: Found::Here
        }
    );
    assert_eq!(
        locate(&at("outer/repo/src"), &options).unwrap(),
        Located::One {
            root: at("outer/repo"),
            found: Found::Above
        }
    );
    // `outer` has skills too, but the repository `plain` is its own project, so the search ends there.
    let error = locate(&at("outer/plain/src"), &options).unwrap_err();
    assert!(matches!(error, LocalError::NoProject { .. }), "{error}");
}

#[test]
fn below_the_folder_one_project_is_taken_and_several_are_listed() {
    let tree = TempTree::new();
    tree.write_all(&[
        ("one/a/.agents/skills/x/SKILL.md", "x"),
        ("two/a/.agents/skills/x/SKILL.md", "x"),
        ("two/b/.agents/skills/x/SKILL.md", "x"),
    ]);
    let options = ScanOptions::default();
    let at = |rel: &str| tree.path().join(rel);

    assert_eq!(
        locate(&at("one"), &options).unwrap(),
        Located::One {
            root: at("one/a"),
            found: Found::Below
        }
    );
    assert_eq!(
        locate(&at("two"), &options).unwrap(),
        Located::Several(vec![at("two/a"), at("two/b")])
    );
}
