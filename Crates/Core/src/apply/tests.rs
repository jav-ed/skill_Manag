use std::collections::BTreeMap;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::Mutex;

use super::*;
use crate::events::{Event, Status, ignore_events};
use crate::testutil::Fixture;

/// Every file below `dir` as relative path to (content, mode), plus directories as `None`.
fn snapshot(dir: &Path) -> BTreeMap<String, Option<(String, u32)>> {
    fn visit(base: &Path, rel: &Path, out: &mut BTreeMap<String, Option<(String, u32)>>) {
        for entry in std::fs::read_dir(base.join(rel)).unwrap() {
            let entry = entry.unwrap();
            let rel = rel.join(entry.file_name());
            let key = rel.to_string_lossy().into_owned();
            if entry.file_type().unwrap().is_dir() {
                out.insert(key, None);
                visit(base, &rel, out);
            } else {
                let mode = entry.metadata().unwrap().permissions().mode() & 0o777;
                out.insert(
                    key,
                    Some((std::fs::read_to_string(entry.path()).unwrap(), mode)),
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    visit(dir, Path::new(""), &mut out);
    out
}

fn run(f: &Fixture, targets: Vec<crate::scan::Target>) -> ApplyReport {
    apply(f.plan(targets), ApplyOptions::default(), &ignore_events).unwrap()
}

fn no_stage_left(project: &Path) {
    let names: Vec<_> = std::fs::read_dir(project.join(".agents"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["skills"], "leftover entries in .agents");
}

#[test]
fn create_writes_every_file_with_the_vault_modes() {
    let f = Fixture::new(&[
        ("coding/SKILL.md", "a"),
        ("coding/lang/js.md", "b"),
        ("coding/run.sh", "#!/bin/sh\n"),
    ]);
    std::fs::set_permissions(
        f.vault.path.join("coding/run.sh"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    let target = f.target("proj", "coding");
    let report = run(&f, vec![target.clone()]);
    assert!(
        matches!(report.applied[0].outcome, Outcome::Created),
        "{:?}",
        report.applied[0].outcome
    );
    let seen = snapshot(&target.path);
    assert_eq!(seen["SKILL.md"].as_ref().unwrap().0, "a");
    assert_eq!(seen["lang/js.md"].as_ref().unwrap().0, "b");
    assert_eq!(seen["run.sh"].as_ref().unwrap().1, 0o755);
    no_stage_left(&f.tree.path().join("proj"));
}

#[test]
fn update_makes_the_destination_an_exact_mirror() {
    let f = Fixture::new(&[("coding/SKILL.md", "new"), ("coding/added.md", "x")]);
    let target = f.target("proj", "coding");
    f.tree.write("proj/.agents/skills/coding/SKILL.md", "old");
    f.tree.write("proj/.agents/skills/coding/stale.md", "gone");
    f.tree
        .write("proj/.agents/skills/coding/old_dir/deep.md", "gone");
    std::os::unix::fs::symlink("SKILL.md", target.path.join("link")).unwrap();
    let report = run(&f, vec![target.clone()]);
    assert!(matches!(report.applied[0].outcome, Outcome::Updated));
    assert_eq!(
        snapshot(&target.path).keys().collect::<Vec<_>>(),
        ["SKILL.md", "added.md"]
    );
    assert_eq!(
        snapshot(&target.path)["SKILL.md"].as_ref().unwrap().0,
        "new"
    );
    no_stage_left(&f.tree.path().join("proj"));
}

#[test]
fn second_run_changes_nothing() {
    let f = Fixture::new(&[("coding/SKILL.md", "a"), ("coding/x/y.md", "b")]);
    let target = f.target("proj", "coding");
    run(&f, vec![target.clone()]);
    let before = std::fs::metadata(&target.path).unwrap().modified().unwrap();
    let report = run(&f, vec![target.clone()]);
    assert!(
        matches!(report.applied[0].outcome, Outcome::Unchanged),
        "{:?}",
        report.applied[0].outcome
    );
    assert_eq!(
        std::fs::metadata(&target.path).unwrap().modified().unwrap(),
        before
    );
    no_stage_left(&f.tree.path().join("proj"));
}

#[test]
fn a_copy_failure_leaves_the_destination_untouched() {
    let f = Fixture::new(&[("coding/SKILL.md", "new"), ("coding/secret.md", "x")]);
    let target = f.target("proj", "coding");
    f.tree
        .write("proj/.agents/skills/coding/SKILL.md", "old but precious");
    let plan = f.plan(vec![target.clone()]);
    std::fs::set_permissions(
        f.vault.path.join("coding/secret.md"),
        std::fs::Permissions::from_mode(0o000),
    )
    .unwrap();
    if std::fs::File::open(f.vault.path.join("coding/secret.md")).is_ok() {
        return; // running as root: permissions do not bite
    }
    let report = apply(plan, ApplyOptions::default(), &ignore_events).unwrap();
    std::fs::set_permissions(
        f.vault.path.join("coding/secret.md"),
        std::fs::Permissions::from_mode(0o644),
    )
    .unwrap();
    assert!(matches!(
        report.applied[0].outcome,
        Outcome::Failed(Failure::Apply(_))
    ));
    assert_eq!(
        snapshot(&target.path)["SKILL.md"].as_ref().unwrap().0,
        "old but precious"
    );
    no_stage_left(&f.tree.path().join("proj"));
}

#[test]
fn one_failing_target_does_not_stop_the_others() {
    let f = Fixture::new(&[("coding/SKILL.md", "a")]);
    let good = f.target("p1", "coding");
    let ghost = f.target("p2", "ghost");
    f.tree.write("p2/.agents/skills/ghost/SKILL.md", "keep me");
    let seen = Mutex::new(Vec::new());
    let observer = |event: Event| seen.lock().unwrap().push(event);
    let report = apply(
        f.plan(vec![ghost.clone(), good.clone()]),
        ApplyOptions {
            threads: 2,
            ..ApplyOptions::default()
        },
        &observer,
    )
    .unwrap();
    assert_eq!(report.failed(), 1);
    assert!(matches!(
        report.applied[0].outcome,
        Outcome::Failed(Failure::Plan(_))
    ));
    assert!(matches!(report.applied[1].outcome, Outcome::Created));
    assert_eq!(
        std::fs::read_to_string(ghost.path.join("SKILL.md")).unwrap(),
        "keep me"
    );
    let statuses: Vec<_> = seen
        .lock()
        .unwrap()
        .iter()
        .map(|e| {
            matches!(
                e,
                Event::TargetDone {
                    status: Status::Failed(_),
                    ..
                }
            )
        })
        .collect();
    assert_eq!(statuses.len(), 2);
    assert_eq!(statuses.iter().filter(|failed| **failed).count(), 1);
}

#[test]
fn non_ascii_file_names_survive_the_round_trip() {
    let f = Fixture::new(&[("coding/SKILL.md", "a"), ("coding/Grüße 日本.md", "hallo")]);
    let target = f.target("proj", "coding");
    run(&f, vec![target.clone()]);
    assert_eq!(
        std::fs::read_to_string(target.path.join("Grüße 日本.md")).unwrap(),
        "hallo"
    );
}

#[test]
fn many_targets_in_parallel_all_succeed() {
    let f = Fixture::new(&[("coding/SKILL.md", "a"), ("web/SKILL.md", "b")]);
    let mut targets = Vec::new();
    for i in 0..40 {
        targets.push(f.target(&format!("proj{i}"), "coding"));
        targets.push(f.target(&format!("proj{i}"), "web"));
    }
    let report = run(&f, targets);
    assert_eq!(report.failed(), 0);
    assert_eq!(report.applied.len(), 80);
    for i in 0..40 {
        no_stage_left(&f.tree.path().join(format!("proj{i}")));
    }
}
