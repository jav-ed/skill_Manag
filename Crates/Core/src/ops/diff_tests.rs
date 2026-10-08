//! `diff`: the lines a sync would bring in and take away, and nothing written.

use std::os::unix::fs::PermissionsExt;

use super::*;
use crate::config::{Dirs, EnvOverrides, Flags, Settings};
use crate::events::ignore_events;
use crate::testutil::TempTree;

fn world() -> (TempTree, Workspace) {
    let tree = TempTree::new();
    tree.write_all(&[
        ("vault/coding/SKILL.md", "title\nsame\nnew line\n"),
        ("vault/coding/ref.md", "reference\n"),
        ("vault/coding/run.sh", "#!/bin/sh\n"),
        ("vault/coding/img.bin", "BIN\u{0}ARY v2"),
        ("vault/tmux/SKILL.md", "tmux\n"),
        (
            "projects/p1/.agents/skills/coding/SKILL.md",
            "title\nsame\nold line\n",
        ),
        ("projects/p1/.agents/skills/coding/run.sh", "#!/bin/sh\n"),
        (
            "projects/p1/.agents/skills/coding/img.bin",
            "BIN\u{0}ARY v1",
        ),
        ("projects/p1/.agents/skills/coding/stray/notes.md", "mine\n"),
        ("projects/p1/.agents/skills/tmux/SKILL.md", "tmux\n"),
        (
            "projects/p2/.agents/skills/coding/SKILL.md",
            "title\nsame\nnew line\n",
        ),
        ("projects/p2/.agents/skills/coding/ref.md", "reference\n"),
        ("projects/p2/.agents/skills/coding/run.sh", "#!/bin/sh\n"),
        (
            "projects/p2/.agents/skills/coding/img.bin",
            "BIN\u{0}ARY v2",
        ),
    ]);
    std::fs::set_permissions(
        tree.path().join("projects/p2/.agents/skills/coding/run.sh"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    std::fs::set_permissions(
        tree.path().join("vault/coding/run.sh"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    tree.git_init_commit("vault");
    let flags = Flags {
        vault: Some(tree.path().join("vault")),
        root: Some(tree.path().join("projects")),
    };
    let dirs = Dirs::under(&tree.path().join("home"));
    let settings = Settings::load(&flags, &EnvOverrides::default(), &dirs).unwrap();
    (tree, Workspace::open(settings).unwrap())
}

fn run(ws: &Workspace, filter: &DiffFilter) -> DiffReport {
    let report = ws.scan(&ignore_events).unwrap();
    diff(ws, &report, filter)
}

fn file<'a>(skill: &'a SkillDiff, path: &str) -> &'a FileDiff {
    skill
        .files
        .iter()
        .find(|f| f.path.to_str() == Some(path))
        .unwrap_or_else(|| panic!("no file {path} in {:?}", skill.files))
}

#[test]
fn each_kind_of_difference_is_a_file_with_its_lines() {
    let (_tree, ws) = world();

    let report = run(&ws, &DiffFilter::default());

    assert_eq!(
        report.skills.len(),
        1,
        "only the skill that differs: coding in p1"
    );
    let coding = &report.skills[0];
    assert!(coding.target.project.ends_with("p1"));

    let skill_md = file(coding, "SKILL.md");
    assert_eq!(skill_md.kind, DiffKind::Modified);
    assert_eq!((skill_md.added_lines, skill_md.removed_lines), (1, 1));
    assert!(
        skill_md.text.contains("--- a/SKILL.md"),
        "{}",
        skill_md.text
    );
    assert!(
        skill_md.text.contains("+++ b/SKILL.md"),
        "{}",
        skill_md.text
    );
    assert!(skill_md.text.contains("-old line"), "{}", skill_md.text);
    assert!(skill_md.text.contains("+new line"), "{}", skill_md.text);

    let added = file(coding, "ref.md");
    assert_eq!(added.kind, DiffKind::Added);
    assert_eq!((added.added_lines, added.removed_lines), (1, 0));
    assert!(added.text.contains("--- /dev/null"), "{}", added.text);

    let removed = file(coding, "stray/notes.md");
    assert_eq!(removed.kind, DiffKind::Removed);
    assert_eq!((removed.added_lines, removed.removed_lines), (0, 1));
    assert!(removed.text.contains("+++ /dev/null"), "{}", removed.text);

    let mode = file(coding, "run.sh");
    assert_eq!(mode.kind, DiffKind::ModeChanged);
    assert_eq!(mode.modes, Some((0o644, 0o755)));
    assert!(mode.text.is_empty(), "{}", mode.text);

    let binary = file(coding, "img.bin");
    assert_eq!(binary.skipped, Some(Skipped::Binary));
    assert!(binary.text.is_empty(), "{}", binary.text);

    let paths: Vec<_> = coding
        .files
        .iter()
        .map(|f| f.path.to_str().unwrap())
        .collect();
    let mut sorted = paths.clone();
    sorted.sort_unstable();
    assert_eq!(paths, sorted, "files come sorted");
    assert!(
        coding
            .files
            .iter()
            .all(|f| f.path.to_str() != Some("stray")),
        "a folder that goes is shown through its files"
    );
}

#[test]
fn a_skill_filter_and_a_project_filter_narrow_the_report() {
    let (tree, ws) = world();
    std::fs::write(
        tree.path().join("projects/p1/.agents/skills/tmux/SKILL.md"),
        "old tmux\n",
    )
    .unwrap();

    let all = run(&ws, &DiffFilter::default());
    assert_eq!(all.skills.len(), 2);

    let only_tmux = run(
        &ws,
        &DiffFilter {
            skill: Some("tmux".to_string()),
            project: None,
        },
    );
    assert_eq!(only_tmux.skills.len(), 1);
    assert_eq!(only_tmux.skills[0].target.skill, "tmux");

    let other_project = run(
        &ws,
        &DiffFilter {
            skill: None,
            project: Some(tree.path().join("projects/p2")),
        },
    );
    assert!(other_project.skills.is_empty(), "p2 is in sync");
}

#[test]
fn a_big_file_is_listed_but_not_shown() {
    let (tree, ws) = world();
    std::fs::write(
        tree.path()
            .join("projects/p2/.agents/skills/coding/SKILL.md"),
        "x".repeat(2 * 1024 * 1024),
    )
    .unwrap();

    let report = run(&ws, &DiffFilter::default());

    let p2 = report
        .skills
        .iter()
        .find(|s| s.target.project.ends_with("p2"))
        .unwrap();
    assert_eq!(file(p2, "SKILL.md").skipped, Some(Skipped::TooLarge));
}

#[test]
fn diff_writes_nothing() {
    let (tree, ws) = world();
    let before = std::fs::read_to_string(
        tree.path()
            .join("projects/p1/.agents/skills/coding/SKILL.md"),
    )
    .unwrap();

    run(&ws, &DiffFilter::default());

    let after = std::fs::read_to_string(
        tree.path()
            .join("projects/p1/.agents/skills/coding/SKILL.md"),
    )
    .unwrap();
    assert_eq!(before, after);
    assert!(
        !tree.path().join("home").exists(),
        "no state or config written"
    );
}

#[test]
fn a_folder_that_cannot_be_compared_is_reported() {
    let (tree, _) = world();
    // A vault skill whose SKILL.md git does not track: the plan refuses it, and p2 has a copy of it.
    tree.write("vault/ghost/SKILL.md", "not committed\n");
    tree.write("projects/p2/.agents/skills/ghost/SKILL.md", "old\n");
    let flags = Flags {
        vault: Some(tree.path().join("vault")),
        root: Some(tree.path().join("projects")),
    };
    let dirs = Dirs::under(&tree.path().join("home"));
    let ws =
        Workspace::open(Settings::load(&flags, &EnvOverrides::default(), &dirs).unwrap()).unwrap();

    let report = run(&ws, &DiffFilter::default());

    assert_eq!(report.failed.len(), 1);
    assert!(report.failed[0].0.project.ends_with("p2"));
    assert_eq!(report.failed[0].0.skill, "ghost");
}

#[test]
fn the_diff_of_a_made_plan_is_the_diff_of_those_skills() {
    let (_tree, ws) = world();
    let scanned = ws.scan(&ignore_events).unwrap();
    let plan = crate::ops::plan_sync(&ws, &scanned);

    let shown = diff_of_plan(&plan);

    let direct = diff(&ws, &scanned, &DiffFilter::default());
    let name = |skills: &[SkillDiff]| -> Vec<(String, Vec<(String, String)>)> {
        skills
            .iter()
            .map(|s| {
                (
                    s.target.project.display().to_string(),
                    s.files
                        .iter()
                        .map(|f| (f.path.display().to_string(), f.text.clone()))
                        .collect(),
                )
            })
            .collect()
    };
    assert_eq!(name(&shown), name(&direct.skills));
    assert!(!shown.is_empty(), "the world has differences");
}
