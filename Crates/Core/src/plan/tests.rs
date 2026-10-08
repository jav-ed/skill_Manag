use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use super::*;
use crate::Hint;
use crate::testutil::Fixture;

fn one(f: &Fixture, skill: &str) -> Result<SkillPlan, PlanError> {
    let mut plan = f.plan(vec![f.target("proj", skill)]);
    plan.entries.remove(0).result
}

fn rels(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect()
}

fn changes(plan: &SkillPlan) -> Vec<(String, ChangeKind)> {
    plan.changes
        .iter()
        .map(|c| (c.rel.to_string_lossy().into_owned(), c.kind))
        .collect()
}

#[test]
fn missing_destination_is_a_create_with_every_file_added() {
    let f = Fixture::new(&[("coding/SKILL.md", "a"), ("coding/lang/js.md", "b")]);
    let plan = one(&f, "coding").unwrap();
    assert_eq!(plan.kind, PlanKind::Create);
    assert_eq!(plan.file_count(), 2);
    assert!(plan.changes.iter().all(|c| c.kind == ChangeKind::Added));
    assert!(plan.removed.is_empty());
}

#[test]
fn identical_destination_is_unchanged() {
    let f = Fixture::new(&[("coding/SKILL.md", "a"), ("coding/lang/js.md", "b")]);
    let target = f.target("proj", "coding");
    f.tree.write("proj/.agents/skills/coding/SKILL.md", "a");
    f.tree.write("proj/.agents/skills/coding/lang/js.md", "b");
    for rel in ["SKILL.md", "lang/js.md"] {
        let src = std::fs::metadata(f.vault.path.join("coding").join(rel))
            .unwrap()
            .permissions()
            .mode();
        std::fs::set_permissions(
            target.path.join(rel),
            std::fs::Permissions::from_mode(src & 0o777),
        )
        .unwrap();
    }
    let plan = f.plan(vec![target]).entries.remove(0).result.unwrap();
    assert_eq!(
        plan.kind,
        PlanKind::Unchanged,
        "{:?} {:?}",
        plan.changes,
        plan.removed
    );
}

