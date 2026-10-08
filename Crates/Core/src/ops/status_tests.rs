//! `status`: every project against the vault, and nothing written.

use super::*;
use crate::config::{Dirs, EnvOverrides, Flags, Settings};
use crate::events::ignore_events;
use crate::testutil::TempTree;

/// A vault with coding, astro and tmux (tmux is mandatory) and four projects in different states.
fn world(mandatory: &str) -> (TempTree, Workspace) {
    let tree = TempTree::new();
    tree.write_all(&[
        ("vault/coding/SKILL.md", "coding v2"),
        ("vault/web/astro/SKILL.md", "astro v2"),
        ("vault/web/astro/ref.md", "ref"),
        ("vault/tmux/SKILL.md", "tmux v2"),
        (
            "vault/config.yaml",
            &format!(
                "root: {}/projects\nmandatory: [{mandatory}]\n",
                tree.path().display()
            ),
        ),
        // a: everything current
        ("projects/a/.agents/skills/coding/SKILL.md", "coding v2"),
        ("projects/a/.agents/skills/tmux/SKILL.md", "tmux v2"),
        // b: outdated astro (one changed file, one added, one stray file), a skill the vault lacks
        ("projects/b/.agents/skills/astro/SKILL.md", "astro v1"),
        ("projects/b/.agents/skills/astro/stray/notes.md", "mine"),
        ("projects/b/.agents/skills/local-only/SKILL.md", "mine"),
        ("projects/b/.agents/skills/tmux/SKILL.md", "tmux v2"),
        // c: no tmux at all
        ("projects/c/.agents/skills/coding/SKILL.md", "coding v2"),
        // d: has a skills directory and nothing in it
        ("projects/d/.agents/skills/.keep", ""),
        ("projects/plain/src/main.rs", ""),
    ]);
    tree.git_init_commit("vault");
    let flags = Flags {
        vault: Some(tree.path().join("vault")),
        root: None,
    };
    let dirs = Dirs::under(&tree.path().join("home"));
    let settings = Settings::load(&flags, &EnvOverrides::default(), &dirs).unwrap();
    (tree, Workspace::open(settings).unwrap())
}

fn project<'a>(report: &'a StatusReport, name: &str) -> &'a ProjectStatus {
    report
        .projects
        .iter()
        .find(|p| p.project.ends_with(name))
        .unwrap_or_else(|| panic!("no project {name}"))
}

fn names(items: &[String]) -> Vec<&str> {
    items.iter().map(String::as_str).collect()
}

#[test]
fn every_project_with_a_skills_directory_is_listed_with_what_differs() {
    let (_tree, ws) = world("tmux");
    let scanned = ws.scan(&ignore_events).unwrap();

    let report = status(&ws, &scanned).unwrap();

    assert_eq!(
        report.projects.len(),
        4,
        "the project without .agents is not one"
    );
    let a = project(&report, "a");
    assert_eq!(names(&a.up_to_date), ["coding", "tmux"]);
    assert!(!a.drifts());

    let b = project(&report, "b");
    assert_eq!(names(&b.up_to_date), ["tmux"]);
    assert_eq!(
        b.outdated,
        [Outdated {
            skill: "astro".to_string(),
            added: 1,
            changed: 1,
            removed: 1
        }],
        "astro: ref.md is new, SKILL.md changed, stray/notes.md would go (the folder stray counts once, through its file)"
    );
    assert_eq!(names(&b.not_in_vault), ["local-only"]);
    assert!(b.drifts());

    let c = project(&report, "c");
    assert_eq!(names(&c.missing_mandatory), ["tmux"]);
    assert!(c.drifts());

    let d = project(&report, "d");
    assert_eq!(names(&d.missing_mandatory), ["tmux"]);
    assert_eq!(report.drifting(), 3);
    assert_eq!(report.failed(), 0);
}

#[test]
fn a_skill_the_vault_lacks_alone_is_no_drift() {
    let (tree, ws) = world("");
    std::fs::remove_dir_all(tree.path().join("projects/b/.agents/skills/astro")).unwrap();
    let scanned = ws.scan(&ignore_events).unwrap();

    let report = status(&ws, &scanned).unwrap();

    let b = project(&report, "b");
    assert_eq!(names(&b.not_in_vault), ["local-only"]);
    assert!(!b.drifts());
}

#[test]
fn status_writes_nothing() {
    let (tree, ws) = world("tmux");
    let scanned = ws.scan(&ignore_events).unwrap();
    let before = snapshot(&tree.path().join("projects"));

    status(&ws, &scanned).unwrap();

    assert_eq!(snapshot(&tree.path().join("projects")), before);
}

