use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use super::block::render;
use super::plan::{Intent, plan_agents};
use super::text::{Source, load_source};
use super::{AgentsError, AgentsPlan, Written, apply_agents};
use crate::apply::ApplyError;
use crate::events::{Event, Status, ignore_events};
use crate::testutil::TempTree;

pub(super) fn source(text: &str) -> Source {
    let vault = TempTree::new();
    vault.write("AGENTS.md", text);
    load_source(vault.path()).unwrap()
}

pub(super) fn plan(project: &TempTree, text: &str, intent: Intent) -> AgentsPlan {
    plan_agents(&[project.path().to_path_buf()], &source(text), intent)
}

pub(super) fn file(project: &TempTree) -> PathBuf {
    project.path().join("AGENTS.md")
}

pub(super) fn no_leftovers(project: &Path) {
    let names: Vec<String> = std::fs::read_dir(project)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        names
            .iter()
            .all(|n| !n.contains(".stage-") && !n.contains(".trash-")),
        "leftovers in {names:?}"
    );
}

#[test]
fn a_create_writes_the_planned_file_with_ordinary_permissions_and_leaves_nothing_behind() {
    let project = TempTree::new();
    let plan = plan(&project, "# Rules", Intent::ADD);

    let report = apply_agents(&plan, None, 0, &ignore_events);

    assert_eq!(report.wrote(), 1);
    assert_eq!(
        report.applied[0].result.as_ref().unwrap(),
        &Written::Created
    );
    assert_eq!(project.read("AGENTS.md"), render("# Rules"));
    let mode = std::fs::metadata(file(&project))
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode & 0o600, 0o600);
    no_leftovers(project.path());
}

#[test]
fn an_update_changes_only_the_block_and_keeps_the_permission_bits() {
    let project = TempTree::new();
    project.write("AGENTS.md", &format!("mine\n{}tail\n", render("old")));
    std::fs::set_permissions(file(&project), std::fs::Permissions::from_mode(0o640)).unwrap();

    let report = apply_agents(
        &plan(&project, "new", Intent::SYNC),
        None,
        0,
        &ignore_events,
    );

    assert_eq!(
        report.applied[0].result.as_ref().unwrap(),
        &Written::Updated
    );
    assert_eq!(
        project.read("AGENTS.md"),
        format!("mine\n{}tail\n", render("new"))
    );
    let mode = std::fs::metadata(file(&project))
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o640);
    no_leftovers(project.path());
}

#[test]
fn an_insert_keeps_the_whole_old_file() {
    let project = TempTree::new();
    project.write("AGENTS.md", "all mine\n");

    let report = apply_agents(
        &plan(&project, "text", Intent::ADD),
        None,
        0,
        &ignore_events,
    );

    assert_eq!(
        report.applied[0].result.as_ref().unwrap(),
        &Written::Inserted
    );
    assert_eq!(
        project.read("AGENTS.md"),
        format!("{}\nall mine\n", render("text"))
    );
}

#[test]
fn an_edit_made_after_planning_survives_and_the_write_fails() {
    let project = TempTree::new();
    project.write("AGENTS.md", &format!("mine\n{}", render("old")));
    let plan = plan(&project, "new", Intent::SYNC);
    // Same length as before, different bytes: the kind of edit a size check would miss.
    let edited = project.read("AGENTS.md").replace("mine", "MINE");
    project.write("AGENTS.md", &edited);

    let report = apply_agents(&plan, None, 0, &ignore_events);

    assert!(matches!(
        report.applied[0].result,
        Err(AgentsError::Apply(ApplyError::DestinationChanged { .. }))
    ));
    assert_eq!(project.read("AGENTS.md"), edited);
    no_leftovers(project.path());
}

#[test]
fn a_file_that_appeared_after_planning_is_never_overwritten_by_a_create() {
    let project = TempTree::new();
    let plan = plan(&project, "text", Intent::ADD);
    project.write("AGENTS.md", "written meanwhile");

    let report = apply_agents(&plan, None, 0, &ignore_events);

    assert!(report.applied[0].result.is_err());
    assert_eq!(project.read("AGENTS.md"), "written meanwhile");
    no_leftovers(project.path());
}

#[test]
fn a_file_turned_into_a_link_after_planning_is_not_written_through_or_replaced() {
    let project = TempTree::new();
    project.write("AGENTS.md", &render("old"));
    let plan = plan(&project, "new", Intent::SYNC);
    let target = TempTree::new();
    target.write("real.md", "someone else's file");
    std::fs::remove_file(file(&project)).unwrap();
    symlink(target.path().join("real.md"), file(&project)).unwrap();

    let report = apply_agents(&plan, None, 0, &ignore_events);

    assert!(matches!(
        report.applied[0].result,
        Err(AgentsError::Refused { .. })
    ));
    assert_eq!(target.read("real.md"), "someone else's file");
    assert!(
        std::fs::symlink_metadata(file(&project))
            .unwrap()
            .file_type()
            .is_symlink()
    );
    no_leftovers(project.path());
}

#[test]
fn entries_the_plan_does_not_write_are_reported_not_touched_and_a_failed_one_is_an_error() {
    let current = TempTree::new();
    current.write("AGENTS.md", &render("same"));
    let broken = TempTree::new();
    broken.write("AGENTS.md", "<!-- skillmirror:autogenerated end -->\n");
    let projects = [current.path().to_path_buf(), broken.path().to_path_buf()];
    let plan = plan_agents(&projects, &source("same"), Intent::SYNC);
    let events = std::sync::Mutex::new(Vec::new());

    let report = apply_agents(&plan, None, 0, &|e| events.lock().unwrap().push(e));

    assert_eq!(report.wrote(), 0);
    assert_eq!(report.failed(), 1);
    assert_eq!(
        report.applied[0].result.as_ref().unwrap(),
        &Written::Unchanged
    );
    assert!(matches!(
        report.applied[1].result,
        Err(AgentsError::Refused { .. })
    ));
    assert_eq!(current.read("AGENTS.md"), render("same"));
    let statuses: Vec<Status> = events
        .into_inner()
        .unwrap()
        .into_iter()
        .filter_map(|e| match e {
            Event::TargetDone { status, target } => {
                assert_eq!(target.skill, "AGENTS.md");
                Some(status)
            }
            _ => None,
        })
        .collect();
    assert_eq!(statuses[0], Status::Unchanged);
    assert!(matches!(statuses[1], Status::Failed(_)));
}
