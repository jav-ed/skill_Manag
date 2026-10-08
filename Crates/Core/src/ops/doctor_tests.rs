//! `doctor`: a healthy world says nothing, and each broken thing is named by its own check.

use std::os::unix::fs::symlink;

use super::*;
use crate::config::{ConfigError, Dirs, EnvOverrides, Flags};
use crate::testutil::TempTree;

const HEADER: &str = "---\nname: NAME\ndescription: Use it for things.\n---\nBody\n";

fn skill_md(name: &str) -> String {
    HEADER.replace("NAME", name)
}

/// A committed vault with two good skills and a scan root with one project.
fn healthy() -> TempTree {
    let tree = TempTree::new();
    tree.write_all(&[
        ("vault/coding/SKILL.md", &skill_md("coding")),
        ("vault/web/astro/SKILL.md", &skill_md("astro")),
        (
            "vault/config.yaml",
            &format!(
                "root: {}/projects\nmandatory: [coding]\n",
                tree.path().display()
            ),
        ),
        (
            "projects/one/.agents/skills/coding/SKILL.md",
            &skill_md("coding"),
        ),
    ]);
    tree.git_init_commit("vault");
    tree
}

fn run_with(tree: &TempTree, flags: &Flags) -> DoctorReport {
    let dirs = Dirs::under(&tree.path().join("home"));
    doctor(flags, Ok(EnvOverrides::default()), &dirs)
}

fn run(tree: &TempTree) -> DoctorReport {
    run_with(
        tree,
        &Flags {
            vault: Some(tree.path().join("vault")),
            root: None,
        },
    )
}

fn of<'a>(report: &'a DoctorReport, check: &str) -> Vec<&'a Finding> {
    report
        .findings
        .iter()
        .filter(|f| f.check == check)
        .collect()
}

fn single<'a>(report: &'a DoctorReport, check: &str) -> &'a Finding {
    let found = of(report, check);
    assert_eq!(found.len(), 1, "{check}: {:?}", report.findings);
    found[0]
}

fn about(finding: &Finding, subject: &str) -> bool {
    finding.subject.as_deref() == Some(subject)
}

#[test]
fn a_healthy_world_has_no_warning_and_no_error() {
    let tree = healthy();

    let report = run(&tree);

    assert_eq!(report.worst(), None, "{:?}", report.findings);
    for check in [
        "config",
        "vault",
        "vault-git",
        "skill-files",
        "skill-edits",
        "mandatory",
        "profiles",
        "scan",
        "leftovers",
        "filesystem",
        "git",
        "pointer",
        "backups",
    ] {
        assert!(
            report.checks.contains(&check),
            "{check} did not run: {:?}",
            report.checks
        );
    }
}

#[test]
fn a_header_with_an_unquoted_colon_is_reported_as_invalid_yaml() {
    let tree = healthy();
    tree.write(
        "vault/coding/SKILL.md",
        "---\nname: coding\ndescription: Use when: the user codes\n---\n",
    );
    tree.git(".", &["-C", "vault", "commit", "-aqm", "colon"]);

    let report = run(&tree);

    let finding = single(&report, "skill-files");
    assert_eq!(finding.severity, Severity::Warning);
    assert!(about(finding, "coding"));
    assert!(
        finding.message.contains("not valid YAML"),
        "{}",
        finding.message
    );
}

#[test]
fn a_name_that_is_not_the_folder_and_a_missing_description_are_reported() {
    let tree = healthy();
    tree.write("vault/coding/SKILL.md", "---\nname: coder\n---\nBody\n");
    tree.write("vault/web/astro/SKILL.md", "no header at all\n");
    tree.git(".", &["-C", "vault", "commit", "-aqm", "headers"]);

    let report = run(&tree);

    let coding: Vec<_> = of(&report, "skill-files")
        .into_iter()
        .filter(|f| about(f, "coding"))
        .collect();
    assert_eq!(coding.len(), 2, "{coding:?}");
    assert!(coding.iter().any(|f| f.message.contains("\"coder\"")));
    assert!(coding.iter().any(|f| f.message.contains("no description")));
    let astro: Vec<_> = of(&report, "skill-files")
        .into_iter()
        .filter(|f| about(f, "astro"))
        .collect();
    assert_eq!(astro.len(), 1);
    assert!(
        astro[0].message.contains("does not start with"),
        "{}",
        astro[0].message
    );
}

#[test]
fn a_header_that_is_never_closed_is_reported() {
    let tree = healthy();
    tree.write(
        "vault/coding/SKILL.md",
        "---\nname: coding\ndescription: x\n",
    );
    tree.git(".", &["-C", "vault", "commit", "-aqm", "open"]);

    let finding = single(&run(&tree), "skill-files").clone();

    assert!(
        finding.message.contains("never closed"),
        "{}",
        finding.message
    );
}

