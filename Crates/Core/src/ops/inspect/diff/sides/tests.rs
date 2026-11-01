//! The side-by-side rows: which line stands beside which, which words are marked, which targets are one group.
//! Each test names the rule it protects and what it is worth (1 to 10): these run on every `cargo test`.
// A list of one byte range is exactly what several of these tests compare.
#![allow(clippy::single_range_in_vec_init)]

use std::path::PathBuf;

use super::pairs::pair;
use super::rows::rows_of;
use super::words::marks;
use super::*;
use crate::events::ignore_events;
use crate::ops::inspect::diff::tests::{open, world};
use crate::ops::plan_sync;

fn change(row: &SideRow) -> (Option<&SideLine>, Option<&SideLine>) {
    match row {
        SideRow::Change { old, new } => (old.as_ref(), new.as_ref()),
        SideRow::Same { .. } => panic!("expected a changed row, got {row:?}"),
    }
}

// Rule: a line added in front of a changed paragraph must not move every partner down one. Each old line
// stands beside the new line that reads like it; the added line stands alone. 8/10: this is what makes the
// two columns readable at all.
#[test]
fn lines_that_read_alike_stand_together_and_an_added_line_stands_alone() {
    let old = "intro\nalpha beta gamma\nsecond paragraph line\nend\n";
    let new = "intro\nbrand new line here\nalpha beta gamma!\nsecond paragraph line.\nend\n";

    let rows = rows_of(old, new);

    assert_eq!(rows.len(), 5, "{rows:?}");
    assert!(matches!(&rows[0], SideRow::Same { old_no: 1, new_no: 1, text } if text == "intro"));
    let (old_line, new_line) = change(&rows[1]);
    assert!(old_line.is_none(), "nothing stands beside the added line");
    assert_eq!(new_line.unwrap().text, "brand new line here");
    let (old_line, new_line) = change(&rows[2]);
    let (old_line, new_line) = (old_line.unwrap(), new_line.unwrap());
    assert_eq!((old_line.no, new_line.no), (2, 3));
    assert_eq!(old_line.text, "alpha beta gamma");
    assert_eq!(new_line.text, "alpha beta gamma!");
    assert_eq!(new_line.marks, vec![16..17], "only the new `!` is marked");
    assert_eq!(old_line.marks, Vec::<Range<usize>>::new());
    let (old_line, new_line) = change(&rows[3]);
    assert_eq!(old_line.unwrap().text, "second paragraph line");
    assert_eq!(new_line.unwrap().text, "second paragraph line.");
    assert!(matches!(
        &rows[4],
        SideRow::Same {
            old_no: 4,
            new_no: 5,
            ..
        }
    ));
}

// Rule: every line of both sides is on exactly one row, in order, however many lines there are; past the
// size limits the pairing is in order. A line lost here would be a change nobody sees. 7/10.
#[test]
fn every_line_is_paired_exactly_once_and_in_order() {
    let lines =
        |n: usize, tag: &str| -> Vec<String> { (0..n).map(|i| format!("{tag} {i}")).collect() };
    for (old_len, new_len) in [(0, 0), (3, 0), (0, 3), (2, 5), (5, 2), (300, 280)] {
        let (old, new) = (lines(old_len, "old line"), lines(new_len, "new line"));
        let old: Vec<&str> = old.iter().map(String::as_str).collect();
        let new: Vec<&str> = new.iter().map(String::as_str).collect();

        let pairs = pair(&old, &new);

        let olds: Vec<usize> = pairs.iter().filter_map(|p| p.0).collect();
        let news: Vec<usize> = pairs.iter().filter_map(|p| p.1).collect();
        assert_eq!(
            olds,
            (0..old_len).collect::<Vec<_>>(),
            "{old_len}/{new_len}"
        );
        assert_eq!(
            news,
            (0..new_len).collect::<Vec<_>>(),
            "{old_len}/{new_len}"
        );
        assert!(pairs.iter().all(|p| p.0.is_some() || p.1.is_some()));
    }
}

// Rule: the words that differ are marked on their own side; changed words with only blanks between them are
// one mark; a line that was replaced rather than edited has no marks. 7/10: it is the emphasis in the page.
#[test]
fn only_the_words_that_differ_are_marked() {
    let (old, new) = marks("use the old name here", "use the new name here");
    assert_eq!((old, new), (vec![8..11], vec![8..11]));

    let (old, new) = marks("a b c", "a x y c");
    assert_eq!((old, new), (vec![2..3], vec![2..5]), "x and y are one mark");

    let (old, new) = marks("alpha beta", "gamma delta");
    assert!(
        old.is_empty() && new.is_empty(),
        "a replaced line is tinted whole"
    );
}

// Rule: two lines that read the same but end differently (CRLF against LF, or a missing last newline) say how
// they end; without it the change is red and green with nothing to see. 7/10.
#[test]
fn a_change_that_is_only_the_line_ending_shows_the_ending() {
    let rows = rows_of("same\r\nlast", "same\nlast\n");

    assert_eq!(rows.len(), 2, "{rows:?}");
    let (old, new) = change(&rows[0]);
    let (old, new) = (old.unwrap(), new.unwrap());
    assert_eq!((old.text.as_str(), new.text.as_str()), ("same␍␊", "same␊"));
    assert_eq!(old.marks, vec!["same".len().."same␍␊".len()]);
    let (old, new) = change(&rows[1]);
    assert!(
        old.unwrap().text.starts_with("last∅"),
        "no newline at the end of the old file"
    );
    assert_eq!(new.unwrap().text, "last␊");
}

