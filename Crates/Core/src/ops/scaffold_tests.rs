//! The project scaffold is the doc-start skill's script: it is checked before anything is made and run after.

use std::collections::BTreeSet;

use super::*;
use crate::config::{Dirs, EnvOverrides, Flags, Settings};
use crate::testutil::TempTree;

const FILES: [&str; 3] = [
    "Code/bootstrap.sh",
    "Code/install_Scratch.sh",
    "Templates/clean_Scratch.sh",
];

fn world(skip: Option<&str>) -> (TempTree, Workspace) {
    let tree = TempTree::new();
    tree.write("vault/coding/SKILL.md", "coding");
    tree.write("vault/doc-start/SKILL.md", "doc-start");
    for file in FILES.iter().filter(|f| Some(**f) != skip) {
        tree.write(&format!("vault/doc-start/{file}"), "#!/usr/bin/env bash\n");
    }
    tree.write("vault/config.yaml", "mandatory: [coding]\n");
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
    (tree, Workspace::open(settings).unwrap())
}

fn set(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|n| (*n).to_string()).collect()
}

#[test]
fn the_scaffold_needs_the_skill_in_the_install_and_the_files_it_runs_in_the_vault() {
    let (_tree, ws) = world(None);
    assert!(require_scaffold(&ws, &set(&["coding", "doc-start"])).is_ok());
    assert!(matches!(
        require_scaffold(&ws, &set(&["coding"])),
        Err(ProjectError::ScaffoldNeedsSkill)
    ));

    for missing in FILES {
        let (_tree, ws) = world(Some(missing));
        let err = require_scaffold(&ws, &set(&["doc-start"])).unwrap_err();
        assert!(
            matches!(&err, ProjectError::ScaffoldFileMissing { file } if file == missing),
            "{missing}: {err}"
        );
    }
}

fn project_with_script(tree: &TempTree, script: &str) -> std::path::PathBuf {
    tree.write("p/.agents/skills/doc-start/Code/bootstrap.sh", script);
    tree.path().join("p")
}

#[test]
fn the_script_runs_in_the_project_and_its_lines_come_back() {
    let tree = TempTree::new();
    let project = project_with_script(
        &tree,
        "set -e\ncd \"$1\"\nmkdir -p Scratch/Agent_Tasks\necho '  created: Scratch/'\necho\necho done.\n",
    );

    let lines = run_scaffold(&project).unwrap();

    assert_eq!(lines, ["  created: Scratch/", "done."]);
    assert!(project.join("Scratch/Agent_Tasks").is_dir());
}

#[test]
fn a_script_that_fails_or_is_missing_is_an_error_that_says_what_to_run_again() {
    let tree = TempTree::new();
    let project = project_with_script(&tree, "echo 'no just installed' >&2\nexit 3\n");
    let err = run_scaffold(&project).unwrap_err();
    assert!(
        matches!(&err, ProjectError::ScaffoldRun { detail, .. } if detail.contains("no just installed")),
        "{err}"
    );
    assert!(crate::Hint::hint(&err).unwrap().contains("bootstrap.sh"));

    let bare = tree.path().join("bare");
    std::fs::create_dir(&bare).unwrap();
    assert!(matches!(
        run_scaffold(&bare),
        Err(ProjectError::ScaffoldRun { .. })
    ));
}
