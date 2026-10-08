//! The data behind the HTML report: skills, projects, and what each skill is in each project.

use super::*;
use crate::config::{Dirs, EnvOverrides, Flags, Settings};
use crate::events::ignore_events;
use crate::testutil::TempTree;

fn header(name: &str, description: &str) -> String {
    format!("---\nname: {name}\ndescription: {description}\n---\n\nbody of {name}\n")
}

/// A vault with a top-level skill, a skill in a group and a mandatory one; three projects.
fn world() -> (TempTree, Workspace) {
    let tree = TempTree::new();
    tree.write_all(&[
        (
            "vault/coding/SKILL.md",
            &header("coding", "Write code <b>well</b>"),
        ),
        ("vault/coding/ref/style.md", "style v2\n"),
        ("vault/web/astro/SKILL.md", &header("astro", "Astro sites")),
        ("vault/tmux/SKILL.md", &header("tmux", "Terminals")),
        ("vault/broken/SKILL.md", "no header at all\n"),
        (
            "vault/config.yaml",
            &format!(
                "root: {}/projects\nmandatory: [tmux]\ntargets: [claude]\n",
                tree.path().display()
            ),
        ),
        // a: coding current, tmux current
        (
            "projects/a/.agents/skills/coding/SKILL.md",
            &header("coding", "Write code <b>well</b>"),
        ),
        (
            "projects/a/.agents/skills/coding/ref/style.md",
            "style v2\n",
        ),
        (
            "projects/a/.agents/skills/tmux/SKILL.md",
            &header("tmux", "Terminals"),
        ),
        // b: astro outdated, a skill the vault lacks, no tmux
        ("projects/b/.agents/skills/astro/SKILL.md", "old astro\n"),
        ("projects/b/.agents/skills/local-only/SKILL.md", "mine\n"),
        // b keeps its own Claude skills in a real folder, so the link cannot be made there
        ("projects/b/.claude/skills/own/SKILL.md", "own\n"),
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

fn data() -> (TempTree, ReportData) {
    let (tree, ws) = world();
    let scanned = ws.scan(&ignore_events).unwrap();
    let data = report_data(&ws, &scanned).unwrap();
    (tree, data)
}

fn project(data: &ReportData, name: &str) -> usize {
    data.projects
        .iter()
        .position(|p| p.ends_with(name))
        .unwrap_or_else(|| panic!("no project {name}: {:?}", data.projects))
}

#[test]
fn skills_come_with_group_description_files_and_the_mandatory_mark() {
    let (_tree, data) = data();

    let names: Vec<&str> = data.skills.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        ["broken", "coding", "tmux", "astro"],
        "top level first, then groups"
    );
    let coding = data.skills.iter().find(|s| s.name == "coding").unwrap();
    assert_eq!(
        coding.description.as_deref(),
        Some("Write code <b>well</b>")
    );
    assert_eq!(coding.files, ["SKILL.md", "ref/style.md"]);
    assert!(!coding.mandatory);
    assert!(
        data.skills
            .iter()
            .find(|s| s.name == "tmux")
            .unwrap()
            .mandatory
    );
    let astro = data.skills.iter().find(|s| s.name == "astro").unwrap();
    assert_eq!(astro.group, ["web"]);
    let broken = data.skills.iter().find(|s| s.name == "broken").unwrap();
    assert!(broken.description.is_none());
    assert!(
        broken
            .header_problem
            .as_deref()
            .is_some_and(|p| p.contains("header")),
        "{:?}",
        broken.header_problem
    );
}

#[test]
fn each_installed_skill_has_a_cell_and_absent_skills_have_none() {
    let (_tree, data) = data();
    let (a, b) = (project(&data, "a"), project(&data, "b"));

    assert!(matches!(
        data.cells.get(&("coding".into(), a)),
        Some(Cell::Current)
    ));
    assert!(matches!(
        data.cells.get(&("tmux".into(), a)),
        Some(Cell::Current)
    ));
    assert!(matches!(
        data.cells.get(&("tmux".into(), b)),
        Some(Cell::MissingMandatory)
    ));
    assert!(matches!(
        data.cells.get(&("local-only".into(), b)),
        Some(Cell::NotInVault)
    ));
    assert!(
        !data.cells.contains_key(&("coding".into(), b)),
        "not installed, not mandatory: no cell"
    );
    let Some(Cell::Outdated(files)) = data.cells.get(&("astro".into(), b)) else {
        panic!(
            "astro in b is outdated: {:?}",
            data.cells.get(&("astro".into(), b))
        );
    };
    assert!(
        files.iter().any(|f| f.text.contains('+')),
        "the diff is there"
    );
    assert_eq!(data.extra, ["local-only"]);
}

#[test]
fn projects_are_listed_in_walk_order_with_the_vault_and_root() {
    let (tree, data) = data();

    assert_eq!(data.projects.len(), 2);
    assert!(data.projects[0].ends_with("a"));
    assert_eq!(data.vault, tree.path().join("vault"));
    assert_eq!(data.root, tree.path().join("projects"));
}

#[test]
fn building_the_data_writes_nothing() {
    let (tree, ws) = world();
    let scanned = ws.scan(&ignore_events).unwrap();
    let listing = |root: &std::path::Path| -> Vec<String> {
        let mut out = Vec::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(dir).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.file_name().is_some_and(|n| n == ".git") {
                    continue;
                }
                out.push(format!(
                    "{} {}",
                    path.display(),
                    entry.metadata().unwrap().len()
                ));
                if path.is_dir() {
                    stack.push(path);
                }
            }
        }
        out.sort();
        out
    };
    let before = listing(tree.path());

    report_data(&ws, &scanned).unwrap();

    assert_eq!(listing(tree.path()), before);
}

#[test]
fn links_that_are_missing_or_in_the_way_are_told_apart_from_skill_problems() {
    let (_tree, data) = data();
    let (a, b) = (project(&data, "a"), project(&data, "b"));

    assert_eq!(data.missing_bridges, [(a, "claude".to_string())]);
    assert_eq!(
        data.project_problems.len(),
        1,
        "{:?}",
        data.project_problems
    );
    assert_eq!(data.project_problems[0].0, b);
    assert!(data.project_problems[0].1.contains("real folder"));
    assert!(
        data.cells
            .keys()
            .all(|(skill, _)| !skill.starts_with("bridge ")),
        "a link problem is not a cell of a skill"
    );
}
