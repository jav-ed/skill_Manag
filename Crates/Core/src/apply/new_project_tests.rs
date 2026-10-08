//! Targets that create the skills directory of a new project: one failing target must not hurt its siblings.

use super::*;
use crate::events::ignore_events;
use crate::testutil::Fixture;

const BIG: &str = "y";

fn fixture() -> Fixture {
    let big = BIG.repeat(200_000);
    Fixture::new(&[
        ("a/SKILL.md", "a"),
        ("a/big.md", &big),
        ("b/SKILL.md", "b"),
        ("b/big.md", &big),
        ("c/SKILL.md", "c"),
        ("c/big.md", &big),
    ])
}

fn threads(n: usize) -> ApplyOptions<'static> {
    ApplyOptions {
        threads: n,
        ..ApplyOptions::default()
    }
}

#[test]
fn a_failing_sibling_never_breaks_the_healthy_targets_of_a_new_project() {
    let f = fixture();
    let source = f.vault.path.join("b/big.md");
    let big = BIG.repeat(200_000);
    // A race: the old code removed `skills/` for the failing target while its siblings still needed it.
    for round in 0..50 {
        let targets: Vec<_> = ["a", "b", "c"]
            .map(|s| f.new_project_target(&format!("p{round}"), s))
            .to_vec();
        let plan = f.plan_creating(targets);
        // `b` cannot be copied any more, so it fails while `a` and `c` are healthy.
        std::fs::remove_file(&source).unwrap();

        let report = apply(plan, threads(3), &ignore_events).unwrap();

        std::fs::write(&source, &big).unwrap();
        let failed: Vec<_> = report
            .applied
            .iter()
            .filter(|a| matches!(a.outcome, Outcome::Failed(_)))
            .map(|a| a.target.skill.as_str())
            .collect();
        assert_eq!(failed, ["b"], "round {round}: {:?}", report.applied);
        let project = f.tree.path().join(format!("p{round}"));
        assert!(project.join(".agents/skills/a/SKILL.md").is_file());
        assert!(project.join(".agents/skills/c/SKILL.md").is_file());
        assert!(!project.join(".agents/skills/b").exists());
    }
}

#[test]
fn when_every_target_of_a_new_project_fails_nothing_is_left_behind() {
    let f = fixture();
    let targets: Vec<_> = ["a", "b"]
        .map(|s| f.new_project_target("fresh", s))
        .to_vec();
    let plan = f.plan_creating(targets);
    std::fs::remove_file(f.vault.path.join("a/big.md")).unwrap();
    std::fs::remove_file(f.vault.path.join("b/big.md")).unwrap();

    let report = apply(plan, threads(2), &ignore_events).unwrap();

    assert_eq!(report.failed(), 2);
    assert!(!f.tree.path().join("fresh/.agents").exists());
}

#[test]
fn a_project_whose_skills_directory_cannot_be_made_fails_alone() {
    let f = fixture();
    let blocked = ["a", "b"].map(|s| f.new_project_target("blocked", s));
    let fine = f.new_project_target("fine", "a");
    let plan = f.plan_creating(vec![blocked[0].clone(), blocked[1].clone(), fine.clone()]);
    // `.agents` turns into a file after planning, so `skills/` cannot be created below it.
    std::fs::write(f.tree.path().join("blocked/.agents"), "in the way").unwrap();

    let report = apply(plan, threads(3), &ignore_events).unwrap();

    assert!(matches!(report.applied[0].outcome, Outcome::Failed(_)));
    assert!(matches!(report.applied[1].outcome, Outcome::Failed(_)));
    assert!(matches!(report.applied[2].outcome, Outcome::Created));
    assert_eq!(
        std::fs::read_to_string(f.tree.path().join("blocked/.agents")).unwrap(),
        "in the way"
    );
    assert!(fine.path.join("SKILL.md").is_file());
}
