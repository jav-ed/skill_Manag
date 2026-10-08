//! Scoped sync, push and status: a named skill or one project, and nothing outside it.

use std::collections::BTreeSet;

use super::status_tests::world;
use super::*;
use crate::events::ignore_events;

fn skills(names: &[&str]) -> Option<BTreeSet<String>> {
    let set: BTreeSet<String> = names.iter().map(|n| (*n).to_string()).collect();
    (!set.is_empty()).then_some(set)
}

fn targets(plan: &crate::plan::Plan) -> Vec<String> {
    let mut out: Vec<String> = plan
        .entries
        .iter()
        .map(|e| format!("{}:{}", project_name(&e.target.project), e.target.skill))
        .collect();
    out.sort();
    out
}

fn project_name(path: &std::path::Path) -> String {
    path.file_name().unwrap().to_string_lossy().into_owned()
}

#[test]
fn an_empty_scope_plans_what_the_unscoped_function_plans() {
    let (_tree, ws) = world("tmux");
    let scanned = ws.scan(&ignore_events).unwrap();
    let all = plan_sync(&ws, &scanned);
    let scoped = plan_sync_scoped(&ws, &scanned, &Scope::default()).unwrap();
    assert_eq!(targets(&all), targets(&scoped));
    assert!(
        Scope::default().is_everything(),
        "the default scope is everything"
    );
}

#[test]
fn sync_of_a_named_skill_leaves_the_others_alone() {
    let (_tree, ws) = world("tmux");
    let scanned = ws.scan(&ignore_events).unwrap();
    let scope = Scope {
        skills: skills(&["coding"]),
        project: None,
    };
    let plan = plan_sync_scoped(&ws, &scanned, &scope).unwrap();
    assert_eq!(targets(&plan), ["a:coding", "c:coding"]);
}

#[test]
fn a_named_skill_is_not_added_to_a_project_that_lacks_it() {
    let (tree, ws) = world("tmux");
    let scanned = ws.scan(&ignore_events).unwrap();
    let scope = Scope {
        skills: skills(&["coding"]),
        project: Some(tree.path().join("projects/b")),
    };
    let plan = plan_sync_scoped(&ws, &scanned, &scope).unwrap();
    assert!(
        plan.entries.is_empty(),
        "project b has no coding folder, so sync must not create one"
    );
}

#[test]
fn sync_of_one_project_takes_every_skill_of_that_project_only() {
    let (tree, ws) = world("tmux");
    let scanned = ws.scan(&ignore_events).unwrap();
    let scope = Scope {
        skills: None,
        project: Some(tree.path().join("projects/b")),
    };
    let plan = plan_sync_scoped(&ws, &scanned, &scope).unwrap();
    assert_eq!(targets(&plan), ["b:astro", "b:tmux"]);
}

#[test]
fn a_project_the_scan_did_not_find_is_an_error_not_an_empty_plan() {
    let (tree, ws) = world("tmux");
    let scanned = ws.scan(&ignore_events).unwrap();
    let scope = Scope {
        skills: None,
        project: Some(tree.path().join("projects/plain")),
    };
    let error = plan_sync_scoped(&ws, &scanned, &scope).unwrap_err();
    assert!(
        matches!(error, ScopeError::ProjectNotFound { .. }),
        "got {error:?}"
    );
}

#[test]
fn a_project_given_through_a_link_is_the_same_project() {
    let (tree, ws) = world("tmux");
    std::os::unix::fs::symlink(
        tree.path().join("projects/b"),
        tree.path().join("projects-b-link"),
    )
    .unwrap();
    let scanned = ws.scan(&ignore_events).unwrap();
    let scope = Scope {
        skills: None,
        project: Some(tree.path().join("projects-b-link")),
    };
    let plan = plan_sync_scoped(&ws, &scanned, &scope).unwrap();
    assert_eq!(targets(&plan), ["b:astro", "b:tmux"]);
}

#[test]
fn push_of_one_project_installs_the_mandatory_skills_there_only() {
    let (tree, ws) = world("tmux");
    let scanned = ws.scan(&ignore_events).unwrap();
    let scope = Scope {
        skills: None,
        project: Some(tree.path().join("projects/c")),
    };
    let plan = plan_push_scoped(&ws, &scanned, &scope).unwrap();
    assert_eq!(targets(&plan), ["c:tmux"]);
}

#[test]
fn push_of_a_skill_that_is_not_mandatory_is_refused() {
    let (_tree, ws) = world("tmux");
    let scanned = ws.scan(&ignore_events).unwrap();
    let scope = Scope {
        skills: skills(&["coding"]),
        project: None,
    };
    let error = plan_push_scoped(&ws, &scanned, &scope).unwrap_err();
    assert!(
        matches!(error, crate::Error::Scope(ScopeError::NotMandatory { .. })),
        "got {error:?}"
    );
}

#[test]
fn push_of_a_mandatory_skill_by_name_works() {
    let (_tree, ws) = world("tmux, coding");
    let scanned = ws.scan(&ignore_events).unwrap();
    let scope = Scope {
        skills: skills(&["tmux"]),
        project: None,
    };
    let plan = plan_push_scoped(&ws, &scanned, &scope).unwrap();
    assert_eq!(targets(&plan), ["a:tmux", "b:tmux", "c:tmux", "d:tmux"]);
}

#[test]
fn status_of_a_named_skill_lists_only_that_skill() {
    let (_tree, ws) = world("tmux");
    let scanned = ws.scan(&ignore_events).unwrap();
    let scope = Scope {
        skills: skills(&["astro"]),
        project: None,
    };
    let report = status_scoped(&ws, &scanned, &scope).unwrap();
    assert_eq!(report.projects.len(), 1, "only project b has astro");
    let b = &report.projects[0];
    assert!(b.project.ends_with("b"), "got {:?}", b.project);
    assert_eq!(b.outdated.len(), 1);
    assert!(b.up_to_date.is_empty() && b.not_in_vault.is_empty());
}

#[test]
fn status_of_a_skill_a_project_lacks_as_mandatory_names_the_project() {
    let (_tree, ws) = world("tmux");
    let scanned = ws.scan(&ignore_events).unwrap();
    let scope = Scope {
        skills: skills(&["tmux"]),
        project: None,
    };
    let report = status_scoped(&ws, &scanned, &scope).unwrap();
    let missing: Vec<_> = report
        .projects
        .iter()
        .filter(|p| !p.missing_mandatory.is_empty())
        .map(|p| project_name(&p.project))
        .collect();
    assert_eq!(missing, ["c", "d"]);
}

#[test]
fn status_of_one_project_hides_the_others() {
    let (tree, ws) = world("tmux");
    let scanned = ws.scan(&ignore_events).unwrap();
    let scope = Scope {
        skills: None,
        project: Some(tree.path().join("projects/a")),
    };
    let report = status_scoped(&ws, &scanned, &scope).unwrap();
    assert_eq!(report.projects.len(), 1);
    assert!(report.projects[0].project.ends_with("a"));
}
