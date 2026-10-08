//! `undo`: what comes back, what is refused, and that undoing twice is a redo.

use std::os::unix::fs::PermissionsExt;

use super::tests::{Rig, files_of, no_leftovers};
use super::*;
use crate::events::ignore_events;

fn undo_run(rig: &Rig, id: Option<&str>, filter: &Filter, dry_run: bool) -> UndoReport {
    undo(&rig.backups, id, filter, dry_run, &ignore_events).unwrap()
}

fn undo_all(rig: &Rig) -> UndoReport {
    undo_run(rig, None, &Filter::default(), false)
}

fn project_filter(rig: &Rig, project: &str) -> Filter {
    Filter {
        project: Some(rig.f.tree.path().join(project)),
        skill: None,
    }
}

/// A project whose `coding` copy is old, has an executable and a local note.
fn old_copy(rig: &Rig, project: &str) -> crate::scan::Target {
    let target = rig.target(project, "coding");
    rig.seed(project, "coding", "SKILL.md", "old");
    rig.seed(project, "coding", "run.sh", "#!old\n");
    rig.seed(project, "coding", "my_notes.md", "mine");
    std::fs::set_permissions(
        target.path.join("run.sh"),
        std::fs::Permissions::from_mode(0o750),
    )
    .unwrap();
    target
}

#[test]
fn undoing_an_update_brings_back_the_old_bytes_and_modes() {
    let rig = Rig::new(&[("coding/SKILL.md", "new")]);
    let target = old_copy(&rig, "proj");
    let before = files_of(&target.path);
    let (_, saved) = rig.apply(RunKind::Sync, vec![target.clone()]);

    let report = undo_all(&rig);

    assert_eq!(report.from, saved.id);
    assert!(matches!(report.entries[0].result, Ok(Undone::Restored)));
    assert_eq!(files_of(&target.path), before);
    assert!(
        !rig.backups.root().join(&saved.id).exists(),
        "spent run kept"
    );
    no_leftovers(&target.project);
}

#[test]
fn undoing_twice_is_a_redo() {
    let rig = Rig::new(&[("coding/SKILL.md", "new")]);
    let target = old_copy(&rig, "proj");
    rig.apply(RunKind::Sync, vec![target.clone()]);
    let synced = files_of(&target.path);
    let first = undo_all(&rig);
    let saved = first
        .saved
        .as_ref()
        .expect("the undo run saved the synced copy");
    assert_ne!(saved.id, first.from);

    let second = undo_all(&rig);

    assert_eq!(second.from, saved.id);
    assert_eq!(files_of(&target.path), synced);
    no_leftovers(&target.project);
}

#[test]
fn undoing_a_create_removes_the_skill_and_a_redo_brings_it_back() {
    let rig = Rig::new(&[("coding/SKILL.md", "new"), ("coding/lang/js.md", "js")]);
    let target = rig.target("proj", "coding");
    rig.apply(RunKind::Push, vec![target.clone()]);
    let created = files_of(&target.path);

    let report = undo_all(&rig);

    assert!(matches!(report.entries[0].result, Ok(Undone::Removed)));
    assert!(!target.path.exists());
    assert!(target.path.parent().unwrap().is_dir(), "skills/ stays");
    undo_all(&rig);
    assert_eq!(files_of(&target.path), created);
    no_leftovers(&target.project);
}

#[test]
fn a_created_skill_that_is_gone_is_not_an_error() {
    let rig = Rig::new(&[("coding/SKILL.md", "new")]);
    let target = rig.target("proj", "coding");
    rig.apply(RunKind::Push, vec![target.clone()]);
    std::fs::remove_dir_all(&target.path).unwrap();

    let report = undo_all(&rig);

    assert!(matches!(report.entries[0].result, Ok(Undone::AlreadyGone)));
    assert!(report.saved.is_none());
    assert_eq!(report.failed(), 0);
}

#[test]
fn a_filter_undoes_only_the_chosen_project_and_keeps_the_rest_of_the_run() {
    let rig = Rig::new(&[("coding/SKILL.md", "new")]);
    let (one, two) = (old_copy(&rig, "p1"), old_copy(&rig, "p2"));
    let before = files_of(&one.path);
    let (_, saved) = rig.apply(RunKind::Sync, vec![one.clone(), two.clone()]);

    let report = undo_run(&rig, Some(&saved.id), &project_filter(&rig, "p1"), false);

    assert_eq!(report.entries.len(), 1);
    assert_eq!(files_of(&one.path), before);
    assert_eq!(files_of(&two.path)["SKILL.md"].0, "new");
    let left = rig.backups.load(&saved.id).unwrap();
    assert_eq!(left.entries.len(), 1);
    assert_eq!(left.entries[0].entry.project, two.project);
}

#[test]
fn a_filter_by_skill_leaves_the_other_skills_alone() {
    let rig = Rig::new(&[("coding/SKILL.md", "new"), ("web/SKILL.md", "new")]);
    let (coding, web) = (old_copy(&rig, "proj"), rig.target("proj", "web"));
    rig.seed("proj", "web", "SKILL.md", "old web");
    rig.apply(RunKind::Sync, vec![coding.clone(), web.clone()]);
    let filter = Filter {
        project: None,
        skill: Some("web".to_string()),
    };

    let report = undo_run(&rig, None, &filter, false);

    assert_eq!(report.entries.len(), 1);
    assert_eq!(files_of(&web.path)["SKILL.md"].0, "old web");
    assert_eq!(files_of(&coding.path)["SKILL.md"].0, "new");
}

