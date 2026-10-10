//! `doctor` and the text of the AGENTS.md blocks: the vault's own file, the built-in rules, the folder.

use super::doctor_tests::{about, healthy, of, run, single, skill_md};
use super::*;
use crate::testutil::TempTree;

fn commit(tree: &TempTree, message: &str) {
    tree.git(".", &["-C", "vault", "add", "-A"]);
    tree.git(".", &["-C", "vault", "commit", "-qm", message]);
}

#[test]
fn a_text_the_vault_cannot_use_is_an_error_that_names_the_file_and_the_consequence() {
    let tree = healthy();
    tree.write("vault/project-files/AGENTS.md", "  \n");
    commit(&tree, "empty text");

    let report = run(&tree);

    let finding = single(&report, "agents-text");
    assert_eq!(finding.severity, Severity::Error);
    assert!(
        about(
            finding,
            &tree
                .path()
                .join("vault/project-files/AGENTS.md")
                .display()
                .to_string()
        ),
        "{finding:?}"
    );
    assert!(finding.message.contains("is empty"), "{}", finding.message);
    assert!(finding.message.contains("`init`"), "{}", finding.message);
    assert!(finding.hint.as_deref().unwrap().contains("fix "));
}

#[test]
fn a_usable_text_of_the_vaults_own_says_nothing_even_when_the_three_skills_are_missing() {
    let tree = bare();
    tree.write("vault/project-files/AGENTS.md", "# Mine\n");
    commit(&tree, "text");

    let report = run(&tree);

    assert!(
        of(&report, "agents-text").is_empty(),
        "{:?}",
        report.findings
    );
    assert!(report.checks.contains(&"agents-text"));
}

#[test]
fn the_built_in_text_needs_its_skills_and_doctor_names_each_one_that_is_missing() {
    let tree = bare();
    tree.write("vault/doc-start/SKILL.md", &skill_md("doc-start"));
    commit(&tree, "doc-start");

    let report = run(&tree);

    let finding = single(&report, "agents-text");
    assert_eq!(finding.severity, Severity::Warning);
    assert!(
        finding.message.contains("`file-tree-optimization`"),
        "{}",
        finding.message
    );
    assert!(!finding.message.contains("`doc-start`"));
    assert!(finding.message.contains("`init` stops"));
}

#[test]
fn the_folder_of_the_text_is_not_noted_as_holding_no_skill_but_other_empty_folders_are() {
    let tree = healthy();
    tree.write("vault/project-files/AGENTS.md", "# Mine\n");
    tree.write("vault/notes/readme.md", "x");
    commit(&tree, "folders");

    let report = run(&tree);

    let noted: Vec<&str> = of(&report, "vault")
        .iter()
        .filter_map(|f| f.subject.as_deref())
        .collect();
    assert!(noted.iter().any(|s| s.ends_with("/notes")), "{noted:?}");
    assert!(
        !noted.iter().any(|s| s.ends_with("project-files")),
        "{noted:?}"
    );
}

/// A committed vault with `coding` only: none of the other skills the built-in text names.
fn bare() -> TempTree {
    let tree = TempTree::new();
    tree.write("vault/coding/SKILL.md", &skill_md("coding"));
    tree.write(
        "vault/config.yaml",
        &format!(
            "root: {}/projects\nmandatory: [coding]\n",
            tree.path().display()
        ),
    );
    tree.write(
        "projects/one/.agents/skills/coding/SKILL.md",
        &skill_md("coding"),
    );
    tree.git_init_commit("vault");
    tree
}
