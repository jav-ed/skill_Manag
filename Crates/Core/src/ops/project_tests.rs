use std::collections::BTreeSet;

use super::*;
use crate::apply::{ApplyOptions, Outcome, apply};
use crate::config::{Dirs, EnvOverrides, Flags, Settings};
use crate::events::ignore_events;
use crate::testutil::TempTree;

const CONFIG: &str = "\
mandatory: [coding]
profiles:
  base:
    skills: [coding]
  web-site:
    description: A site
    extends: [base]
    groups: [web]
    exclude: [vite]
  tiny:
    skills: [tmux]
";

fn world() -> (TempTree, Workspace) {
    let tree = TempTree::new();
    tree.write_all(&[
        ("vault/coding/SKILL.md", "coding"),
        ("vault/tmux/SKILL.md", "tmux"),
        ("vault/web/astro/SKILL.md", "astro"),
        ("vault/web/vite/SKILL.md", "vite"),
        ("vault/web/seo/schema/SKILL.md", "schema"),
        ("vault/config.yaml", CONFIG),
    ]);
    tree.git_init_commit("vault");
    let flags = Flags {
        vault: Some(tree.path().join("vault")),
        root: Some(tree.path().join("projects")),
    };
    let settings = Settings::load(
        &flags,
        &EnvOverrides::default(),
        &Dirs::under(&tree.path().join("home")),
    )
    .unwrap();
    let workspace = Workspace::open(settings).unwrap();
    (tree, workspace)
}

fn names(set: &BTreeSet<String>) -> Vec<&str> {
    set.iter().map(String::as_str).collect()
}

fn pick(
    workspace: &Workspace,
    skills: &[&str],
    groups: &[&str],
    profiles: &[&str],
) -> Result<BTreeSet<String>, SelectError> {
    let own = |items: &[&str]| items.iter().map(|s| (*s).to_string()).collect();
    let selection = Selection {
        skills: own(skills),
        groups: own(groups),
        profiles: own(profiles),
    };
    resolve(&workspace.vault, workspace.settings.config(), &selection)
}

#[test]
fn skills_groups_and_profiles_resolve_to_skill_names() {
    let (_tree, ws) = world();
    assert_eq!(names(&pick(&ws, &["tmux"], &[], &[]).unwrap()), ["tmux"]);
    assert_eq!(
        names(&pick(&ws, &[], &["web"], &[]).unwrap()),
        ["astro", "schema", "vite"]
    );
    assert_eq!(
        names(&pick(&ws, &[], &["web/seo"], &[]).unwrap()),
        ["schema"]
    );
    assert_eq!(
        names(&pick(&ws, &[], &[], &["web-site"]).unwrap()),
        ["astro", "coding", "schema"]
    );
    assert_eq!(
        names(&pick(&ws, &["tmux"], &[], &["web-site", "tiny"]).unwrap()),
        ["astro", "coding", "schema", "tmux"]
    );
}

#[test]
fn unknown_names_and_empty_selections_are_hard_errors() {
    let (_tree, ws) = world();
    assert!(matches!(
        pick(&ws, &["ghost"], &[], &[]),
        Err(SelectError::UnknownSkill { .. })
    ));
    assert!(matches!(
        pick(&ws, &[], &["mobile"], &[]),
        Err(SelectError::UnknownGroup { .. })
    ));
    assert!(matches!(
        pick(&ws, &[], &[], &["nope"]),
        Err(SelectError::UnknownProfile { .. })
    ));
    assert!(matches!(pick(&ws, &[], &[], &[]), Err(SelectError::Empty)));
    assert!(matches!(
        resolve_names(&ws.vault, &["ghost".to_string()]),
        Err(SelectError::UnknownSkill { .. })
    ));
}

#[test]
fn check_new_accepts_absent_or_empty_directories_only() {
    let tree = TempTree::new();
    assert!(check_new(&tree.path().join("absent")).is_ok());
    std::fs::create_dir(tree.path().join("empty")).unwrap();
    assert!(check_new(&tree.path().join("empty")).is_ok());
    tree.write("full/file", "x");
    assert!(matches!(
        check_new(&tree.path().join("full")),
        Err(ProjectError::NotEmpty { .. })
    ));
    assert!(matches!(
        check_new(&tree.path().join("full/file")),
        Err(ProjectError::NotADirectory { .. })
    ));
}

#[test]
fn check_existing_refuses_missing_projects_and_linked_agent_directories() {
    let tree = TempTree::new();
    assert!(matches!(
        check_existing(&tree.path().join("nope")),
        Err(ProjectError::NotFound { .. })
    ));
    std::fs::create_dir_all(tree.path().join("ok")).unwrap();
    assert!(check_existing(&tree.path().join("ok")).is_ok());
    std::fs::create_dir_all(tree.path().join("linked/.agents")).unwrap();
    std::fs::create_dir_all(tree.path().join("elsewhere")).unwrap();
    std::os::unix::fs::symlink(
        tree.path().join("elsewhere"),
        tree.path().join("linked/.agents/skills"),
    )
    .unwrap();
    assert!(matches!(
        check_existing(&tree.path().join("linked")),
        Err(ProjectError::LayoutNotReal { .. })
    ));
}

#[test]
fn create_new_makes_the_directory_and_optionally_a_git_repository() {
    let tree = TempTree::new();
    create_new(&tree.path().join("a/b"), false).unwrap();
    assert!(tree.path().join("a/b").is_dir() && !tree.path().join("a/b/.git").exists());
    create_new(&tree.path().join("c"), true).unwrap();
    assert!(tree.path().join("c/.git").exists());
    assert!(
        create_new(&tree.path().join("c"), true).is_err(),
        "a repository is not empty"
    );
}

#[test]
fn install_into_a_project_without_skills_dir_creates_it_and_is_idempotent() {
    let (tree, ws) = world();
    let project = tree.path().join("projects/new");
    std::fs::create_dir_all(&project).unwrap();
    let skills = pick(&ws, &[], &[], &["web-site"]).unwrap();

    let plan = plan_install(&ws, &project, &skills);
    assert_eq!(plan.counts().create, 3);
    assert!(!project.join(".agents").exists(), "planning writes nothing");
    let done = apply(plan, ApplyOptions::default(), &ignore_events).unwrap();
    assert!(
        done.applied
            .iter()
            .all(|a| matches!(a.outcome, Outcome::Created))
    );
    assert_eq!(
        std::fs::read_to_string(project.join(".agents/skills/astro/SKILL.md")).unwrap(),
        "astro"
    );
    assert!(project.join(".agents/skills/schema/SKILL.md").exists());
    assert!(
        !project.join(".agents/skills/web").exists(),
        "groups never appear in the project"
    );

    assert_eq!(plan_install(&ws, &project, &skills).counts().unchanged, 3);
}
