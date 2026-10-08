//! What the snapshot guard accepts and refuses: edits that hide their mtime, and saves that change nothing.

use super::*;
use crate::events::ignore_events;
use crate::testutil::Fixture;

fn run(f: &Fixture, target: crate::scan::Target) -> ApplyReport {
    apply(
        f.plan(vec![target]),
        ApplyOptions::default(),
        &ignore_events,
    )
    .unwrap()
}

fn set_mtime(path: &std::path::Path, time: std::time::SystemTime) {
    let file = std::fs::OpenOptions::new().write(true).open(path).unwrap();
    file.set_modified(time).unwrap();
}

#[test]
fn a_same_size_edit_that_restores_the_mtime_is_still_caught() {
    let f = Fixture::new(&[("sk/SKILL.md", "NEW!\n")]);
    let target = f.target("proj", "sk");
    f.tree.write("proj/.agents/skills/sk/SKILL.md", "OLD1\n");
    let skill_md = target.path.join("SKILL.md");
    let mtime = std::fs::metadata(&skill_md).unwrap().modified().unwrap();
    let plan = f.plan(vec![target.clone()]);
    // rsync -t, cp -p and touch -r put the old time back after an edit.
    std::fs::write(&skill_md, "EDIT\n").unwrap();
    set_mtime(&skill_md, mtime);

    let report = apply(plan, ApplyOptions::default(), &ignore_events).unwrap();

    assert!(
        matches!(
            &report.applied[0].outcome,
            Outcome::Failed(Failure::Apply(ApplyError::DestinationChanged { .. }))
        ),
        "{:?}",
        report.applied[0].outcome
    );
    assert_eq!(std::fs::read_to_string(&skill_md).unwrap(), "EDIT\n");
}

#[test]
fn saving_a_file_again_with_the_same_bytes_leaves_an_unchanged_plan_unchanged() {
    let f = Fixture::new(&[("sk/SKILL.md", "same\n")]);
    let target = f.target("proj", "sk");
    f.tree.write("proj/.agents/skills/sk/SKILL.md", "same\n");
    let plan = f.plan(vec![target.clone()]);
    assert_eq!(plan.counts().unchanged, 1);
    std::thread::sleep(std::time::Duration::from_millis(20));
    f.tree.write("proj/.agents/skills/sk/SKILL.md", "same\n");

    let report = apply(plan, ApplyOptions::default(), &ignore_events).unwrap();

    assert!(
        matches!(report.applied[0].outcome, Outcome::Unchanged),
        "{:?}",
        report.applied[0].outcome
    );
    assert_eq!(report.failed(), 0);
}

#[test]
fn a_different_edit_of_the_same_size_still_fails_an_unchanged_plan() {
    let f = Fixture::new(&[("sk/SKILL.md", "same\n")]);
    let target = f.target("proj", "sk");
    f.tree.write("proj/.agents/skills/sk/SKILL.md", "same\n");
    let plan = f.plan(vec![target.clone()]);
    std::thread::sleep(std::time::Duration::from_millis(20));
    f.tree.write("proj/.agents/skills/sk/SKILL.md", "diff\n");

    let report = apply(plan, ApplyOptions::default(), &ignore_events).unwrap();

    assert!(
        matches!(report.applied[0].outcome, Outcome::Failed(_)),
        "{:?}",
        report.applied[0].outcome
    );
    assert_eq!(f.tree.read("proj/.agents/skills/sk/SKILL.md"), "diff\n");
}

#[test]
fn an_ordinary_run_on_a_calm_folder_is_unchanged_twice() {
    let f = Fixture::new(&[("sk/SKILL.md", "v\n"), ("sk/lang/js.md", "js\n")]);
    let target = f.target("proj", "sk");
    run(&f, target.clone());

    let report = run(&f, target);

    assert!(matches!(report.applied[0].outcome, Outcome::Unchanged));
}
