//! The list of backup runs and the undo of one, as the front ends use them.

use super::status_tests::world;
use super::*;
use crate::backup::RunKind;
use crate::config::Dirs;
use crate::events::ignore_events;

fn dirs(tree: &crate::testutil::TempTree) -> Dirs {
    Dirs::under(&tree.path().join("home"))
}

fn sync_once(tree: &crate::testutil::TempTree, ws: &Workspace) -> RunOutcome {
    let scanned = ws.scan(&ignore_events).unwrap();
    run_plan(
        &dirs(tree),
        RunKind::Sync,
        plan_sync(ws, &scanned),
        &ignore_events,
    )
    .unwrap()
}

#[test]
fn a_written_plan_is_listed_and_its_undo_puts_the_old_copy_back() {
    let (tree, ws) = world("tmux");
    let outcome = sync_once(&tree, &ws);
    assert_eq!(outcome.report.failed(), 0);
    assert!(outcome.backup.stored > 0, "an update keeps the old copy");

    let runs = list_runs(&dirs(&tree)).unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].id, outcome.backup.id);
    assert_eq!(runs[0].command, "sync");
    assert_eq!(runs[0].skills, outcome.backup.stored);
    assert_eq!(runs[0].error, None);

    let planned = plan_undo(&dirs(&tree), &runs[0].id).unwrap();
    assert!(!planned.applied);
    assert_eq!(planned.failed(), 0);
    assert!(planned.actionable() > 0);
    assert_eq!(
        tree.read("projects/b/.agents/skills/astro/SKILL.md"),
        "astro v2"
    );

    let done = do_undo(&dirs(&tree), &runs[0].id, &ignore_events).unwrap();
    assert!(done.applied);
    assert_eq!(
        tree.read("projects/b/.agents/skills/astro/SKILL.md"),
        "astro v1"
    );
    assert!(
        done.saved_as.is_some(),
        "undoing is a run too, so it can be undone"
    );
}

#[test]
fn a_plan_that_changes_nothing_leaves_no_run_behind() {
    let (tree, ws) = world("tmux");
    sync_once(&tree, &ws);
    let before = list_runs(&dirs(&tree)).unwrap().len();

    let second = sync_once(&tree, &ws);

    assert_eq!(second.backup.stored, 0, "everything was current already");
    assert_eq!(list_runs(&dirs(&tree)).unwrap().len(), before);
}

#[test]
fn a_skill_the_run_created_is_listed_and_undoing_removes_it() {
    let (tree, ws) = world("tmux");
    let scanned = ws.scan(&ignore_events).unwrap();
    let plan = plan_push(&ws, &scanned).unwrap();
    let outcome = run_plan(&dirs(&tree), RunKind::Push, plan, &ignore_events).unwrap();
    assert_eq!(outcome.backup.stored, 2, "tmux was created in c and d");
    assert!(tree.path().join("projects/c/.agents/skills/tmux").exists());

    let done = do_undo(&dirs(&tree), &outcome.backup.id, &ignore_events).unwrap();

    assert!(done.lines.iter().all(|l| l.step == Step::Remove));
    assert!(!tree.path().join("projects/c/.agents/skills/tmux").exists());
}

#[test]
fn undoing_a_run_that_does_not_exist_is_an_error_with_words() {
    let (tree, _ws) = world("tmux");
    let error = plan_undo(&dirs(&tree), "20200101-000000-0000").unwrap_err();
    assert!(error.len() > 3, "an error with words: {error:?}");
}

#[test]
fn an_empty_store_lists_nothing() {
    let (tree, _ws) = world("tmux");
    let runs = list_runs(&dirs(&tree)).unwrap();
    assert!(runs.is_empty(), "no run yet: {runs:?}");
}
