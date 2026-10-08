//! `doctor` on the machine and the configuration: git, config, pointer, leftovers, backups, the root.

use super::doctor_tests::{about, healthy, listing, run, run_with, single, skill_md};
use super::*;
use crate::config::{ConfigError, Dirs, Flags};
use crate::testutil::TempTree;

#[test]
fn a_vault_that_is_not_a_git_repository_is_an_error_and_ends_the_vault_checks() {
    let tree = TempTree::new();
    tree.write("vault/coding/SKILL.md", &skill_md("coding"));

    let report = run(&tree);

    let finding = single(&report, "vault-git");
    assert_eq!(finding.severity, Severity::Error);
    assert!(!report.checks.contains(&"skill-files"));
}

#[test]
fn a_broken_config_is_a_finding_and_stops_the_checks_that_need_it() {
    let tree = healthy();
    tree.write("vault/config.yaml", "root: [unclosed\n");

    let report = run(&tree);

    let finding = single(&report, "config");
    assert_eq!(finding.severity, Severity::Error);
    assert!(!report.checks.contains(&"vault"), "{:?}", report.checks);
    assert!(
        report.checks.contains(&"git"),
        "the machine checks still ran"
    );
}

#[test]
fn a_legacy_environment_variable_is_a_finding_not_a_crash() {
    let tree = healthy();
    let dirs = Dirs::under(&tree.path().join("home"));
    let env = Err(ConfigError::LegacyEnv {
        vars: vec!["SKILL_MANAG_VAULT".to_string()],
    });

    let report = doctor(&Flags::default(), env, &dirs);

    let finding = single(&report, "config");
    assert!(
        finding.message.contains("SKILL_MANAG_VAULT"),
        "{}",
        finding.message
    );
}

#[test]
fn no_vault_at_all_is_an_error() {
    let tree = TempTree::new();

    let report = run_with(&tree, &Flags::default());

    let finding = single(&report, "vault");
    assert_eq!(finding.severity, Severity::Error);
}

#[test]
fn a_pointer_to_a_missing_folder_and_the_old_tools_pointer_are_reported() {
    let tree = healthy();
    let dirs = Dirs::under(&tree.path().join("home"));
    std::fs::create_dir_all(dirs.config()).unwrap();
    std::fs::write(dirs.pointer_file(), "/does/not/exist\n").unwrap();
    std::fs::create_dir_all(dirs.legacy_pointer_file().parent().unwrap()).unwrap();
    std::fs::write(dirs.legacy_pointer_file(), "/old\n").unwrap();

    let report = run(&tree);

    let pointer = single(&report, "pointer");
    assert_eq!(pointer.severity, Severity::Warning);
    assert!(
        pointer.message.contains("/does/not/exist"),
        "{}",
        pointer.message
    );
    let legacy = single(&report, "legacy");
    assert_eq!(legacy.severity, Severity::Note);
}

#[test]
fn a_leftover_of_an_interrupted_run_is_named() {
    let tree = healthy();
    tree.write(
        "projects/one/.agents/.stage-4294967000-abc-0/SKILL.md",
        "half",
    );

    let report = run_with(
        &tree,
        &Flags {
            vault: Some(tree.path().join("vault")),
            root: Some(tree.path().join("projects")),
        },
    );

    let finding = single(&report, "leftovers");
    assert_eq!(finding.severity, Severity::Warning);
    assert!(
        finding
            .subject
            .as_deref()
            .is_some_and(|s| s.contains(".stage-"))
    );
}

#[test]
fn a_damaged_backup_run_is_reported() {
    let tree = healthy();
    let dirs = Dirs::under(&tree.path().join("home"));
    let run_dir = dirs.state().join("backups/20260101-000000-000-1-0/0");
    std::fs::create_dir_all(&run_dir).unwrap();
    std::fs::write(run_dir.join("entry.json"), "not json").unwrap();

    let report = run(&tree);

    let finding = single(&report, "backups");
    assert_eq!(finding.severity, Severity::Warning);
    assert!(about(finding, "20260101-000000-000-1-0"));
}

#[test]
fn a_missing_scan_root_is_a_warning() {
    let tree = healthy();
    tree.write("vault/config.yaml", "mandatory: [coding]\n");

    let report = run(&tree);

    let finding = single(&report, "scan");
    assert_eq!(finding.severity, Severity::Warning);
}

#[test]
fn filesystems_that_cannot_swap_folders_are_recognised() {
    use super::doctor::unsupported_magic;
    assert_eq!(unsupported_magic(0x6969), Some("NFS"));
    assert_eq!(unsupported_magic(0x6573_5546), Some("a FUSE filesystem"));
    assert_eq!(unsupported_magic(0xEF53), None, "ext4 is fine");
    assert_eq!(unsupported_magic(0x0102_1994), None, "tmpfs is fine");
}

#[test]
fn doctor_writes_nothing() {
    let tree = healthy();
    tree.write("vault/tmux/SKILL.md", "no header");
    let before = listing(tree.path());

    run(&tree);
    run_with(
        &tree,
        &Flags {
            vault: Some(tree.path().join("vault")),
            root: Some(tree.path().join("projects")),
        },
    );

    assert_eq!(listing(tree.path()), before);
}
