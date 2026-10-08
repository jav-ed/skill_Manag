//! Scan behaviour around odd names and leftovers of interrupted runs.

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

use super::*;
use crate::testutil::TempTree;

/// A process id no running process has (above the kernel's maximum).
const DEAD_PID: u32 = 4_294_967_000;

#[test]
fn an_installed_folder_with_a_non_utf8_name_is_reported_not_dropped() {
    let tree = TempTree::new();
    tree.write("p/.agents/skills/ok/SKILL.md", "x");
    let odd = tree
        .path()
        .join("p/.agents/skills")
        .join(OsStr::from_bytes(b"bad\xffname"));
    std::fs::create_dir_all(&odd).unwrap();
    let report = scan(tree.path(), &ScanOptions::default()).unwrap();
    let set = all_targets(&report.skills_dirs);
    assert_eq!(set.targets.len(), 1, "the readable folder is still listed");
    assert_eq!(set.issues.len(), 1, "the odd one is visible");
    assert!(set.issues[0].message.contains("UTF-8"));
}

#[test]
fn stage_and_trash_folders_of_an_interrupted_run_are_reported() {
    let tree = TempTree::new();
    tree.write("p/.agents/skills/ok/SKILL.md", "x");
    tree.write(
        &format!("p/.agents/.stage-{DEAD_PID}-abc-0/SKILL.md"),
        "half built",
    );
    tree.write(
        &format!("p/.agents/.trash-{DEAD_PID}-abc-2/SKILL.md"),
        "half deleted",
    );
    tree.write("q/.agents/skills/ok/SKILL.md", "x");
    let report = scan(tree.path(), &ScanOptions::default()).unwrap();
    assert_eq!(report.skills_dirs.len(), 2);
    let paths: Vec<_> = report
        .issues
        .iter()
        .map(|i| i.path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        paths,
        [
            format!(".stage-{DEAD_PID}-abc-0"),
            format!(".trash-{DEAD_PID}-abc-2")
        ]
    );
    assert!(report.issues[0].message.contains("interrupted"));
}

fn leftover_names(tree: &TempTree) -> Vec<String> {
    scan(tree.path(), &ScanOptions::default())
        .unwrap()
        .issues
        .iter()
        .map(|i| i.path.file_name().unwrap().to_string_lossy().into_owned())
        .collect()
}

#[test]
fn the_stage_folder_of_a_run_that_is_still_going_is_not_a_leftover() {
    let tree = TempTree::new();
    tree.write("p/.agents/skills/ok/SKILL.md", "x");
    // A sync in another terminal builds its copy here right now.
    let live = format!("p/.agents/.stage-{}-abc-0/SKILL.md", std::process::id());
    tree.write(&live, "being built");

    let names = leftover_names(&tree);
    assert!(names.is_empty(), "{names:?}");
}

#[test]
fn a_file_with_a_stage_name_is_the_users_file_not_a_leftover() {
    let tree = TempTree::new();
    tree.write("p/.agents/skills/ok/SKILL.md", "x");
    tree.write("p/.agents/.stage-notes", "my own file");

    let names = leftover_names(&tree);
    assert!(names.is_empty(), "{names:?}");
}

#[test]
fn a_stage_folder_whose_name_names_no_process_is_still_reported() {
    let tree = TempTree::new();
    tree.write("p/.agents/skills/ok/SKILL.md", "x");
    tree.write("p/.agents/.stage-weird/SKILL.md", "half built");

    assert_eq!(leftover_names(&tree), [".stage-weird"]);
}
