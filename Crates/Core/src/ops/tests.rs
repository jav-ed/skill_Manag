use super::*;
use crate::apply::{ApplyOptions, Outcome, apply};
use crate::config::{Dirs, EnvOverrides, Flags, Settings};
use crate::events::ignore_events;
use crate::plan::PlanKind;
use crate::testutil::TempTree;

/// A vault of three skills (committed), a scan root with projects, and a config naming mandatory skills.
struct World {
    tree: TempTree,
}

impl World {
    fn new(mandatory: &str) -> Self {
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
            ("projects/one/.agents/skills/coding/SKILL.md", "coding v1"),
            ("projects/one/.agents/skills/astro/SKILL.md", "astro v1"),
            ("projects/one/.agents/skills/local-only/SKILL.md", "mine"),
            ("projects/two/.agents/skills/coding/SKILL.md", "coding v1"),
            ("projects/empty/.agents/skills/.keep", ""),
            ("projects/no-skills/src/main.rs", ""),
        ]);
        tree.git_init_commit("vault");
        Self { tree }
    }

    fn workspace(&self) -> Workspace {
        let flags = Flags {
            vault: Some(self.tree.path().join("vault")),
            root: None,
        };
        let dirs = Dirs::under(&self.tree.path().join("home"));
        Workspace::open(Settings::load(&flags, &EnvOverrides::default(), &dirs).unwrap()).unwrap()
    }

    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.tree.path().join(rel)).unwrap()
    }
}

#[test]
fn sync_updates_only_what_a_project_already_has() {
    let world = World::new("");
    let ws = world.workspace();
    let report = ws.scan(&ignore_events).unwrap();
    let plan = plan_sync(&ws, &report);
    assert_eq!(plan.entries.len(), 3);
    let done = apply(plan, ApplyOptions::default(), &ignore_events).unwrap();
    assert!(
        done.applied
            .iter()
            .all(|a| matches!(a.outcome, Outcome::Updated))
    );
    assert_eq!(
        world.read("projects/one/.agents/skills/coding/SKILL.md"),
        "coding v2"
    );
    assert_eq!(
        world.read("projects/one/.agents/skills/astro/SKILL.md"),
        "astro v2"
    );
    assert_eq!(
        world.read("projects/one/.agents/skills/astro/ref.md"),
        "ref"
    );
    assert_eq!(
        world.read("projects/one/.agents/skills/local-only/SKILL.md"),
        "mine"
    );
    assert!(
        !world
            .tree
            .path()
            .join("projects/two/.agents/skills/tmux")
            .exists()
    );
    assert!(
        !world
            .tree
            .path()
            .join("projects/empty/.agents/skills/coding")
            .exists()
    );
}

#[test]
fn push_installs_mandatory_skills_everywhere_including_empty_skills_dirs() {
    let world = World::new("coding, tmux");
    let ws = world.workspace();
    let report = ws.scan(&ignore_events).unwrap();
    let plan = plan_push(&ws, &report).unwrap();
    assert_eq!(plan.entries.len(), 6);
    let counts = plan.counts();
    assert_eq!((counts.create, counts.update), (4, 2));
    apply(plan, ApplyOptions::default(), &ignore_events).unwrap();
    assert_eq!(
        world.read("projects/empty/.agents/skills/tmux/SKILL.md"),
        "tmux v2"
    );
    assert_eq!(
        world.read("projects/two/.agents/skills/coding/SKILL.md"),
        "coding v2"
    );
    assert!(
        !world
            .tree
            .path()
            .join("projects/no-skills/.agents")
            .exists()
    );
}

#[test]
fn push_with_an_unknown_mandatory_name_is_a_hard_error() {
    let world = World::new("coding, ghost");
    let ws = world.workspace();
    let report = ws.scan(&ignore_events).unwrap();
    let err = plan_push(&ws, &report).unwrap_err();
    assert!(err.to_string().contains("ghost"));
}

#[test]
fn list_shows_every_installed_folder_and_marks_vault_membership() {
    let world = World::new("");
    let ws = world.workspace();
    let report = ws.scan(&ignore_events).unwrap();
    let set = installed(&report, Some(&ws.vault));
    let rows: Vec<_> = set
        .rows
        .iter()
        .map(|r| (r.target.skill.as_str(), r.in_vault))
        .collect();
    assert_eq!(
        rows,
        [
            ("astro", Some(true)),
            ("coding", Some(true)),
            ("local-only", Some(false)),
            ("coding", Some(true))
        ]
    );
    assert!(
        installed(&report, None)
            .rows
            .iter()
            .all(|r| r.in_vault.is_none())
    );
}

#[test]
fn delete_by_name_removes_only_that_folder_and_dry_run_touches_nothing() {
    let world = World::new("");
    let ws = world.workspace();
    let report = ws.scan(&ignore_events).unwrap();
    let targets = targets_named(&report, "coding").unwrap().targets;
    assert_eq!(targets.len(), 2);

    let dry = delete(targets.clone(), true, None, &ignore_events);
    assert_eq!(dry.failed(), 0);
    assert!(
        world
            .tree
            .path()
            .join("projects/one/.agents/skills/coding")
            .exists()
    );

    let real = delete(targets, false, None, &ignore_events);
    assert_eq!(real.failed(), 0);
    assert!(
        !world
            .tree
            .path()
            .join("projects/one/.agents/skills/coding")
            .exists()
    );
    assert!(
        world
            .tree
            .path()
            .join("projects/one/.agents/skills/astro")
            .exists()
    );
    assert!(
        world
            .tree
            .path()
            .join("projects/one/.agents/skills")
            .exists()
    );
}

#[test]
fn delete_rejects_names_that_could_escape_the_skills_directory() {
    let world = World::new("");
    for bad in ["..", ".", "", "a/b", "coding/../../..", ".agents", "x\0y"] {
        assert!(
            matches!(validate_name(bad), Err(DeleteError::InvalidName { .. })),
            "{bad:?}"
        );
    }
    let project = world.tree.path().join("projects/one");
    assert!(target_in_project(&project, "..").is_err());
    assert!(matches!(
        target_in_project(&project, "nope"),
        Err(DeleteError::NotInstalled { .. })
    ));
    assert!(target_in_project(&project, "coding").is_ok());
    assert!(project.join(".agents").exists());
}

#[test]
fn deleting_a_symlinked_skill_removes_the_link_not_its_target() {
    let world = World::new("");
    let project = world.tree.path().join("projects/one");
    let outside = world.tree.write("outside/keep.md", "precious");
    std::os::unix::fs::symlink(
        world.tree.path().join("outside"),
        project.join(".agents/skills/linked"),
    )
    .unwrap();
    let target = crate::scan::Target {
        project: project.clone(),
        skill: "linked".into(),
        path: project.join(".agents/skills/linked"),
    };
    assert_eq!(
        delete(vec![target], false, None, &ignore_events).failed(),
        0
    );
    assert!(outside.exists());
    assert!(!project.join(".agents/skills/linked").exists());
}

#[test]
fn a_second_sync_is_a_no_op() {
    let world = World::new("");
    let ws = world.workspace();
    let report = ws.scan(&ignore_events).unwrap();
    apply(
        plan_sync(&ws, &report),
        ApplyOptions::default(),
        &ignore_events,
    )
    .unwrap();
    let again = plan_sync(&ws, &report);
    assert!(
        again
            .entries
            .iter()
            .all(|e| e.result.as_ref().unwrap().kind == PlanKind::Unchanged)
    );
}
