//! The rows of one file: the lines that stayed, and the lines that changed with their partners.

use std::ops::Range;

use similar::{DiffOp, TextDiff};

use super::pairs::pair;
use super::words::marks;
use super::{DiffKind, RawFile, SideBody, SideFile, SideLine, SideRow};

/// A tab is shown as this many spaces: a terminal would stop at its own tab stops, and the two sides must
/// have the same width for the same text.
const TAB: &str = "    ";

pub(super) fn side_file(raw: RawFile) -> SideFile {
    let RawFile {
        path,
        kind,
        modes,
        skipped,
        old,
        new,
    } = raw;
    let (body, added, removed) = match (skipped, kind) {
        (Some(why), _) => (SideBody::Skipped(why), 0, 0),
        (None, DiffKind::ModeChanged) => (SideBody::ModeOnly, 0, 0),
        (None, _) => {
            let rows = rows_of(&old, &new);
            let count = |side: fn(&SideRow) -> bool| rows.iter().filter(|row| side(row)).count();
            let added = count(|row| matches!(row, SideRow::Change { new: Some(_), .. }));
            let removed = count(|row| matches!(row, SideRow::Change { old: Some(_), .. }));
            (SideBody::Rows(rows), added, removed)
        }
    };
    SideFile {
        path,
        kind,
        modes,
        added,
        removed,
        body,
    }
}

/// Every line of both texts as rows, in file order.
pub(super) fn rows_of(old: &str, new: &str) -> Vec<SideRow> {
    let diff = TextDiff::from_lines(old, new);
    // The ops index into the lines, so they are collected once: `similar` 3 has no slice of them.
    let old_lines: Vec<&str> = diff.iter_old_slices().collect();
    let new_lines: Vec<&str> = diff.iter_new_slices().collect();
    let mut rows = Vec::new();
    for op in diff.ops() {
        match *op {
            DiffOp::Equal {
                old_index,
                new_index,
                len,
            } => rows.extend(old_lines.iter().skip(old_index).take(len).enumerate().map(
                |(k, line)| SideRow::Same {
                    old_no: old_index + k + 1,
                    new_no: new_index + k + 1,
                    text: display(body(line).0),
                },
            )),
            DiffOp::Delete {
                old_index, old_len, ..
            } => rows.extend(lines(&old_lines, old_index, old_len).map(|(no, line)| {
                SideRow::Change {
                    old: Some(alone(no, line)),
                    new: None,
                }
            })),
            DiffOp::Insert {
                new_index, new_len, ..
            } => rows.extend(lines(&new_lines, new_index, new_len).map(|(no, line)| {
                SideRow::Change {
                    old: None,
                    new: Some(alone(no, line)),
                }
            })),
            DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                let old_run: Vec<&str> = lines(&old_lines, old_index, old_len)
                    .map(|(_, line)| line)
                    .collect();
                let new_run: Vec<&str> = lines(&new_lines, new_index, new_len)
                    .map(|(_, line)| line)
                    .collect();
                for (a, b) in pair(&old_run, &new_run) {
                    let a = a.and_then(|k| old_run.get(k).map(|line| (old_index + k + 1, *line)));
                    let b = b.and_then(|k| new_run.get(k).map(|line| (new_index + k + 1, *line)));
                    rows.push(changed(a, b));
                }
            }
        }
    }
    rows
}

/// The `len` lines from `first`, with their numbers counted from 1.
fn lines<'a>(
    all: &'a [&'a str],
    first: usize,
    len: usize,
) -> impl Iterator<Item = (usize, &'a str)> {
    all.iter()
        .skip(first)
        .take(len)
        .enumerate()
        .map(move |(k, line)| (first + k + 1, *line))
}

/// A line that has no partner: shown whole, nothing marked.
fn alone(no: usize, line: &str) -> SideLine {
    SideLine {
        no,
        text: display(body(line).0),
        marks: Vec::new(),
    }
}

/// Two lines that stand side by side: the words they do not share are marked.
fn changed(old: Option<(usize, &str)>, new: Option<(usize, &str)>) -> SideRow {
    let (old, new) = match (old, new) {
        (Some(old), Some(new)) => {
            let (old, new) = paired(old, new);
            (Some(old), Some(new))
        }
        (old, new) => (
            old.map(|(no, line)| alone(no, line)),
            new.map(|(no, line)| alone(no, line)),
        ),
    };
    SideRow::Change { old, new }
}

fn paired(
    (old_no, old_line): (usize, &str),
    (new_no, new_line): (usize, &str),
) -> (SideLine, SideLine) {
    let ((old_body, old_end), (new_body, new_end)) = (body(old_line), body(new_line));
    let (mut old_text, mut new_text) = (display(old_body), display(new_body));
    if old_text == new_text {
        // The lines read the same, so they differ in how they end. Say how, or the change cannot be seen.
        let (old_mark, new_mark) = (
            end_mark(&mut old_text, old_end),
            end_mark(&mut new_text, new_end),
        );
        return (
            line_of(old_no, old_text, vec![old_mark]),
            line_of(new_no, new_text, vec![new_mark]),
        );
    }
    let (old_marks, new_marks) = marks(&old_text, &new_text);
    (
        line_of(old_no, old_text, old_marks),
        line_of(new_no, new_text, new_marks),
    )
}

fn line_of(no: usize, text: String, marks: Vec<Range<usize>>) -> SideLine {
    SideLine { no, text, marks }
}

/// How a line ends.
#[derive(Clone, Copy)]
enum End {
    /// `\n`.
    Lf,
    /// `\r\n`.
    CrLf,
    /// The last line of a file that has no newline.
    Open,
}

/// The line without its ending, and the ending.
fn body(line: &str) -> (&str, End) {
    match line.strip_suffix('\n') {
        Some(rest) => match rest.strip_suffix('\r') {
            Some(rest) => (rest, End::CrLf),
            None => (rest, End::Lf),
        },
        None => (line, End::Open),
    }
}

/// Writes the ending after the text, so that it can be seen, and returns where it is.
fn end_mark(text: &mut String, end: End) -> Range<usize> {
    let start = text.len();
    text.push_str(match end {
        End::Lf => "␊",
        End::CrLf => "␍␊",
        End::Open => "∅ no newline at the end",
    });
    start..text.len()
}

/// The line as it is shown: tabs as spaces and control characters as their pictures, so that what the file
/// holds cannot move the cursor or draw on the screen, and the width of the text is what is counted.
fn display(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\t' => out.push_str(TAB),
            '\u{7f}' => out.push('␡'),
            // The pictures of the controls 0 to 31 are the characters from U+2400 on.
            c if c.is_ascii_control() => out
                .push(char::from_u32(0x2400 + u32::from(c)).unwrap_or(char::REPLACEMENT_CHARACTER)),
            c if c.is_control() => out.push(char::REPLACEMENT_CHARACTER),
            c => out.push(c),
        }
    }
    out
}
