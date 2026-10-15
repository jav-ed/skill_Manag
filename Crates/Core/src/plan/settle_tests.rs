//! A snapshot of a folder that was just written to must still tell a later edit apart from the folder.

use std::time::{Duration, SystemTime};

use super::inspect::{walk, walk_settled};
use crate::testutil::TempTree;

fn set_mtime(path: &std::path::Path, time: SystemTime) {
    let file = std::fs::OpenOptions::new().write(true).open(path).unwrap();
    file.set_modified(time).unwrap();
}

#[test]
fn an_edit_that_keeps_the_size_and_restores_the_mtime_is_never_hidden_by_a_coarse_clock() {
    // The clock of the files moves every few milliseconds, so one try proves little: repeat.
    for round in 0..15 {
        let tree = TempTree::new();
        tree.write("skill/SKILL.md", "OLD1\n");
        let file = tree.path().join("skill/SKILL.md");
        let mtime = std::fs::metadata(&file).unwrap().modified().unwrap();

        let planned = walk_settled(&tree.path().join("skill")).unwrap();
        std::fs::write(&file, "EDIT\n").unwrap();
        set_mtime(&file, mtime);

        let later = walk(&tree.path().join("skill")).unwrap();
        assert_ne!(later, planned, "round {round}: the edit left no trace");
    }
}

#[test]
fn a_folder_that_was_written_a_moment_ago_is_waited_for_and_an_old_one_is_not() {
    let tree = TempTree::new();
    tree.write("fresh/SKILL.md", "x\n");
    let started = std::time::Instant::now();
    walk_settled(&tree.path().join("fresh")).unwrap();
    assert!(
        started.elapsed() >= Duration::from_millis(10),
        "a file written just now needs a quiet moment first"
    );

    let file = tree.path().join("fresh/SKILL.md");
    // Same files, but the change time can only be moved by a change: wait it out, then nothing is left to wait for.
    std::thread::sleep(Duration::from_millis(30));
    let started = std::time::Instant::now();
    walk_settled(&tree.path().join("fresh")).unwrap();
    assert!(
        started.elapsed() < Duration::from_millis(10),
        "{} was quiet already",
        file.display()
    );
}
