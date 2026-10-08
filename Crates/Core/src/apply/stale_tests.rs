//! Plans that go stale while they wait, and failures that must leave nothing behind.

use std::os::unix::fs::PermissionsExt;

use super::*;
use crate::events::ignore_events;
use crate::plan::Plan;
use crate::scan::Target;
use crate::testutil::Fixture;

fn outcome_text(report: &ApplyReport) -> String {
    format!("{:?}", report.applied[0].outcome)
}

#[test]
fn an_edit_made_after_planning_is_never_overwritten() {
    let f = Fixture::new(&[("coding/SKILL.md", "vault version")]);
    let target = f.target("proj", "coding");
    f.tree.write("proj/.agents/skills/coding/SKILL.md", "old");
    let plan = f.plan(vec![target.clone()]);
    // The user edits the copy while the plan waits for a confirmation.
    f.tree
        .write("proj/.agents/skills/coding/SKILL.md", "my own edit");
    f.tree
        .write("proj/.agents/skills/coding/my_notes.md", "notes");

    let report = apply(plan, ApplyOptions::default(), &ignore_events).unwrap();
    assert!(
        matches!(
            &report.applied[0].outcome,
            Outcome::Failed(Failure::Apply(ApplyError::DestinationChanged { .. }))
        ),
        "{}",
        outcome_text(&report)
    );
    assert_eq!(
        f.tree.read("proj/.agents/skills/coding/SKILL.md"),
        "my own edit"
    );
    assert_eq!(
        f.tree.read("proj/.agents/skills/coding/my_notes.md"),
        "notes"
    );
    let leftovers: Vec<_> = std::fs::read_dir(f.tree.path().join("proj/.agents"))
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(
        leftovers.len(),
        1,
        "no stage directory stays: {leftovers:?}"
    );
}

#[test]
fn an_edit_that_keeps_the_size_is_noticed_too() {
    let f = Fixture::new(&[("coding/SKILL.md", "vault version")]);
    let target = f.target("proj", "coding");
    f.tree.write("proj/.agents/skills/coding/SKILL.md", "aaaa");
    let plan = f.plan(vec![target]);
    std::thread::sleep(std::time::Duration::from_millis(20));
    f.tree.write("proj/.agents/skills/coding/SKILL.md", "bbbb");
    let report = apply(plan, ApplyOptions::default(), &ignore_events).unwrap();
    assert!(
        matches!(report.applied[0].outcome, Outcome::Failed(_)),
        "{}",
        outcome_text(&report)
    );
    assert_eq!(f.tree.read("proj/.agents/skills/coding/SKILL.md"), "bbbb");
}

#[test]
fn an_unchanged_plan_is_not_trusted_after_an_edit() {
    let f = Fixture::new(&[("coding/SKILL.md", "same")]);
    let target = f.target("proj", "coding");
    f.tree.write("proj/.agents/skills/coding/SKILL.md", "same");
    std::fs::set_permissions(
        target.path.join("SKILL.md"),
        std::fs::Permissions::from_mode(
            std::fs::metadata(f.vault.path.join("coding/SKILL.md"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
        ),
    )
    .unwrap();
    let plan = f.plan(vec![target]);
    assert_eq!(plan.counts().unchanged, 1);
    f.tree
        .write("proj/.agents/skills/coding/extra.md", "added later");
    let report = apply(plan, ApplyOptions::default(), &ignore_events).unwrap();
    assert!(
        matches!(report.applied[0].outcome, Outcome::Failed(_)),
        "{}",
        outcome_text(&report)
    );
}

fn running_as_root() -> bool {
    use std::os::unix::fs::MetadataExt;
    std::fs::metadata("/proc/self").unwrap().uid() == 0
}

#[test]
fn an_old_copy_that_cannot_be_removed_is_a_warning_not_a_failure() {
    if running_as_root() {
        return; // root ignores the permission bits this test relies on
    }
    let f = Fixture::new(&[("coding/SKILL.md", "new\n")]);
    let target = f.target("proj", "coding");
    f.tree.write("proj/.agents/skills/coding/SKILL.md", "old\n");
    f.tree
        .write("proj/.agents/skills/coding/locked/inner.md", "x");
    let locked = target.path.join("locked");
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o555)).unwrap();

    let report = apply(
        f.plan(vec![target.clone()]),
        ApplyOptions::default(),
        &ignore_events,
    )
    .unwrap();
    let applied = &report.applied[0];
    let leftover = applied.leftover.as_ref().expect("the old copy stays");
    // Make the leftover removable again before any assertion can fail.
    std::fs::set_permissions(
        leftover.path.join("locked"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    assert!(
        matches!(applied.outcome, Outcome::Updated),
        "{:?}",
        applied.outcome
    );
    assert_eq!(report.failed(), 0);
    assert_eq!(
        std::fs::read_to_string(target.path.join("SKILL.md")).unwrap(),
        "new\n"
    );
    assert!(leftover.path.to_string_lossy().contains(".stage-"));
}

#[test]
fn a_failed_create_in_a_new_project_leaves_no_empty_agents_directory() {
    let f = Fixture::new(&[("coding/SKILL.md", "a")]);
    let project = f.tree.path().join("fresh");
    std::fs::create_dir_all(&project).unwrap();
    let target = Target {
        project: project.clone(),
        skill: "coding".to_string(),
        path: project.join(".agents/skills/coding"),
    };
    let plan = Plan::for_targets_creating(&f.vault, &f.files, vec![target]);
    std::fs::remove_file(f.vault.path.join("coding/SKILL.md")).unwrap();
    let report = apply(plan, ApplyOptions::default(), &ignore_events).unwrap();
    assert!(
        matches!(report.applied[0].outcome, Outcome::Failed(_)),
        "{}",
        outcome_text(&report)
    );
    assert!(
        !project.join(".agents").exists(),
        "the failure took back what it created"
    );
}

#[test]
fn the_swap_hint_blames_permissions_for_a_permission_error_and_the_filesystem_otherwise() {
    use crate::Hint;
    let swap = |code: i32| ApplyError::Swap {
        stage: "/s".into(),
        dest: "/d".into(),
        source: std::io::Error::from_raw_os_error(code),
    };
    assert!(swap(13).hint().unwrap().contains("permission"));
    assert!(swap(22).hint().unwrap().contains("renameat2"));
    assert!(swap(2).hint().is_none());
}