#[test]
fn a_skill_md_that_git_does_not_track_is_an_error() {
    let tree = healthy();
    tree.write("vault/tmux/SKILL.md", &skill_md("tmux"));

    let report = run(&tree);

    let finding = of(&report, "skill-files")
        .into_iter()
        .find(|f| about(f, "tmux"))
        .unwrap();
    assert_eq!(finding.severity, Severity::Error);
    assert!(
        finding.message.contains("not tracked"),
        "{}",
        finding.message
    );
    assert_eq!(report.worst(), Some(Severity::Error));
}

#[test]
fn edits_untracked_files_and_missing_files_are_told_apart() {
    let tree = healthy();
    tree.write("vault/web/astro/ref.md", "tracked\n");
    tree.git(".", &["-C", "vault", "add", "-A"]);
    tree.git(".", &["-C", "vault", "commit", "-qm", "ref"]);
    // Now, with everything committed: a new file, an edit, and a tracked file that disappears.
    tree.write("vault/coding/notes.md", "new, not added\n");
    tree.write(
        "vault/coding/SKILL.md",
        &format!("{}more\n", skill_md("coding")),
    );
    std::fs::remove_file(tree.path().join("vault/web/astro/ref.md")).unwrap();

    let report = run(&tree);

    let edits = of(&report, "skill-edits");
    let coding_untracked = edits
        .iter()
        .find(|f| about(f, "coding") && f.severity == Severity::Warning)
        .unwrap();
    assert!(
        coding_untracked.message.contains("not tracked"),
        "{}",
        coding_untracked.message
    );
    let coding_edited = edits
        .iter()
        .find(|f| about(f, "coding") && f.severity == Severity::Note)
        .unwrap();
    assert!(
        coding_edited.message.contains("already copies"),
        "{}",
        coding_edited.message
    );
    let astro = edits.iter().find(|f| about(f, "astro")).unwrap();
    assert_eq!(astro.severity, Severity::Error);
    assert!(astro.message.contains("missing"), "{}", astro.message);
}

#[test]
fn a_link_and_an_empty_folder_in_the_vault_are_named() {
    let tree = healthy();
    symlink(
        tree.path().join("vault/coding"),
        tree.path().join("vault/alias"),
    )
    .unwrap();
    std::fs::create_dir_all(tree.path().join("vault/empty-folder")).unwrap();

    let report = run(&tree);

    let found = of(&report, "vault");
    let link = found
        .iter()
        .find(|f| f.subject.as_deref().is_some_and(|s| s.ends_with("alias")))
        .unwrap();
    assert_eq!(link.severity, Severity::Warning);
    let empty = found
        .iter()
        .find(|f| {
            f.subject
                .as_deref()
                .is_some_and(|s| s.ends_with("empty-folder"))
        })
        .unwrap();
    assert_eq!(empty.severity, Severity::Note);
}

#[test]
fn an_unknown_mandatory_name_and_broken_profiles_are_errors() {
    let tree = healthy();
    tree.write(
        "vault/config.yaml",
        &format!(
            "root: {}/projects\nmandatory: [coding, ghost]\nprofiles:\n  broken:\n    skills: [nope]\n  nothing:\n    exclude: [coding]\n",
            tree.path().display()
        ),
    );

    let report = run(&tree);

    let mandatory = single(&report, "mandatory");
    assert_eq!(mandatory.severity, Severity::Error);
    assert!(about(mandatory, "ghost"));
    let profiles = of(&report, "profiles");
    let broken = profiles.iter().find(|f| about(f, "broken")).unwrap();
    assert_eq!(broken.severity, Severity::Error);
    assert!(broken.message.contains("nope"), "{}", broken.message);
    let nothing = profiles.iter().find(|f| about(f, "nothing")).unwrap();
    assert_eq!(nothing.severity, Severity::Warning);
}

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

fn listing(dir: &std::path::Path) -> Vec<(String, u64)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(next) = stack.pop() {
        for entry in std::fs::read_dir(&next).unwrap().flatten() {
            let path = entry.path();
            // git's own bookkeeping may refresh its index when it reads the vault; the files are what count
            if path.ends_with(".git") {
                continue;
            }
            let meta = std::fs::symlink_metadata(&path).unwrap();
            out.push((path.display().to_string(), meta.len()));
            if meta.is_dir() {
                stack.push(path);
            }
        }
    }
    out.sort();
    out
}
