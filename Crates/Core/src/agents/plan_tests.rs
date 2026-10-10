use std::path::PathBuf;

use super::block::{render, with_endings};
use super::plan::{Action, AgentsEntry, AgentsPlan, Intent, plan_agents};
use super::text::{Source, load_source};
use crate::testutil::TempTree;

/// A source whose text is `text`, read the way a vault file is.
fn source(text: &str) -> Source {
    let vault = TempTree::new();
    vault.write("AGENTS.md", text);
    load_source(vault.path()).unwrap()
}

fn plan_for(project: &TempTree, source: &Source, intent: Intent) -> AgentsPlan {
    plan_agents(&[project.path().to_path_buf()], source, intent)
}

fn entry(plan: &AgentsPlan) -> &AgentsEntry {
    plan.entries.first().unwrap()
}

fn after(plan: &AgentsPlan) -> String {
    entry(plan).after.clone().unwrap()
}

#[test]
fn add_creates_the_file_with_the_block_where_there_is_none() {
    let project = TempTree::new();
    let new = source("# Rules\n\n1. one");

    let plan = plan_for(&project, &new, Intent::ADD);

    assert_eq!(entry(&plan).action, Action::Create);
    assert_eq!(after(&plan), render("# Rules\n\n1. one"));
    assert!(
        !project.path().join("AGENTS.md").exists(),
        "planning writes nothing"
    );
}

#[test]
fn add_puts_the_block_in_front_of_a_file_that_has_none_and_keeps_the_rest_byte_for_byte() {
    let project = TempTree::new();
    let mine = "# My project\n\nrules of mine\n\n\ttabs\n";
    project.write("AGENTS.md", mine);

    let plan = plan_for(&project, &source("text"), Intent::ADD);

    assert_eq!(entry(&plan).action, Action::Insert);
    assert_eq!(after(&plan), format!("{}\n{mine}", render("text")));
}

#[test]
fn a_byte_order_mark_stays_first_when_the_block_is_put_in() {
    let project = TempTree::new();
    project.write("AGENTS.md", "\u{feff}# Mine\n");

    let plan = plan_for(&project, &source("text"), Intent::ADD);

    assert_eq!(
        after(&plan),
        format!("\u{feff}{}\n# Mine\n", render("text"))
    );
}

#[test]
fn a_crlf_file_stays_crlf_everywhere_after_an_insert_and_an_update() {
    let project = TempTree::new();
    project.write("AGENTS.md", "# Mine\r\nline\r\n");
    let inserted = after(&plan_for(&project, &source("a\nb"), Intent::ADD));
    assert!(
        !inserted.replace("\r\n", "").contains('\n'),
        "a bare newline in {inserted:?}"
    );
    assert!(inserted.ends_with("# Mine\r\nline\r\n"));

    project.write("AGENTS.md", &inserted);
    let updated = after(&plan_for(&project, &source("a\nb\nc"), Intent::SYNC));
    assert!(
        !updated.replace("\r\n", "").contains('\n'),
        "a bare newline in {updated:?}"
    );
    assert!(updated.contains("a\r\nb\r\nc\r\n"));
    assert!(updated.ends_with("# Mine\r\nline\r\n"));
}

#[test]
fn sync_rewrites_only_the_block_of_an_outdated_file() {
    let project = TempTree::new();
    let around_before = "intro\n\n";
    let around_after = "\n## Mine\nkeep this\n";
    project.write(
        "AGENTS.md",
        &format!("{around_before}{}{around_after}", render("old text")),
    );

    let plan = plan_for(&project, &source("new text"), Intent::SYNC);

    assert_eq!(entry(&plan).action, Action::Update);
    assert_eq!(
        after(&plan),
        format!("{around_before}{}{around_after}", render("new text"))
    );
}

#[test]
fn a_block_at_the_end_of_a_file_without_a_final_newline_stays_without_one() {
    let project = TempTree::new();
    let old = render("old");
    project.write("AGENTS.md", old.trim_end());

    let plan = plan_for(&project, &source("new"), Intent::SYNC);

    assert_eq!(after(&plan), render("new").trim_end());
}

#[test]
fn sync_leaves_files_it_is_not_for_alone_and_says_what_is() {
    let missing = TempTree::new();
    let no_block = TempTree::new();
    no_block.write("AGENTS.md", "mine\n");
    let new = source("new");

    for project in [&missing, &no_block] {
        let plan = plan_for(project, &new, Intent::SYNC);
        assert!(
            matches!(&entry(&plan).action, Action::Skipped(why) if why.contains("agents add")),
            "{:?}",
            entry(&plan).action
        );
        assert!(entry(&plan).after.is_none());
    }
}

#[test]
fn add_leaves_existing_blocks_alone() {
    let current = TempTree::new();
    current.write("AGENTS.md", &render("same"));
    let outdated = TempTree::new();
    outdated.write("AGENTS.md", &render("older"));

    assert_eq!(
        entry(&plan_for(&current, &source("same"), Intent::ADD)).action,
        Action::Unchanged
    );
    assert!(matches!(
        entry(&plan_for(&outdated, &source("same"), Intent::ADD)).action,
        Action::Skipped(why) if why.contains("agents sync")
    ));
}

#[test]
fn an_edited_block_is_a_failure_for_sync_unless_forced() {
    let project = TempTree::new();
    project.write(
        "AGENTS.md",
        &render("older").replace("older", "older, plus mine"),
    );
    let new = source("new");

    let plain = plan_for(&project, &new, Intent::SYNC);
    assert!(matches!(&entry(&plain).action, Action::Failed(why) if why.contains("--force")));
    assert_eq!(plain.failed(), 1);

    let forced = plan_for(
        &project,
        &new,
        Intent {
            force: true,
            ..Intent::SYNC
        },
    );
    assert_eq!(entry(&forced).action, Action::Update);
    assert_eq!(after(&forced), render("new"));
}

