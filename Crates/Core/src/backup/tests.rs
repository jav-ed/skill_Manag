//! The store on its own, and the setup the other backup tests share.

use std::collections::BTreeMap;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use super::*;
use crate::apply::{ApplyOptions, ApplyReport, apply};
use crate::events::ignore_events;
use crate::scan::Target;
use crate::testutil::Fixture;

/// A vault, a tree of projects and a backup store beside them.
pub(super) struct Rig {
    pub(super) f: Fixture,
    pub(super) backups: Backups,
}

impl Rig {
    pub(super) fn new(files: &[(&str, &str)]) -> Self {
        let f = Fixture::new(files);
        let backups = Backups::at(f.tree.path().join("state/backups"));
        Self { f, backups }
    }

    pub(super) fn target(&self, project: &str, skill: &str) -> Target {
        self.f.target(project, skill)
    }

    /// Puts a file into the copy of `skill` a project already has, so a sync has something to replace.
    pub(super) fn seed(&self, project: &str, skill: &str, file: &str, content: &str) {
        self.f
            .tree
            .write(&format!("{project}/.agents/skills/{skill}/{file}"), content);
    }

    /// Applies `targets` as one run of `kind` with backups on, and ends the run.
    pub(super) fn apply(&self, kind: RunKind, targets: Vec<Target>) -> (ApplyReport, Finished) {
        let run = self.backups.begin(kind).unwrap();
        let options = ApplyOptions {
            backup: Some(&run),
            ..ApplyOptions::default()
        };
        let report = apply(self.f.plan(targets), options, &ignore_events).unwrap();
        (report, run.finish(&self.backups))
    }
}

/// Every file below `dir`: relative path to (content, permission bits).
pub(super) fn files_of(dir: &Path) -> BTreeMap<String, (String, u32)> {
    fn visit(base: &Path, rel: &Path, out: &mut BTreeMap<String, (String, u32)>) {
        for entry in std::fs::read_dir(base.join(rel)).unwrap() {
            let entry = entry.unwrap();
            let rel = rel.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                visit(base, &rel, out);
            } else {
                let mode = entry.metadata().unwrap().permissions().mode() & 0o777;
                let content = std::fs::read_to_string(entry.path()).unwrap();
                out.insert(rel.to_string_lossy().into_owned(), (content, mode));
            }
        }
    }
    let mut out = BTreeMap::new();
    visit(dir, Path::new(""), &mut out);
    out
}

