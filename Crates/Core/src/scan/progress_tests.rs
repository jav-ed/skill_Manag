//! The scan reports how far it has come, and reporting never changes what it finds.

use std::sync::Mutex;

use super::*;
use crate::testutil::TempTree;

/// `count` empty folders plus two projects.
fn wide_tree(count: usize) -> TempTree {
    let tree = TempTree::new();
    for n in 0..count {
        tree.write(&format!("many/d{n}/keep"), "x");
    }
    tree.write("a/.agents/skills/coding/SKILL.md", "x");
    tree.write("b/.agents/skills/coding/SKILL.md", "x");
    tree
}

fn counts_of(tree: &TempTree) -> (ScanReport, Vec<ScanCounts>) {
    let seen = Mutex::new(Vec::new());
    let report = scan_with_progress(tree.path(), &ScanOptions::default(), &|counts| {
        seen.lock().unwrap().push(counts);
    })
    .unwrap();
    let mut seen = seen.into_inner().unwrap();
    seen.sort_by_key(|c| c.directories);
    (report, seen)
}

#[test]
fn progress_is_reported_every_so_many_directories() {
    // 800 folders, `many`, the root, and three for each project: 808 in all.
    let tree = wide_tree(800);

    let (_, seen) = counts_of(&tree);

    let steps: Vec<usize> = seen.iter().map(|c| c.directories).collect();
    assert_eq!(steps, [256, 512, 768]);
    assert!(
        seen.iter().all(|c| c.projects <= 2),
        "never more projects than there are: {seen:?}"
    );
}

#[test]
fn a_small_scan_reports_nothing_and_a_watched_scan_finds_the_same() {
    let small = TempTree::new();
    small.write("a/.agents/skills/coding/SKILL.md", "x");
    assert_eq!(counts_of(&small).1, []);

    let tree = wide_tree(300);
    let (watched, _) = counts_of(&tree);
    let plain = scan(tree.path(), &ScanOptions::default()).unwrap();
    assert_eq!(watched.skills_dirs, plain.skills_dirs);
    assert_eq!(watched.issues.len(), plain.issues.len());
}
