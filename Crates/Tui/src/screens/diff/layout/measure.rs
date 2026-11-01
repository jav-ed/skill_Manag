//! The sums a layout is made of: which rows are near a change, how many screen rows a line takes, how the
//! names of the projects break into lines. Pure functions of the report and the width.

use std::ops::Range;

use skillmirror_core::ops::{ChangeGroup, SideBody, SideReport, SideRow, Skipped};

use super::Mark;
use crate::results::short_path;
use crate::screens::diff::wrap::wrap;

/// Unchanged lines kept around each change when the others are folded.
const CONTEXT: usize = 3;
/// Lines of project names under the head of a group; the rest is counted.
const NAME_LINES: usize = 3;

pub(super) fn mark_of(row: &SideRow) -> Mark {
    match row {
        SideRow::Same { .. } => Mark::None,
        SideRow::Change {
            old: Some(_),
            new: Some(_),
        } => Mark::Both,
        SideRow::Change {
            old: Some(_),
            new: None,
        } => Mark::Removed,
        SideRow::Change { old: None, new: _ } => Mark::Added,
    }
}

/// The screen rows of a row: the taller side when the sides are beside each other, both when one is under
/// the other.
pub(super) fn row_height(row: &SideRow, text_w: usize, split: bool) -> usize {
    let rows = |text: &str| wrap(text, text_w).len();
    match row {
        SideRow::Same { text, .. } => rows(text),
        SideRow::Change { old, new } => {
            let old = old.as_ref().map_or(0, |line| rows(&line.text));
            let new = new.as_ref().map_or(0, |line| rows(&line.text));
            if split { old.max(new) } else { old + new }
        }
    }
}

/// For every row, whether it is within `CONTEXT` rows of a change.
pub(super) fn near_changes(rows: &[SideRow]) -> Vec<bool> {
    let changed = |row: &SideRow| matches!(row, SideRow::Change { .. });
    let mut since = usize::MAX;
    let before: Vec<usize> = rows
        .iter()
        .map(|row| {
            since = if changed(row) {
                0
            } else {
                since.saturating_add(1)
            };
            since
        })
        .collect();
    let mut until = usize::MAX;
    let mut after: Vec<usize> = rows
        .iter()
        .rev()
        .map(|row| {
            until = if changed(row) {
                0
            } else {
                until.saturating_add(1)
            };
            until
        })
        .collect();
    after.reverse();
    before
        .into_iter()
        .zip(after)
        .map(|(before, after)| before.min(after) <= CONTEXT)
        .collect()
}

/// Stretches of equal flags, as `(flag, range)`.
pub(super) fn runs(flags: &[bool]) -> Vec<(bool, Range<usize>)> {
    let mut out: Vec<(bool, Range<usize>)> = Vec::new();
    for (i, &flag) in flags.iter().enumerate() {
        match out.last_mut() {
            Some((last, range)) if *last == flag => range.end = i + 1,
            _ => out.push((flag, i..i + 1)),
        }
    }
    out
}

/// The names of the projects of a group, as lines no wider than `width`; names beyond the lines are counted.
pub(super) fn name_lines(group: &ChangeGroup, width: usize) -> Vec<String> {
    let names: Vec<String> = group
        .targets
        .iter()
        .map(|target| short_path(&target.project.to_string_lossy()))
        .collect();
    let mut lines = vec![String::new()];
    for (i, name) in names.iter().enumerate() {
        let piece = if i + 1 < names.len() {
            format!("{name}, ")
        } else {
            name.clone()
        };
        let line_width = lines.last().map_or(0, |line| line.chars().count());
        if line_width > 0 && line_width + piece.chars().count() > width {
            if lines.len() == NAME_LINES {
                lines.push(format!("and {} more", names.len() - i));
                return lines;
            }
            lines.push(String::new());
        }
        if let Some(line) = lines.last_mut() {
            line.push_str(&piece);
        }
    }
    lines
}

pub(super) fn skipped_text(why: &Skipped) -> String {
    match why {
        Skipped::Binary => "binary file, lines not shown".to_string(),
        Skipped::TooLarge => "too large, lines not shown".to_string(),
        Skipped::Unreadable(reason) => format!("cannot be read: {reason}"),
    }
}

/// The highest line number of the report, to know how wide the numbers are.
pub(super) fn last_number(report: &SideReport) -> usize {
    report
        .groups
        .iter()
        .flat_map(|group| &group.files)
        .filter_map(|file| match &file.body {
            SideBody::Rows(rows) => Some(rows),
            _ => None,
        })
        .flatten()
        .map(|row| match row {
            SideRow::Same { old_no, new_no, .. } => *old_no.max(new_no),
            SideRow::Change { old, new } => {
                old.iter().chain(new).map(|line| line.no).max().unwrap_or(0)
            }
        })
        .max()
        .unwrap_or(0)
}

pub(super) fn digits(n: usize) -> usize {
    n.max(1).to_string().len()
}