#[test]
fn force_does_not_touch_a_block_that_sync_may_not_update() {
    let project = TempTree::new();
    project.write("AGENTS.md", &render("older").replace("older", "edited"));

    let plan = plan_for(
        &project,
        &source("new"),
        Intent {
            force: true,
            ..Intent::ADD
        },
    );

    assert!(matches!(entry(&plan).action, Action::Skipped(_)));
}

#[test]
fn broken_markers_and_unusable_files_fail_for_every_intent() {
    let broken = TempTree::new();
    broken.write("AGENTS.md", "<!-- skillmirror:autogenerated end -->\n");
    let linked = TempTree::new();
    linked.write("real.md", "x");
    std::os::unix::fs::symlink(
        linked.path().join("real.md"),
        linked.path().join("AGENTS.md"),
    )
    .unwrap();
    let new = source("new");

    for project in [&broken, &linked] {
        for intent in [Intent::ADD, Intent::SYNC] {
            let plan = plan_for(project, &new, intent);
            assert!(
                matches!(entry(&plan).action, Action::Failed(_)),
                "{intent:?}"
            );
            assert!(entry(&plan).after.is_none());
        }
    }
}

#[test]
fn a_current_file_is_unchanged_and_has_no_diff() {
    let project = TempTree::new();
    project.write("AGENTS.md", &render("same"));

    let plan = plan_for(&project, &source("same"), Intent::SYNC);

    assert_eq!(entry(&plan).action, Action::Unchanged);
    assert_eq!(plan.changes(), 0);
    assert_eq!(entry(&plan).diff(), "");
}

#[test]
fn the_diff_shows_what_comes_in_and_what_goes() {
    let project = TempTree::new();
    project.write("AGENTS.md", &format!("mine\n{}", render("old line")));

    let diff = entry(&plan_for(&project, &source("new line"), Intent::SYNC)).diff();

    assert!(diff.contains("-old line"), "{diff}");
    assert!(diff.contains("+new line"), "{diff}");
    assert!(!diff.contains("-mine"), "{diff}");
}

#[test]
fn entries_come_back_in_the_order_the_projects_were_given_and_are_counted() {
    let (a, b, c) = (TempTree::new(), TempTree::new(), TempTree::new());
    a.write("AGENTS.md", &render("same"));
    c.write("AGENTS.md", "mine");
    let projects: Vec<PathBuf> = [&a, &b, &c]
        .iter()
        .map(|p| p.path().to_path_buf())
        .collect();

    let plan = plan_agents(&projects, &source("same"), Intent::ADD);

    let order: Vec<&PathBuf> = plan.entries.iter().map(|e| &e.project).collect();
    assert_eq!(order, projects.iter().collect::<Vec<_>>());
    assert_eq!(plan.changes(), 2);
    assert_eq!(plan.count(|a| *a == Action::Unchanged), 1);
    assert_eq!(with_endings("x\n", false), "x\n");
}

#[test]
fn a_create_is_refused_when_the_built_in_text_names_skills_the_project_lacks() {
    let project = TempTree::new();
    let mut plan = plan_for(&project, &Source::builtin(), Intent::ADD);
    assert_eq!(entry(&plan).action, Action::Create);

    plan.refuse_missing_skills(|_| ["coding".to_string()].into());

    assert!(matches!(
        &entry(&plan).action,
        Action::Failed(why) if why.contains("doc-start, file-tree-optimization") && !why.contains('/')
    ));
    assert!(entry(&plan).after.is_none(), "nothing is left to write");
    assert_eq!(plan.changes(), 0);
}

#[test]
fn skills_that_are_there_or_a_vault_text_or_an_entry_that_writes_nothing_are_left_alone() {
    let all: std::collections::BTreeSet<String> = super::BUILTIN_SKILLS
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    let project = TempTree::new();
    let mut ok = plan_for(&project, &Source::builtin(), Intent::ADD);
    ok.refuse_missing_skills(|_| all.clone());
    assert_eq!(entry(&ok).action, Action::Create);

    let mut vault_text = plan_for(&project, &source("mine"), Intent::ADD);
    vault_text.refuse_missing_skills(|_| std::collections::BTreeSet::new());
    assert_eq!(entry(&vault_text).action, Action::Create);

    let current = TempTree::new();
    current.write("AGENTS.md", &render(Source::builtin().text()));
    let mut unchanged = plan_for(&current, &Source::builtin(), Intent::ADD);
    unchanged.refuse_missing_skills(|_| std::collections::BTreeSet::new());
    assert_eq!(entry(&unchanged).action, Action::Unchanged);
}

#[test]
fn installed_skills_lists_real_skill_folders_only() {
    let project = TempTree::new();
    project.write(".agents/skills/coding/SKILL.md", "x");
    project.write(".agents/skills/.stage-1/SKILL.md", "x");
    project.write(".agents/skills/stray-file", "x");

    let have = super::installed_skills(project.path());

    assert_eq!(have.into_iter().collect::<Vec<_>>(), ["coding"]);
    assert!(super::installed_skills(&project.path().join("nope")).is_empty());
}

#[test]
fn an_insert_into_a_file_without_a_block_is_refused_too() {
    let project = TempTree::new();
    project.write("AGENTS.md", "mine\n");
    let mut plan = plan_for(&project, &Source::builtin(), Intent::ADD);
    assert_eq!(entry(&plan).action, Action::Insert);

    plan.refuse_missing_skills(|_| std::collections::BTreeSet::new());

    assert!(matches!(&entry(&plan).action, Action::Failed(_)));
    assert!(entry(&plan).after.is_none());
}