/// A project's `.agents` folder holds nothing but `skills`: no stage or trash folder was left.
pub(super) fn no_leftovers(project: &Path) {
    let names: Vec<_> = std::fs::read_dir(project.join(".agents"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["skills"], "leftover entries in .agents");
}

fn make_run_dir(backups: &Backups, id: &str) {
    std::fs::create_dir_all(backups.root().join(id)).unwrap();
}

#[test]
fn a_missing_store_is_an_empty_list() {
    let rig = Rig::new(&[("a/SKILL.md", "a")]);
    assert!(rig.backups.run_ids().unwrap().is_empty());
    assert!(rig.backups.runs().unwrap().is_empty());
    assert!(matches!(rig.backups.latest(), Err(BackupError::Empty)));
}

#[test]
fn run_ids_come_newest_first_and_skip_plain_files() {
    let rig = Rig::new(&[("a/SKILL.md", "a")]);
    for id in [
        "20260101-000000-000-1-0",
        "20261231-235959-999-1-0",
        "20260601-120000-000-1-0",
    ] {
        make_run_dir(&rig.backups, id);
    }
    std::fs::write(rig.backups.root().join("stray.txt"), "x").unwrap();

    assert_eq!(
        rig.backups.run_ids().unwrap(),
        [
            "20261231-235959-999-1-0",
            "20260601-120000-000-1-0",
            "20260101-000000-000-1-0"
        ]
    );
}

#[test]
fn a_run_id_from_the_user_must_name_a_folder_of_the_store() {
    let rig = Rig::new(&[("a/SKILL.md", "a")]);
    make_run_dir(&rig.backups, "20260101-000000-000-1-0");
    // A real folder next to the store: `../outside` would find it if the id were not checked.
    std::fs::create_dir_all(rig.f.tree.path().join("state/outside")).unwrap();

    for id in ["../outside", "", ".", "a/b", "missing", "nul\0"] {
        let err = rig.backups.load(id).unwrap_err();
        assert!(
            matches!(err, BackupError::NoSuchRun { .. }),
            "{id:?}: {err}"
        );
    }
    assert!(rig.backups.load("20260101-000000-000-1-0").is_ok());
}

#[test]
fn prune_keeps_the_newest_runs() {
    let rig = Rig::new(&[("a/SKILL.md", "a")]);
    for n in 1..=5 {
        make_run_dir(&rig.backups, &format!("2026010{n}-000000-000-1-0"));
    }

    assert_eq!(rig.backups.prune(2).unwrap(), 3);

    assert_eq!(
        rig.backups.run_ids().unwrap(),
        ["20260105-000000-000-1-0", "20260104-000000-000-1-0"]
    );
    assert_eq!(rig.backups.prune(2).unwrap(), 0);
}

#[test]
fn finishing_prunes_only_after_a_run_that_stored_something() {
    let rig = Rig::new(&[("coding/SKILL.md", "new")]);
    for n in 0..KEEP_RUNS + 3 {
        make_run_dir(&rig.backups, &format!("20200101-000000-{n:03}-1-0"));
    }

    let idle = rig
        .backups
        .begin(RunKind::Sync)
        .unwrap()
        .finish(&rig.backups);
    assert_eq!(idle.stored, 0);
    assert_eq!(rig.backups.run_ids().unwrap().len(), KEEP_RUNS + 3);

    rig.seed("proj", "coding", "SKILL.md", "old");
    let (_, busy) = rig.apply(RunKind::Sync, vec![rig.target("proj", "coding")]);
    assert_eq!(busy.stored, 1);
    assert!(busy.prune_error.is_none());
    let ids = rig.backups.run_ids().unwrap();
    assert_eq!(ids.len(), KEEP_RUNS);
    assert_eq!(ids[0], busy.id);
    assert!(!ids.contains(&"20200101-000000-000-1-0".to_string()));
}

#[test]
fn latest_skips_runs_without_entries() {
    let rig = Rig::new(&[("coding/SKILL.md", "new")]);
    rig.seed("proj", "coding", "SKILL.md", "old");
    let (_, saved) = rig.apply(RunKind::Push, vec![rig.target("proj", "coding")]);
    make_run_dir(&rig.backups, "29990101-000000-000-1-0");

    let latest = rig.backups.latest().unwrap();

    assert_eq!(latest.id, saved.id);
    assert_eq!(latest.kind(), Some(RunKind::Push));
}

#[test]
fn entries_load_in_slot_order_not_in_text_order() {
    let rig = Rig::new(&[("coding/SKILL.md", "new")]);
    let targets: Vec<_> = (0..12)
        .map(|n| rig.target(&format!("p{n}"), "coding"))
        .collect();
    let (_, saved) = rig.apply(RunKind::Add, targets);

    let run = rig.backups.load(&saved.id).unwrap();

    let indices: Vec<_> = run.entries.iter().map(|e| e.index).collect();
    assert_eq!(indices, (0..12).collect::<Vec<_>>());
    assert_eq!(run.entries[10].entry.project, rig.f.tree.path().join("p10"));
}

#[test]
fn a_damaged_note_is_reported_with_its_path() {
    let rig = Rig::new(&[("coding/SKILL.md", "new")]);
    rig.seed("proj", "coding", "SKILL.md", "old");
    let (_, saved) = rig.apply(RunKind::Sync, vec![rig.target("proj", "coding")]);
    let note = rig.backups.root().join(&saved.id).join("0/entry.json");
    std::fs::write(&note, "not json").unwrap();

    let err = rig.backups.load(&saved.id).unwrap_err();

    assert!(
        matches!(&err, BackupError::BadEntry { path, .. } if *path == note),
        "{err}"
    );
}
