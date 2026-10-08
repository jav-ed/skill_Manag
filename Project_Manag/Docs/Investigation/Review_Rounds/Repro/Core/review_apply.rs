//! Review round 2, series A (apply: created directories and the snapshot check). They print what they observe.
//!
//! Findings: M3 (A1), L1 (A2), L2 (A3). A4 documents a behavior that is correct. See
//! `Project_Manag/Docs/Investigation/Review_Rounds/round_2_Full.md`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::print_stdout)]

mod review_common;

use std::os::unix::fs::PermissionsExt;

use review_common::{commit_all, short, small_vault, target, write};
use skillmirror_core::apply::{ApplyOptions, Outcome, apply};
use skillmirror_core::events::ignore_events;
use skillmirror_core::plan::Plan;
use skillmirror_core::vault::{discover, read_files};

fn options(threads: usize) -> ApplyOptions<'static> {
    ApplyOptions {
        threads,
        ..ApplyOptions::default()
    }
}

#[test]
fn a1_failing_target_removes_the_skills_dir_its_sibling_still_needs() {
    let tmp = tempfile::tempdir().unwrap();
    let vault = tmp.path().join("vault");
    for skill in ["a", "b", "c"] {
        write(&vault.join(skill).join("SKILL.md"), "x");
        write(&vault.join(skill).join("big.md"), &"y".repeat(200_000));
    }
    commit_all(&vault);
    // `b` cannot be read, so it must fail; `a` and `c` are healthy.
    std::fs::set_permissions(
        vault.join("b/big.md"),
        std::fs::Permissions::from_mode(0o000),
    )
    .unwrap();
    let found = discover(&vault).unwrap();
    let files = read_files(&found).unwrap();
    let rounds = 300;
    let (mut a_failed, mut b_failed, mut c_failed) = (0, 0, 0);
    let mut sample = String::new();
    for i in 0..rounds {
        let project = tmp.path().join(format!("p{i}"));
        std::fs::create_dir_all(&project).unwrap();
        let targets = ["a", "b", "c"].map(|s| target(&project, s)).to_vec();
        let plan = Plan::for_targets_creating(&found, &files, targets);
        let report = apply(plan, options(3), &ignore_events).unwrap();
        for applied in &report.applied {
            if let Outcome::Failed(failure) = &applied.outcome {
                match applied.target.skill.as_str() {
                    "a" => a_failed += 1,
                    "b" => b_failed += 1,
                    _ => c_failed += 1,
                }
                if applied.target.skill != "b" && sample.is_empty() {
                    sample = format!("{failure:?}");
                }
            }
        }
    }
    std::fs::set_permissions(
        vault.join("b/big.md"),
        std::fs::Permissions::from_mode(0o644),
    )
    .unwrap();
    println!(
        "A1 {rounds} rounds on fresh projects: b failed {b_failed}x (expected), a failed {a_failed}x, c failed {c_failed}x"
    );
    if !sample.is_empty() {
        println!(
            "A1 sample failure of an innocent sibling: {}",
            sample.chars().take(220).collect::<String>()
        );
    }
}

#[test]
fn a2_snapshot_check_misses_an_edit_that_keeps_size_and_mtime() {
    let tmp = tempfile::tempdir().unwrap();
    let (found, files) = small_vault(tmp.path());
    let t = target(&tmp.path().join("proj"), "sk");
    write(&t.path.join("SKILL.md"), "OLD1\n");
    let mtime = std::fs::metadata(t.path.join("SKILL.md"))
        .unwrap()
        .modified()
        .unwrap();
    let plan = Plan::for_targets(&found, &files, vec![t.clone()]);
    // A same-size edit that restores the modification time (rsync -t, cp -p, some editors and formatters).
    std::fs::write(t.path.join("SKILL.md"), "EDIT\n").unwrap();
    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(t.path.join("SKILL.md"))
        .unwrap();
    file.set_modified(mtime).unwrap();
    let report = apply(plan, options(8), &ignore_events).unwrap();
    let now = std::fs::read_to_string(t.path.join("SKILL.md")).unwrap();
    println!(
        "A2 same-size edit with restored mtime: {:?}; now {now:?}",
        short(&report.applied[0].outcome)
    );

    // An ordinary edit is caught.
    let plan = Plan::for_targets(&found, &files, vec![t.clone()]);
    write(&t.path.join("SKILL.md"), "OLD1 plus my edit\n");
    let report = apply(plan, options(8), &ignore_events).unwrap();
    let kept = std::fs::read_to_string(t.path.join("SKILL.md")).unwrap();
    println!(
        "A2 ordinary edit: {:?}; file kept: {kept:?}",
        short(&report.applied[0].outcome)
    );
}

#[test]
fn a3_unchanged_plan_fails_when_the_folder_is_merely_touched() {
    let tmp = tempfile::tempdir().unwrap();
    let (found, files) = small_vault(tmp.path());
    let t = target(&tmp.path().join("proj"), "sk");
    write(&t.path.join("SKILL.md"), "NEW!\n");
    std::fs::set_permissions(
        t.path.join("SKILL.md"),
        std::fs::Permissions::from_mode(0o644),
    )
    .unwrap();
    let plan = Plan::for_targets(&found, &files, vec![t.clone()]);
    println!(
        "A3 plan kind: {:?}",
        plan.entries[0].result.as_ref().map(|p| p.kind)
    );
    std::thread::sleep(std::time::Duration::from_millis(20));
    write(&t.path.join("SKILL.md"), "NEW!\n"); // same bytes, new mtime
    let report = apply(plan, options(8), &ignore_events).unwrap();
    println!(
        "A3 outcome after an identical re-save: {}",
        short(&report.applied[0].outcome)
    );
}

#[test]
fn a4_undo_of_a_stale_swap_keeps_the_users_edit() {
    let tmp = tempfile::tempdir().unwrap();
    let (found, files) = small_vault(tmp.path());
    let project = tmp.path().join("proj");
    let t = target(&project, "sk");
    write(&t.path.join("SKILL.md"), "OLD\n");
    let plan = Plan::for_targets(&found, &files, vec![t.clone()]);
    write(&t.path.join("notes.md"), "mine\n");
    let report = apply(plan, options(8), &ignore_events).unwrap();
    let mut left: Vec<_> = std::fs::read_dir(project.join(".agents"))
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    left.sort();
    let skill = std::fs::read_to_string(t.path.join("SKILL.md")).unwrap();
    println!(
        "A4 outcome: {}; notes.md kept: {}; SKILL.md: {skill:?}; .agents: {left:?}",
        short(&report.applied[0].outcome),
        t.path.join("notes.md").exists()
    );
}