#[test]
fn a_dry_run_reports_but_changes_nothing() {
    let rig = Rig::new(&[("coding/SKILL.md", "new")]);
    let target = old_copy(&rig, "proj");
    let (_, saved) = rig.apply(RunKind::Sync, vec![target.clone()]);
    let project_before = files_of(&target.path);
    let store_before = files_of(rig.backups.root());

    let report = undo_run(&rig, None, &Filter::default(), true);

    assert!(matches!(report.entries[0].result, Ok(Undone::Restored)));
    assert!(report.saved.is_none());
    assert_eq!(files_of(&target.path), project_before);
    assert_eq!(files_of(rig.backups.root()), store_before);
    assert_eq!(rig.backups.run_ids().unwrap(), [saved.id]);
}

#[test]
fn a_project_that_is_gone_fails_alone_and_its_backup_stays() {
    let rig = Rig::new(&[("coding/SKILL.md", "new")]);
    let (gone, kept) = (old_copy(&rig, "p1"), old_copy(&rig, "p2"));
    let before = files_of(&kept.path);
    let (_, saved) = rig.apply(RunKind::Sync, vec![gone.clone(), kept.clone()]);
    std::fs::remove_dir_all(&gone.project).unwrap();

    let report = undo_run(&rig, Some(&saved.id), &Filter::default(), false);

    assert!(matches!(
        &report.entries[0].result,
        Err(UndoError::Backup(BackupError::ProjectGone { .. }))
    ));
    assert!(matches!(report.entries[1].result, Ok(Undone::Restored)));
    assert_eq!(files_of(&kept.path), before);
    assert_eq!(rig.backups.load(&saved.id).unwrap().entries.len(), 1);
}

#[test]
fn a_symlinked_agents_folder_is_never_written_through() {
    let rig = Rig::new(&[("coding/SKILL.md", "new")]);
    let target = old_copy(&rig, "proj");
    rig.apply(RunKind::Sync, vec![target.clone()]);
    let elsewhere = rig.f.tree.path().join("elsewhere");
    std::fs::create_dir(&elsewhere).unwrap();
    std::fs::rename(target.project.join(".agents"), elsewhere.join(".agents")).unwrap();
    std::os::unix::fs::symlink(elsewhere.join(".agents"), target.project.join(".agents")).unwrap();
    let behind = files_of(&elsewhere.join(".agents"));

    let report = undo_all(&rig);

    assert!(matches!(
        &report.entries[0].result,
        Err(UndoError::Backup(BackupError::LinkedParent { .. }))
    ));
    assert_eq!(files_of(&elsewhere.join(".agents")), behind);
}

#[test]
fn a_lost_tree_is_reported_and_the_project_is_untouched() {
    let rig = Rig::new(&[("coding/SKILL.md", "new")]);
    let target = old_copy(&rig, "proj");
    let (_, saved) = rig.apply(RunKind::Sync, vec![target.clone()]);
    let synced = files_of(&target.path);
    std::fs::remove_dir_all(rig.backups.root().join(&saved.id).join("0/tree")).unwrap();

    let report = undo_all(&rig);

    assert!(matches!(
        &report.entries[0].result,
        Err(UndoError::Backup(BackupError::BadEntry { .. }))
    ));
    assert_eq!(files_of(&target.path), synced);
}

#[test]
fn a_skill_that_became_a_file_is_refused() {
    let rig = Rig::new(&[("coding/SKILL.md", "new")]);
    let target = old_copy(&rig, "proj");
    rig.apply(RunKind::Sync, vec![target.clone()]);
    std::fs::remove_dir_all(&target.path).unwrap();
    std::fs::write(&target.path, "a file now").unwrap();

    let report = undo_all(&rig);

    assert!(matches!(
        &report.entries[0].result,
        Err(UndoError::Backup(BackupError::NotAFolder { .. }))
    ));
    assert_eq!(std::fs::read_to_string(&target.path).unwrap(), "a file now");
}

#[test]
fn a_run_id_that_leaves_the_store_is_refused() {
    let rig = Rig::new(&[("coding/SKILL.md", "new")]);
    let target = old_copy(&rig, "proj");
    rig.apply(RunKind::Sync, vec![target]);
    std::fs::create_dir_all(rig.f.tree.path().join("state/outside/0")).unwrap();

    for id in ["../outside", "nope", ""] {
        let err = undo(
            &rig.backups,
            Some(id),
            &Filter::default(),
            false,
            &ignore_events,
        )
        .unwrap_err();
        assert!(
            matches!(err, BackupError::NoSuchRun { .. }),
            "{id:?}: {err}"
        );
    }
}

#[test]
fn nothing_to_undo_is_an_error() {
    let rig = Rig::new(&[("coding/SKILL.md", "new")]);

    let err = undo(
        &rig.backups,
        None,
        &Filter::default(),
        false,
        &ignore_events,
    )
    .unwrap_err();

    assert!(matches!(err, BackupError::Empty));
}

#[test]
fn a_poisoned_note_cannot_reach_outside_the_skills_folder() {
    let rig = Rig::new(&[("coding/SKILL.md", "new")]);
    let target = old_copy(&rig, "proj");
    let (_, saved) = rig.apply(RunKind::Sync, vec![target.clone()]);
    let note = rig.backups.root().join(&saved.id).join("0/entry.json");
    let text = std::fs::read_to_string(&note).unwrap();
    std::fs::write(&note, text.replace("\"coding\"", "\"..\"")).unwrap();
    let synced = files_of(&target.project);

    let report = undo_all(&rig);

    assert!(matches!(
        &report.entries[0].result,
        Err(UndoError::Backup(BackupError::BadEntry { .. }))
    ));
    assert_eq!(files_of(&target.project), synced);
}