// Rule: what a file holds cannot reach the terminal as a control: tabs become spaces, an escape sequence
// becomes its picture. 8/10: a skill file is somebody else's text.
#[test]
fn control_characters_never_reach_the_screen() {
    let rows = rows_of("a\tb\n", "a\u{1b}[31mb\u{7}\n");

    let (old, new) = change(&rows[0]);
    let (old, new) = (old.unwrap(), new.unwrap());
    assert_eq!(old.text, "a    b");
    assert_eq!(new.text, "a␛[31mb␇");
    assert!(
        !old.text
            .chars()
            .chain(new.text.chars())
            .any(char::is_control)
    );
}

// Rule: the rows come from the plan the page applies: one group per skill and change, every kind of file
// has its body, failures are not dropped. 7/10: the whole path from a plan to the page.
#[test]
fn a_plan_becomes_groups_with_a_body_for_every_kind_of_file() {
    let (_tree, ws) = world();
    let report = ws.scan(&ignore_events).unwrap();

    let sides = sides_of_plan(&plan_sync(&ws, &report));

    assert_eq!(sides.failed, []);
    assert_eq!(sides.groups.len(), 1, "only coding in p1 differs");
    let group = &sides.groups[0];
    assert_eq!(group.skill, "coding");
    assert!(group.targets[0].project.ends_with("p1"));
    let file = |path: &str| {
        group
            .files
            .iter()
            .find(|f| f.path.to_str() == Some(path))
            .unwrap_or_else(|| panic!("no {path}"))
    };
    let SideBody::Rows(rows) = &file("SKILL.md").body else {
        panic!("SKILL.md has rows");
    };
    let (old, new) = change(&rows[2]);
    assert_eq!(old.unwrap().text, "old line");
    assert_eq!(new.unwrap().text, "new line");
    assert_eq!((file("SKILL.md").added, file("SKILL.md").removed), (1, 1));
    let SideBody::Rows(rows) = &file("ref.md").body else {
        panic!("ref.md has rows");
    };
    assert!(
        matches!(
            &rows[0],
            SideRow::Change {
                old: None,
                new: Some(_)
            }
        ),
        "a new file has nothing on the left"
    );
    assert_eq!(file("run.sh").body, SideBody::ModeOnly);
    assert_eq!(file("img.bin").body, SideBody::Skipped(Skipped::Binary));
    assert_eq!(file("stray/notes.md").kind, DiffKind::Removed);
}

// Rule: a target the plan cannot make is listed with the reason and is not a group: the page must say that a
// skill could not be compared, or "nothing to update" would be a lie. 7/10.
#[test]
fn a_target_that_cannot_be_planned_is_listed_with_its_reason() {
    let (tree, _) = world();
    // A vault skill whose SKILL.md git does not track: the plan refuses it, and p2 has a copy of it.
    tree.write("vault/ghost/SKILL.md", "not committed\n");
    tree.write("projects/p2/.agents/skills/ghost/SKILL.md", "old\n");
    let ws = open(&tree);
    let report = ws.scan(&ignore_events).unwrap();

    let sides = sides_of_plan(&plan_sync(&ws, &report));

    assert_eq!(sides.failed.len(), 1, "{:?}", sides.failed);
    let (target, reason) = &sides.failed[0];
    assert!(target.project.ends_with("p2"));
    assert_eq!(target.skill, "ghost");
    assert!(!reason.is_empty(), "a failure says why");
    assert!(sides.groups.iter().all(|g| g.skill != "ghost"));
    assert!(!sides.is_empty());
}

// Rule: targets of one skill with exactly the same change are one group, the biggest group of a skill first;
// a target that differs is a group of its own. 8/10: twenty projects must not be twenty pages.
#[test]
fn equal_changes_are_one_group_and_the_usual_one_comes_first() {
    let target = |project: &str, skill: &str| Target {
        project: PathBuf::from(project),
        skill: skill.to_string(),
        path: PathBuf::from(project).join(skill),
    };
    let file = |old: &str, new: &str| RawFile {
        path: PathBuf::from("SKILL.md"),
        kind: DiffKind::Modified,
        modes: None,
        skipped: None,
        old: old.to_string(),
        new: new.to_string(),
    };
    let entries = vec![
        (target("/p/one", "a"), vec![file("odd\n", "new\n")]),
        (target("/p/two", "a"), vec![file("old\n", "new\n")]),
        (target("/p/three", "b"), vec![file("old\n", "new\n")]),
        (target("/p/four", "a"), vec![file("old\n", "new\n")]),
    ];

    let groups = group::groups(entries);

    let shape: Vec<(&str, usize)> = groups
        .iter()
        .map(|g| (g.skill.as_str(), g.targets.len()))
        .collect();
    assert_eq!(shape, [("a", 2), ("a", 1), ("b", 1)]);
    assert_eq!(
        groups[0].targets[0].project,
        PathBuf::from("/p/two"),
        "plan order inside a group"
    );
}
