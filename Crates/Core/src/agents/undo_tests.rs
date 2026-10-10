use std::os::unix::fs::symlink;

use super::apply_tests::{file, no_leftovers, plan};
use super::block::render;
use super::plan::{Action, Intent};
use super::{AgentsPlan, apply_agents};
use crate::backup::{Backups, Filter, RunKind, undo};
use crate::events::ignore_events;
use crate::testutil::TempTree;

struct Store {
    _dir: TempTree,
    backups: Backups,
}

fn store() -> Store {
    let dir = TempTree::new();
    let backups = Backups::at(dir.path().join("backups"));
    Store { _dir: dir, backups }
}

fn write_in_run(store: &Store, kind: RunKind, plan: &AgentsPlan, first_index: usize) -> String {
    let run = store.backups.begin(kind).unwrap();
    apply_agents(plan, Some(&run), first_index, &ignore_events);
    run.finish(&store.backups).id
}

fn undo_latest(store: &Store) -> crate::backup::UndoReport {
    undo(
        &store.backups,
        None,
        &Filter::default(),
        false,
        &ignore_events,
    )
    .unwrap()
}

#[test]
fn undo_puts_the_old_file_back_and_a_second_undo_redoes_the_update() {
    let project = TempTree::new();
    let old = format!("mine\n{}", render("old"));
    project.write("AGENTS.md", &old);
    let store = store();
    write_in_run(
        &store,
        RunKind::Sync,
        &plan(&project, "new", Intent::SYNC),
        0,
    );
    let updated = project.read("AGENTS.md");
    assert_ne!(updated, old);

    let report = undo_latest(&store);
    assert_eq!(report.failed(), 0);
    assert_eq!(project.read("AGENTS.md"), old);
    assert_eq!(report.entries[0].skill, "AGENTS.md");

    undo_latest(&store);
    assert_eq!(
        project.read("AGENTS.md"),
        updated,
        "the second undo is a redo"
    );
    no_leftovers(project.path());
}

#[test]
fn undo_removes_a_created_file_and_redo_brings_it_back() {
    let project = TempTree::new();
    let store = store();
    write_in_run(
        &store,
        RunKind::Add,
        &plan(&project, "text", Intent::ADD),
        0,
    );
    let created = project.read("AGENTS.md");

    undo_latest(&store);
    assert!(!file(&project).exists());

    undo_latest(&store);
    assert_eq!(project.read("AGENTS.md"), created);
    no_leftovers(project.path());
}

#[test]
fn undoing_a_create_after_the_user_edited_the_file_keeps_their_version_for_a_redo() {
    let project = TempTree::new();
    let store = store();
    write_in_run(
        &store,
        RunKind::Add,
        &plan(&project, "text", Intent::ADD),
        0,
    );
    let edited = format!("{}\nmy own section\n", project.read("AGENTS.md"));
    project.write("AGENTS.md", &edited);

    undo_latest(&store);
    assert!(!file(&project).exists());

    undo_latest(&store);
    assert_eq!(project.read("AGENTS.md"), edited);
}

#[test]
fn undoing_a_create_that_is_already_gone_is_not_an_error() {
    let project = TempTree::new();
    let store = store();
    write_in_run(
        &store,
        RunKind::Add,
        &plan(&project, "text", Intent::ADD),
        0,
    );
    std::fs::remove_file(file(&project)).unwrap();

    let report = undo_latest(&store);

    assert_eq!(report.failed(), 0);
}

#[test]
fn undo_refuses_to_write_through_a_link_that_took_the_files_place() {
    let project = TempTree::new();
    project.write("AGENTS.md", &render("old"));
    let store = store();
    write_in_run(
        &store,
        RunKind::Sync,
        &plan(&project, "new", Intent::SYNC),
        0,
    );
    let elsewhere = TempTree::new();
    elsewhere.write("x.md", "theirs");
    std::fs::remove_file(file(&project)).unwrap();
    symlink(elsewhere.path().join("x.md"), file(&project)).unwrap();

    let report = undo_latest(&store);

    assert_eq!(report.failed(), 1);
    assert_eq!(elsewhere.read("x.md"), "theirs");
}

#[test]
fn a_run_that_writes_skills_and_the_file_uses_separate_slots() {
    let project = TempTree::new();
    let store = store();
    let run_plan = plan(&project, "text", Intent::ADD);

    let id = write_in_run(&store, RunKind::Init, &run_plan, 3);

    let run = store.backups.load(&id).unwrap();
    assert_eq!(run.entries.len(), 1);
    assert_eq!(
        run.entries[0].index, 3,
        "the file took the slot after the skills"
    );
}

#[test]
fn the_plan_actions_are_what_apply_wrote() {
    let project = TempTree::new();
    let plan = plan(&project, "text", Intent::ADD);
    assert_eq!(plan.entries[0].action, Action::Create);
    assert_eq!(apply_agents(&plan, None, 0, &ignore_events).wrote(), 1);
}