fn snapshot(dir: &std::path::Path) -> Vec<(String, Option<String>)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(next) = stack.pop() {
        for entry in std::fs::read_dir(&next).unwrap() {
            let path = entry.unwrap().path();
            let rel = path
                .strip_prefix(dir)
                .unwrap()
                .to_string_lossy()
                .into_owned();
            if path.is_dir() {
                out.push((rel, None));
                stack.push(path);
            } else {
                out.push((rel, Some(std::fs::read_to_string(&path).unwrap())));
            }
        }
    }
    out.sort();
    out
}

#[test]
fn a_skill_that_cannot_be_compared_is_reported_not_hidden() {
    let (tree, ws) = world("tmux");
    // The project copy of tmux is a symlink: the plan refuses to write through it.
    std::fs::remove_dir_all(tree.path().join("projects/b/.agents/skills/tmux")).unwrap();
    std::os::unix::fs::symlink(
        tree.path().join("projects/a/.agents/skills/tmux"),
        tree.path().join("projects/b/.agents/skills/tmux"),
    )
    .unwrap();
    let scanned = ws.scan(&ignore_events).unwrap();

    let report = status(&ws, &scanned).unwrap();

    let b = project(&report, "b");
    assert_eq!(b.failed.len(), 1, "{:?}", b.failed);
    assert_eq!(b.failed[0].skill, "tmux");
    assert!(
        b.failed[0].message.contains("symlink"),
        "{}",
        b.failed[0].message
    );
    assert_eq!(report.failed(), 1);
}

#[test]
fn an_unknown_mandatory_name_is_a_hard_error_as_for_push() {
    let (_tree, ws) = world("ghost");
    let scanned = ws.scan(&ignore_events).unwrap();

    let err = status(&ws, &scanned).unwrap_err();

    assert!(
        matches!(err, crate::plan::PlanError::MandatoryNotInVault { .. }),
        "{err}"
    );
}

/// The same world with `targets: [claude]` in the vault config, committed and reopened.
fn with_claude_target(tree: &TempTree) -> Workspace {
    let config = std::fs::read_to_string(tree.path().join("vault/config.yaml")).unwrap();
    tree.write("vault/config.yaml", &format!("{config}targets: [claude]\n"));
    tree.git("vault", &["commit", "-aqm", "targets"]);
    let flags = Flags {
        vault: Some(tree.path().join("vault")),
        root: None,
    };
    let dirs = Dirs::under(&tree.path().join("home"));
    Workspace::open(Settings::load(&flags, &EnvOverrides::default(), &dirs).unwrap()).unwrap()
}

#[test]
fn a_missing_bridge_is_drift_and_a_blocked_one_is_a_problem() {
    let (tree, _) = world("tmux");
    let ws = with_claude_target(&tree);
    // b has a real folder where the link should go.
    tree.write("projects/b/.claude/skills/mine/SKILL.md", "mine\n");
    let scanned = ws.scan(&ignore_events).unwrap();

    let report = status(&ws, &scanned).unwrap();

    let a = project(&report, "a");
    assert_eq!(names(&a.missing_bridges), ["claude"]);
    assert!(a.drifts(), "the bridge command would change this project");
    let b = project(&report, "b");
    assert!(
        b.missing_bridges.is_empty(),
        "a blocked bridge is a problem, not a missing one"
    );
    assert!(
        b.failed.is_empty(),
        "a link problem is not a skill that failed: {:?}",
        b.failed
    );
    assert_eq!(
        report.failed(),
        1,
        "but it counts as a problem of the project"
    );
    let blocked = b.project_problems.first().unwrap();
    assert_eq!(blocked.skill, "claude", "named by the target");
    assert!(
        blocked.message.contains("real folder"),
        "{}",
        blocked.message
    );
}

#[test]
fn a_bridge_in_place_is_no_drift() {
    let (tree, _) = world("");
    let ws = with_claude_target(&tree);
    for project in ["a", "b", "c", "d"] {
        let bridges = plan_project_bridges(
            &["claude".to_string()],
            &tree.path().join("projects").join(project),
        );
        create_bridge(&bridges[0]).unwrap();
    }
    let scanned = ws.scan(&ignore_events).unwrap();

    let report = status(&ws, &scanned).unwrap();

    assert!(report.projects.iter().all(|p| p.missing_bridges.is_empty()));
}

#[test]
fn a_skill_folder_named_like_a_link_problem_is_still_a_skill() {
    let (tree, _) = world("tmux");
    tree.write("projects/b/.agents/skills/bridge claude/SKILL.md", "mine");
    let ws = with_claude_target(&tree);
    tree.write("projects/b/.claude/skills/own/SKILL.md", "own");
    let scanned = ws.scan(&ignore_events).unwrap();

    let report = status(&ws, &scanned).unwrap();

    let b = project(&report, "b");
    assert!(
        b.not_in_vault.contains(&"bridge claude".to_string()),
        "{:?}",
        b.not_in_vault
    );
    assert_eq!(b.project_problems.len(), 1);
    assert!(b.failed.is_empty(), "{:?}", b.failed);
}
