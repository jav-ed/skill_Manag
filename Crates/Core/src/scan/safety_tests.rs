//! Scan behaviour around odd names and leftovers of interrupted runs.

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

use super::*;
use crate::testutil::TempTree;

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
    tree.write("p/.agents/.stage-1-abc-0/SKILL.md", "half built");
    tree.write("p/.agents/.trash-1-abc-2/SKILL.md", "half deleted");
    tree.write("q/.agents/skills/ok/SKILL.md", "x");
    let report = scan(tree.path(), &ScanOptions::default()).unwrap();
    assert_eq!(report.skills_dirs.len(), 2);
    let paths: Vec<_> = report
        .issues
        .iter()
        .map(|i| i.path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(paths, [".stage-1-abc-0", ".trash-1-abc-2"]);
    assert!(report.issues[0].message.contains("interrupted"));
}
