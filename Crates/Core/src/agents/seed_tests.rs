use super::AgentsError;
use super::seed::{plan_seed, seed_vault_text};
use super::text::{Origin, Source, load_source};
use crate::testutil::TempTree;

#[test]
fn seeding_writes_the_built_in_text_where_the_vault_reads_it() {
    let vault = TempTree::new();

    let path = seed_vault_text(vault.path()).unwrap();

    assert_eq!(path, vault.path().join("project-files/AGENTS.md"));
    let source = load_source(vault.path()).unwrap();
    assert_eq!(source.text(), Source::builtin().text());
    assert_eq!(source.origin(), &Origin::Vault(path));
    let on_disk = vault.read("project-files/AGENTS.md");
    assert!(
        on_disk.ends_with('\n') && !on_disk.ends_with("\n\n"),
        "the file ends with exactly one newline"
    );
}

#[test]
fn seeding_keeps_what_is_already_in_the_folder() {
    let vault = TempTree::new();
    vault.write("project-files/notes.md", "mine");

    seed_vault_text(vault.path()).unwrap();

    assert_eq!(vault.read("project-files/notes.md"), "mine");
    assert!(vault.path().join("project-files/AGENTS.md").is_file());
}

#[test]
fn seeding_never_touches_a_file_that_is_there() {
    let vault = TempTree::new();
    vault.write("project-files/AGENTS.md", "# My own rules\n");

    let error = seed_vault_text(vault.path()).unwrap_err();

    let AgentsError::Exists { path } = &error else {
        panic!("{error:?}");
    };
    assert_eq!(path, &vault.path().join("project-files/AGENTS.md"));
    assert_eq!(vault.read("project-files/AGENTS.md"), "# My own rules\n");
    assert!(crate::Hint::hint(&error).unwrap().contains("agents sync"));
}

#[test]
fn a_link_at_the_place_counts_as_a_file_that_is_there() {
    let vault = TempTree::new();
    std::fs::create_dir(vault.path().join("project-files")).unwrap();
    std::os::unix::fs::symlink(
        vault.path().join("nowhere.md"),
        vault.path().join("project-files/AGENTS.md"),
    )
    .unwrap();

    assert!(matches!(
        seed_vault_text(vault.path()),
        Err(AgentsError::Exists { .. })
    ));
    assert!(!vault.path().join("nowhere.md").exists());
}

#[test]
fn a_folder_name_taken_by_a_file_is_a_hard_error_and_changes_nothing() {
    let vault = TempTree::new();
    vault.write("project-files", "a file, not a folder");

    assert!(matches!(
        seed_vault_text(vault.path()),
        Err(AgentsError::Io(_))
    ));
    assert_eq!(vault.read("project-files"), "a file, not a folder");
}

#[test]
fn planning_the_seed_writes_nothing() {
    let vault = TempTree::new();

    let path = plan_seed(vault.path()).unwrap();

    assert_eq!(path, vault.path().join("project-files/AGENTS.md"));
    assert!(!vault.path().join("project-files").exists());
}

#[test]
fn planning_the_seed_says_what_seeding_would_say_when_the_file_is_there() {
    let vault = TempTree::new();
    vault.write("project-files/AGENTS.md", "x");

    assert!(matches!(
        plan_seed(vault.path()),
        Err(AgentsError::Exists { .. })
    ));
}
