//! `delete` with a backup run: the folder is kept, and a failed backup never costs the user the folder.

use std::os::unix::fs::PermissionsExt;

use super::tests::{Rig, files_of, no_leftovers};
use super::*;
use crate::events::ignore_events;
use crate::ops::{DeleteError, delete};
use crate::scan::Target;

fn rig() -> Rig {
    Rig::new(&[("coding/SKILL.md", "vault")])
}

/// A skill folder a project already has, with a nested file, an executable and a local note.
fn installed(rig: &Rig, project: &str, skill: &str) -> Target {
    let target = rig.target(project, skill);
    rig.seed(project, skill, "SKILL.md", "mine");
    rig.seed(project, skill, "lang/js.md", "js");
    rig.seed(project, skill, "run.sh", "#!local\n");
    std::fs::set_permissions(
        target.path.join("run.sh"),
        std::fs::Permissions::from_mode(0o750),
    )
    .unwrap();
    target
}

fn delete_all(rig: &Rig, targets: Vec<Target>) -> (crate::ops::DeleteReport, Finished) {
    let run = rig.backups.begin(RunKind::Delete).unwrap();
    let report = delete(targets, false, Some(&run), &ignore_events);
    (report, run.finish(&rig.backups))
}

#[test]
fn a_deleted_folder_is_kept_and_undo_brings_it_back() {
    let rig = rig();
    let target = installed(&rig, "proj", "coding");
    let before = files_of(&target.path);

    let (report, saved) = delete_all(&rig, vec![target.clone()]);

    assert_eq!(report.failed(), 0);
    assert!(!target.path.exists());
    no_leftovers(&target.project);
    let run = rig.backups.load(&saved.id).unwrap();
    assert_eq!(run.entries[0].entry.change, Change::Deleted);
    assert_eq!(run.entries[0].entry.kind, RunKind::Delete);
    assert_eq!(files_of(&run.entries[0].tree()), before);

    let undone = undo(
        &rig.backups,
        None,
        &Filter::default(),
        false,
        &ignore_events,
    )
    .unwrap();

    assert!(matches!(undone.entries[0].result, Ok(Undone::Restored)));
    assert_eq!(files_of(&target.path), before);
    no_leftovers(&target.project);
}

#[test]
fn every_deleted_folder_gets_its_own_slot() {
    let rig = rig();
    let targets: Vec<_> = (0..3)
        .map(|n| installed(&rig, &format!("p{n}"), "coding"))
        .collect();
    let before: Vec<_> = targets.iter().map(|t| files_of(&t.path)).collect();

    let (report, saved) = delete_all(&rig, targets.clone());

    assert_eq!(report.failed(), 0);
    assert_eq!(saved.stored, 3);
    undo(
        &rig.backups,
        None,
        &Filter::default(),
        false,
        &ignore_events,
    )
    .unwrap();
    for (target, expected) in targets.iter().zip(&before) {
        assert_eq!(&files_of(&target.path), expected);
    }
}

#[test]
fn a_delete_whose_backup_fails_puts_the_folder_back() {
    let rig = rig();
    let target = installed(&rig, "proj", "coding");
    let before = files_of(&target.path);
    std::fs::create_dir_all(rig.f.tree.path().join("state")).unwrap();
    std::fs::write(rig.backups.root(), "in the way").unwrap();

    let (report, saved) = delete_all(&rig, vec![target.clone()]);

    assert!(
        matches!(report.deleted[0].result, Err(DeleteError::Backup(_))),
        "{:?}",
        report.deleted[0].result
    );
    assert_eq!(saved.stored, 0);
    assert_eq!(files_of(&target.path), before);
    no_leftovers(&target.project);
}

#[test]
fn a_symlinked_skill_folder_loses_only_its_link_and_stores_nothing() {
    let rig = rig();
    let target = rig.target("proj", "coding");
    let behind = rig.f.tree.path().join("shared/coding");
    std::fs::create_dir_all(&behind).unwrap();
    std::fs::write(behind.join("SKILL.md"), "shared").unwrap();
    std::os::unix::fs::symlink(&behind, &target.path).unwrap();

    let (report, saved) = delete_all(&rig, vec![target.clone()]);

    assert_eq!(report.failed(), 0);
    assert!(
        std::fs::symlink_metadata(&target.path).is_err(),
        "link kept"
    );
    assert_eq!(
        std::fs::read_to_string(behind.join("SKILL.md")).unwrap(),
        "shared"
    );
    assert_eq!(saved.stored, 0);
    assert!(!rig.backups.root().exists());
}

#[test]
fn a_dry_run_stores_nothing_and_removes_nothing() {
    let rig = rig();
    let target = installed(&rig, "proj", "coding");
    let before = files_of(&target.path);
    let run = rig.backups.begin(RunKind::Delete).unwrap();

    let report = delete(vec![target.clone()], true, Some(&run), &ignore_events);

    assert_eq!(report.failed(), 0);
    assert_eq!(run.stored(), 0);
    assert_eq!(files_of(&target.path), before);
    assert!(!rig.backups.root().exists());
}

#[test]
fn undoing_a_delete_over_a_recreated_skill_saves_the_recreated_copy() {
    let rig = rig();
    let target = installed(&rig, "proj", "coding");
    let original = files_of(&target.path);
    delete_all(&rig, vec![target.clone()]);
    rig.seed("proj", "coding", "SKILL.md", "written after the delete");
    let recreated = files_of(&target.path);

    let first = undo(
        &rig.backups,
        None,
        &Filter::default(),
        false,
        &ignore_events,
    )
    .unwrap();

    assert!(matches!(first.entries[0].result, Ok(Undone::Restored)));
    assert_eq!(files_of(&target.path), original);
    undo(
        &rig.backups,
        None,
        &Filter::default(),
        false,
        &ignore_events,
    )
    .unwrap();
    assert_eq!(
        files_of(&target.path),
        recreated,
        "the redo returns the newer copy"
    );
}
