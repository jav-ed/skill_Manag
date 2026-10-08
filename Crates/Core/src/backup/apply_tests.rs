//! `apply` with a backup run: what is stored, and that a failed backup never costs the user a file.

use std::os::unix::fs::PermissionsExt;

use super::tests::{Rig, files_of, no_leftovers};
use super::*;
use crate::apply::{ApplyError, ApplyOptions, Failure, Outcome, apply};
use crate::events::ignore_events;

#[test]
fn an_update_stores_the_old_folder_with_its_modes() {
    let rig = Rig::new(&[("coding/SKILL.md", "new")]);
    let target = rig.target("proj", "coding");
    rig.seed("proj", "coding", "SKILL.md", "old");
    rig.seed("proj", "coding", "run.sh", "#!old\n");
    rig.seed("proj", "coding", "my_notes.md", "mine");
    std::fs::set_permissions(
        target.path.join("run.sh"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    let before = files_of(&target.path);

    let (report, saved) = rig.apply(RunKind::Sync, vec![target.clone()]);

    assert!(matches!(report.applied[0].outcome, Outcome::Updated));
    assert_eq!(saved.stored, 1);
    let run = rig.backups.load(&saved.id).unwrap();
    assert_eq!(
        run.entries[0].entry,
        Entry {
            kind: RunKind::Sync,
            project: target.project.clone(),
            skill: "coding".to_string(),
            change: Change::Updated,
        }
    );
    assert_eq!(files_of(&run.entries[0].tree()), before);
    assert_eq!(
        files_of(&target.path).keys().collect::<Vec<_>>(),
        ["SKILL.md"]
    );
    no_leftovers(&target.project);
}

#[test]
fn a_create_is_noted_so_undo_can_remove_it() {
    let rig = Rig::new(&[("coding/SKILL.md", "new")]);
    let target = rig.target("proj", "coding");

    let (report, saved) = rig.apply(RunKind::Push, vec![target.clone()]);

    assert!(matches!(report.applied[0].outcome, Outcome::Created));
    let run = rig.backups.load(&saved.id).unwrap();
    assert_eq!(run.entries[0].entry.change, Change::Created);
    assert_eq!(run.entries[0].entry.kind, RunKind::Push);
    assert!(!run.entries[0].has_tree);
    no_leftovers(&target.project);
}

#[test]
fn an_unchanged_skill_stores_nothing_and_leaves_no_run_folder() {
    let rig = Rig::new(&[("coding/SKILL.md", "new")]);
    let target = rig.target("proj", "coding");
    rig.seed("proj", "coding", "SKILL.md", "new");

    let (report, saved) = rig.apply(RunKind::Sync, vec![target]);

    assert!(matches!(report.applied[0].outcome, Outcome::Unchanged));
    assert_eq!(saved.stored, 0);
    assert!(!rig.backups.root().exists());
}

#[test]
fn a_mixed_run_counts_only_what_it_changed() {
    let rig = Rig::new(&[("coding/SKILL.md", "new"), ("web/SKILL.md", "new")]);
    rig.seed("old", "coding", "SKILL.md", "stale");
    rig.seed("same", "coding", "SKILL.md", "new");
    let targets = vec![
        rig.target("old", "coding"),
        rig.target("same", "coding"),
        rig.target("fresh", "web"),
    ];

    let (report, saved) = rig.apply(RunKind::Sync, targets);

    assert_eq!(report.failed(), 0);
    assert_eq!(saved.stored, 2);
    assert_eq!(rig.backups.load(&saved.id).unwrap().entries.len(), 2);
}

#[test]
fn without_a_backup_the_old_copy_is_just_removed() {
    let rig = Rig::new(&[("coding/SKILL.md", "new")]);
    let target = rig.target("proj", "coding");
    rig.seed("proj", "coding", "SKILL.md", "old");

    let report = apply(
        rig.f.plan(vec![target.clone()]),
        ApplyOptions::default(),
        &ignore_events,
    )
    .unwrap();

    assert!(matches!(report.applied[0].outcome, Outcome::Updated));
    assert!(!rig.backups.root().exists());
    no_leftovers(&target.project);
}

/// The store path is a plain file, so no run folder can be made in it.
fn block_the_store(rig: &Rig) {
    std::fs::create_dir_all(rig.f.tree.path().join("state")).unwrap();
    std::fs::write(rig.backups.root(), "in the way").unwrap();
}

#[test]
fn an_update_whose_backup_fails_is_swapped_back() {
    let rig = Rig::new(&[("coding/SKILL.md", "new")]);
    let target = rig.target("proj", "coding");
    rig.seed("proj", "coding", "SKILL.md", "old but precious");
    rig.seed("proj", "coding", "my_notes.md", "mine");
    let before = files_of(&target.path);
    block_the_store(&rig);

    let (report, saved) = rig.apply(RunKind::Sync, vec![target.clone()]);

    assert!(
        matches!(
            report.applied[0].outcome,
            Outcome::Failed(Failure::Apply(ApplyError::Backup(_)))
        ),
        "{:?}",
        report.applied[0].outcome
    );
    assert_eq!(saved.stored, 0);
    assert_eq!(files_of(&target.path), before);
    no_leftovers(&target.project);
}

#[test]
fn a_create_whose_note_fails_creates_nothing() {
    let rig = Rig::new(&[("coding/SKILL.md", "new")]);
    let target = rig.target("proj", "coding");
    block_the_store(&rig);

    let (report, _) = rig.apply(RunKind::Push, vec![target.clone()]);

    assert!(matches!(
        report.applied[0].outcome,
        Outcome::Failed(Failure::Apply(ApplyError::Backup(_)))
    ));
    assert!(!target.path.exists());
    no_leftovers(&target.project);
}

#[test]
fn one_target_that_cannot_be_stored_does_not_stop_the_others() {
    let rig = Rig::new(&[("coding/SKILL.md", "new")]);
    let (first, second) = (rig.target("p1", "coding"), rig.target("p2", "coding"));
    rig.seed("p1", "coding", "SKILL.md", "old one");
    rig.seed("p2", "coding", "SKILL.md", "old two");
    let run = rig.backups.begin(RunKind::Sync).unwrap();
    // Slot 0 is taken by a plain file, so only the first target cannot be noted.
    let run_dir = rig.backups.root().join(run.id());
    std::fs::create_dir_all(&run_dir).unwrap();
    std::fs::write(run_dir.join("0"), "in the way").unwrap();
    let options = ApplyOptions {
        backup: Some(&run),
        threads: 2,
    };

    let report = apply(
        rig.f.plan(vec![first.clone(), second.clone()]),
        options,
        &ignore_events,
    )
    .unwrap();

    assert!(matches!(report.applied[0].outcome, Outcome::Failed(_)));
    assert!(matches!(report.applied[1].outcome, Outcome::Updated));
    assert_eq!(files_of(&first.path)["SKILL.md"].0, "old one");
    assert_eq!(files_of(&second.path)["SKILL.md"].0, "new");
    assert_eq!(run.stored(), 1);
    no_leftovers(&first.project);
    no_leftovers(&second.project);
}

#[test]
fn a_project_whose_path_is_not_valid_utf8_can_still_be_synced_with_a_backup() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let rig = Rig::new(&[("coding/SKILL.md", "new")]);
    let project = rig.f.tree.path().join(OsStr::from_bytes(b"caf\xe9-app"));
    let skills = project.join(".agents/skills");
    std::fs::create_dir_all(skills.join("coding")).unwrap();
    std::fs::write(skills.join("coding/SKILL.md"), "old").unwrap();
    let target = crate::scan::Target {
        project: project.clone(),
        skill: "coding".to_string(),
        path: skills.join("coding"),
    };

    let (report, saved) = rig.apply(RunKind::Sync, vec![target.clone()]);

    assert!(
        matches!(report.applied[0].outcome, Outcome::Updated),
        "{:?}",
        report.applied[0].outcome
    );
    assert_eq!(saved.stored, 1);
    let run = rig.backups.load(&saved.id).unwrap();
    assert_eq!(
        run.entries[0].entry.project, project,
        "the path survives the note"
    );
    let undone = undo(
        &rig.backups,
        None,
        &Filter::default(),
        false,
        &ignore_events,
    )
    .unwrap();
    assert!(matches!(undone.entries[0].result, Ok(Undone::Restored)));
    assert_eq!(
        std::fs::read_to_string(target.path.join("SKILL.md")).unwrap(),
        "old"
    );
}