#[test]
fn modified_missing_extra_and_mode_changes_are_all_found() {
    let f = Fixture::new(&[
        ("coding/SKILL.md", "new text"),
        ("coding/keep.md", "same"),
        ("coding/added.md", "x"),
        ("coding/run.sh", "#!/bin/sh\n"),
    ]);
    std::fs::set_permissions(
        f.vault.path.join("coding/run.sh"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    f.tree
        .write("proj/.agents/skills/coding/SKILL.md", "old text!");
    f.tree.write("proj/.agents/skills/coding/keep.md", "same");
    f.tree
        .write("proj/.agents/skills/coding/run.sh", "#!/bin/sh\n");
    f.tree.write("proj/.agents/skills/coding/stale.md", "gone");
    f.tree
        .write("proj/.agents/skills/coding/old_dir/deep.md", "gone");
    std::fs::create_dir_all(f.tree.path().join("proj/.agents/skills/coding/empty_dir")).unwrap();
    let keep = f.tree.path().join("proj/.agents/skills/coding/keep.md");
    std::fs::set_permissions(
        &keep,
        std::fs::Permissions::from_mode(
            f.vault
                .path
                .join("coding/keep.md")
                .metadata()
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
        ),
    )
    .unwrap();
    std::fs::set_permissions(
        f.tree.path().join("proj/.agents/skills/coding/run.sh"),
        std::fs::Permissions::from_mode(0o644),
    )
    .unwrap();

    let plan = one(&f, "coding").unwrap();
    assert_eq!(plan.kind, PlanKind::Update);
    assert_eq!(
        changes(&plan),
        [
            ("SKILL.md".to_string(), ChangeKind::Modified),
            ("added.md".to_string(), ChangeKind::Added),
            ("run.sh".to_string(), ChangeKind::ModeChanged),
        ]
    );
    assert_eq!(
        rels(&plan.removed),
        ["empty_dir", "old_dir", "old_dir/deep.md", "stale.md"]
    );
}

#[test]
fn same_length_different_bytes_is_modified() {
    let f = Fixture::new(&[("coding/SKILL.md", "aaaa")]);
    f.tree.write("proj/.agents/skills/coding/SKILL.md", "aaab");
    assert_eq!(
        changes(&one(&f, "coding").unwrap()),
        [("SKILL.md".to_string(), ChangeKind::Modified)]
    );
}

#[test]
fn skill_missing_from_the_vault_is_never_planned() {
    let f = Fixture::new(&[("coding/SKILL.md", "a")]);
    f.tree
        .write("proj/.agents/skills/ghost/SKILL.md", "precious");
    assert!(matches!(
        one(&f, "ghost"),
        Err(PlanError::NotInVault { .. })
    ));
}

#[test]
fn skill_without_tracked_files_is_a_hard_error() {
    let f = Fixture::new(&[("coding/SKILL.md", "a")]);
    f.tree.write("vault/fresh/SKILL.md", "untracked");
    let f = f.refresh();
    assert!(matches!(
        one(&f, "fresh"),
        Err(PlanError::NoTrackedFiles { .. })
    ));
}

#[test]
fn an_untracked_skill_file_next_to_tracked_files_is_a_hard_error_and_the_copy_stays() {
    let f = Fixture::new(&[("coding/SKILL.md", "a"), ("sk/ref.md", "r")]);
    f.tree.write("vault/sk/SKILL.md", "new and uncommitted");
    f.tree
        .write("project/.agents/skills/sk/SKILL.md", "working");
    let f = f.refresh();
    let err = one(&f, "sk").unwrap_err();
    assert!(
        matches!(err, PlanError::SkillFileNotTracked { .. }),
        "{err}"
    );
    assert!(err.hint().unwrap().contains("git add sk/SKILL.md"));
}

#[test]
fn tracked_symlink_and_missing_working_file_are_hard_errors() {
    let f = Fixture::new(&[
        ("coding/SKILL.md", "a"),
        ("other/SKILL.md", "a"),
        ("other/x.md", "a"),
    ]);
    std::os::unix::fs::symlink("SKILL.md", f.vault.path.join("coding/link.md")).unwrap();
    f.tree.git("vault", &["add", "-A"]);
    f.tree.git("vault", &["commit", "-q", "-m", "link"]);
    std::fs::remove_file(f.vault.path.join("other/x.md")).unwrap();
    let f = f.refresh();
    assert!(matches!(
        one(&f, "coding"),
        Err(PlanError::UnsupportedEntry { .. })
    ));
    assert!(matches!(
        one(&f, "other"),
        Err(PlanError::MissingSourceFile { .. })
    ));
}

#[test]
fn destination_that_is_a_symlink_or_a_file_is_refused() {
    let f = Fixture::new(&[("coding/SKILL.md", "a"), ("web/SKILL.md", "a")]);
    let t = f.target("proj", "coding");
    f.tree.write("elsewhere/x", "x");
    std::os::unix::fs::symlink(f.tree.path().join("elsewhere"), &t.path).unwrap();
    assert!(matches!(
        one(&f, "coding"),
        Err(PlanError::DestinationIsSymlink { .. })
    ));
    f.tree.write("proj/.agents/skills/web", "i am a file");
    assert!(matches!(
        one(&f, "web"),
        Err(PlanError::DestinationNotADirectory { .. })
    ));
}

#[test]
fn create_needs_an_existing_skills_directory() {
    let f = Fixture::new(&[("coding/SKILL.md", "a")]);
    let mut target = f.target("proj", "coding");
    target.path = f.tree.path().join("noproj/.agents/skills/coding");
    assert!(matches!(
        f.plan(vec![target]).entries.remove(0).result,
        Err(PlanError::SkillsDirMissing { .. })
    ));
}

#[test]
fn counts_summarise_a_plan() {
    let f = Fixture::new(&[("coding/SKILL.md", "a"), ("web/SKILL.md", "a")]);
    f.tree.write("proj/.agents/skills/web/SKILL.md", "b");
    let targets = vec![
        f.target("proj", "coding"),
        f.target("proj", "web"),
        f.target("proj", "ghost"),
    ];
    let counts = f.plan(targets).counts();
    assert_eq!(
        counts,
        PlanCounts {
            create: 1,
            update: 1,
            unchanged: 0,
            failed: 1
        }
    );
}

#[test]
fn planning_through_a_symlinked_agents_directory_is_refused_in_every_case() {
    let f = Fixture::new(&[("coding/SKILL.md", "a")]);
    f.tree.write("shared/skills/coding/SKILL.md", "a");
    std::fs::create_dir_all(f.tree.path().join("proj")).unwrap();
    std::os::unix::fs::symlink(
        f.tree.path().join("shared"),
        f.tree.path().join("proj/.agents"),
    )
    .unwrap();
    let target = crate::scan::Target {
        project: f.tree.path().join("proj"),
        skill: "coding".into(),
        path: f.tree.path().join("proj/.agents/skills/coding"),
    };
    let err = f.plan(vec![target]).entries.remove(0).result.unwrap_err();
    assert!(
        matches!(err, PlanError::DestinationIsSymlink { .. }),
        "{err}"
    );
}
