//! Promoting: the project's copy replaces the vault's, staged and not committed, with the old one kept so
//! that undo brings it back; and nothing is replaced that was not looked at.

use crate::config::Dirs;
use crate::events::ignore_events;
use crate::ops::inspect::status_tests::open;
use crate::ops::*;
use crate::testutil::TempTree;

/// A vault with coding and web/astro; project p has an edited coding, an equal astro, and a skill the
/// vault does not have.
fn world() -> (TempTree, Workspace) {
    let tree = TempTree::new();
    tree.write(
        "vault/config.yaml",
        &format!("root: {}/projects\n", tree.path().display()),
    );
    tree.write_all(&[
        ("vault/coding/SKILL.md", "coding v1"),
        ("vault/web/astro/SKILL.md", "astro v1"),
        ("projects/p/.agents/skills/coding/SKILL.md", "coding mine"),
        ("projects/p/.agents/skills/coding/extra.md", "extra"),
        ("projects/p/.agents/skills/astro/SKILL.md", "astro v1"),
        ("projects/p/.agents/skills/local-only/SKILL.md", "mine"),
    ]);
    tree.git_init_commit("vault");
    let ws = open(&tree);
    (tree, ws)
}

fn project(tree: &TempTree) -> std::path::PathBuf {
    tree.path().join("projects/p")
}

fn dirs(tree: &TempTree) -> Dirs {
    Dirs::under(&tree.path().join("home"))
}

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(ToString::to_string).collect()
}

fn vault_status(tree: &TempTree) -> String {
    tree.git("vault", &["status", "--porcelain"])
}

fn stage_folders(tree: &TempTree, group: &str) -> Vec<String> {
    std::fs::read_dir(tree.path().join(group))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with(".stage-") || n.starts_with(".trash-"))
        .collect()
}

#[test]
fn the_copy_replaces_the_vaults_staged_not_committed_and_undo_brings_the_old_one_back() {
    let (tree, ws) = world();
    let changes = local_changes(&ws, &project(&tree)).unwrap();
    let plan = plan_promote(&ws, &changes, &names(&["coding"])).unwrap();

    let report = promote(&ws.vault, &plan, &dirs(&tree)).unwrap();

    assert_eq!(report.failed(), 0, "{report:?}");
    assert_eq!(report.backup.stored, 1, "the old folder was kept");
    assert_eq!(tree.read("vault/coding/SKILL.md"), "coding mine");
    assert_eq!(tree.read("vault/coding/extra.md"), "extra");
    assert_eq!(
        vault_status(&tree),
        "M  coding/SKILL.md\nA  coding/extra.md\n",
        "staged and not committed"
    );
    assert_eq!(
        tree.git("vault", &["rev-list", "--count", "HEAD"]).trim(),
        "1"
    );
    assert_eq!(
        tree.read("projects/p/.agents/skills/coding/SKILL.md"),
        "coding mine"
    );
    assert!(stage_folders(&tree, "vault").is_empty());
    // Now the vault has what the project has, so a sync would not take it back.
    let after = local_changes(&open(&tree), &project(&tree)).unwrap();
    assert_eq!(
        after
            .skills
            .iter()
            .find(|s| s.name == "coding")
            .unwrap()
            .state,
        LocalState::Same
    );

    let undone = do_undo(&dirs(&tree), &report.backup.id, &ignore_events).unwrap();

    assert_eq!(undone.failed(), 0);
    assert_eq!(tree.read("vault/coding/SKILL.md"), "coding v1");
    assert!(!tree.path().join("vault/coding/extra.md").exists());
    assert_eq!(
        vault_status(&tree),
        "",
        "the index is back at what was committed"
    );
}

#[test]
fn only_an_edited_skill_can_be_promoted_and_nothing_is_written_for_the_others() {
    let (tree, ws) = world();
    let changes = local_changes(&ws, &project(&tree)).unwrap();
    let refused = |name: &str| {
        let error = plan_promote(&ws, &changes, &names(&[name])).unwrap_err();
        assert!(matches!(
            error,
            PromoteError::NotEdited { .. } | PromoteError::NotInProject { .. }
        ));
        error.to_string()
    };

    assert!(refused("astro").contains("equals the vault's copy"));
    assert!(refused("local-only").contains("`adopt`"));
    assert!(refused("missing").contains("no skill folder"));
    assert_eq!(vault_status(&tree), "");
}

#[test]
fn a_vault_skill_with_unstaged_changes_is_refused() {
    let (tree, ws) = world();
    tree.write("vault/coding/SKILL.md", "coding being written in the vault");
    let changes = local_changes(&ws, &project(&tree)).unwrap();

    let error = plan_promote(&ws, &changes, &names(&["coding"])).unwrap_err();

    assert!(matches!(error, PromoteError::VaultDirty { .. }), "{error}");
    assert!(error.to_string().contains("SKILL.md"), "{error}");
}

#[test]
fn an_edit_made_after_the_plan_stops_that_skill_and_leaves_both_sides_alone() {
    let (tree, ws) = world();
    tree.write_all(&[
        ("vault/web/astro/SKILL.md", "astro v1"),
        ("projects/p/.agents/skills/astro/SKILL.md", "astro mine"),
    ]);
    tree.git("vault", &["add", "-A"]);
    let changes = local_changes(&ws, &project(&tree)).unwrap();
    let plan = plan_promote(&ws, &changes, &names(&["coding", "astro"])).unwrap();
    // The project's coding is edited again, and the vault's astro is edited, after the look.
    tree.write(
        "projects/p/.agents/skills/coding/extra.md",
        "extra, edited again",
    );
    tree.write("vault/web/astro/SKILL.md", "astro edited in the vault");

    let report = promote(&ws.vault, &plan, &dirs(&tree)).unwrap();

    assert_eq!(report.failed(), 2);
    assert!(matches!(
        report.skills[0].result,
        Err(PromoteError::SourceChanged { .. })
    ));
    assert!(matches!(
        report.skills[1].result,
        Err(PromoteError::VaultChanged { .. })
    ));
    assert_eq!(tree.read("vault/coding/SKILL.md"), "coding v1");
    assert_eq!(
        tree.read("vault/web/astro/SKILL.md"),
        "astro edited in the vault"
    );
    assert_eq!(report.backup.stored, 0);
    assert!(stage_folders(&tree, "vault").is_empty());
    assert!(stage_folders(&tree, "vault/web").is_empty());
}
