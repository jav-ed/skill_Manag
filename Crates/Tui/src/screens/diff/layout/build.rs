//! Walking the report once and placing every item: the heads, the rows near a change, a fold for each long
//! stretch of the others, and the stops the jumps go between.

use std::collections::HashSet;

use skillmirror_core::ops::{ChangeGroup, SideBody, SideReport, SideRow};

use super::measure::{mark_of, name_lines, near_changes, row_height, runs, skipped_text};
use super::{Item, LayoutKey, Mark, Placed};
use crate::results::short_path;
use crate::screens::diff::FoldId;
use crate::screens::diff::wrap::rows as text_rows;

/// Fewer unchanged lines than this are shown, not folded: the line that stands for them would be as long.
const FOLD_MIN: usize = 4;
/// More items than this make the page slow; `skillmirror diff` shows every change.
const MAX_ITEMS: usize = 20_000;

/// What the walk found; the layout adds the widths it was made for.
pub(super) struct Built {
    pub(super) items: Vec<Placed>,
    /// Screen rows in all.
    pub(super) total: usize,
    pub(super) changes: Vec<usize>,
    pub(super) files: Vec<usize>,
    pub(super) groups: Vec<usize>,
}

pub(super) fn build(
    report: &SideReport,
    opened: &HashSet<FoldId>,
    key: LayoutKey,
    split: bool,
    text_w: usize,
) -> Built {
    let mut walk = Builder {
        report,
        opened,
        key,
        split,
        text_w,
        items: Vec::new(),
        top: 0,
        changes: Vec::new(),
        files: Vec::new(),
        groups: Vec::new(),
        after_change: false,
    };
    walk.all();
    Built {
        items: walk.items,
        total: walk.top,
        changes: walk.changes,
        files: walk.files,
        groups: walk.groups,
    }
}

struct Builder<'a> {
    report: &'a SideReport,
    opened: &'a HashSet<FoldId>,
    key: LayoutKey,
    split: bool,
    text_w: usize,
    items: Vec<Placed>,
    top: usize,
    changes: Vec<usize>,
    files: Vec<usize>,
    groups: Vec<usize>,
    /// The item before was a changed row: the next one continues its run.
    after_change: bool,
}

impl Builder<'_> {
    fn push(&mut self, item: Item, height: usize, mark: Mark) {
        self.after_change = matches!(item, Item::Row { .. }) && mark != Mark::None;
        self.items.push(Placed {
            item,
            top: self.top,
            height,
            mark,
        });
        self.top += height;
    }

    fn full(&self) -> bool {
        self.items.len() >= MAX_ITEMS
    }

    fn all(&mut self) {
        for (g, group) in self.report.groups.iter().enumerate() {
            if self.full() {
                self.cut();
                return;
            }
            self.group(g, group);
        }
        if !self.report.failed.is_empty() {
            self.failed();
        }
    }

    fn cut(&mut self) {
        let text =
            "The page stops here: it would be too long. `skillmirror diff` shows every change.";
        let lines = text_rows(text, self.key.width.saturating_sub(2));
        let height = lines.len();
        self.push(Item::Cut(lines), height, Mark::None);
    }

    fn group(&mut self, g: usize, group: &ChangeGroup) {
        if g > 0 {
            self.push(Item::Blank(g), 1, Mark::None);
        }
        // One project is named in the head itself; the names of several go under it.
        let names = if group.targets.len() == 1 {
            Vec::new()
        } else {
            name_lines(group, self.key.width.saturating_sub(2))
        };
        let height = 1 + names.len();
        self.groups.push(self.top);
        self.push(Item::Group { group: g, names }, height, Mark::None);
        for (f, file) in group.files.iter().enumerate() {
            if self.full() {
                self.cut();
                return;
            }
            self.files.push(self.top);
            self.push(Item::File { group: g, file: f }, 1, Mark::None);
            match &file.body {
                SideBody::Rows(rows) => self.rows(g, f, rows),
                SideBody::Skipped(why) => self.note(g, f, &skipped_text(why)),
                SideBody::ModeOnly => {
                    let text = file.modes.map_or_else(
                        || "permissions changed".to_string(),
                        |(old, new)| format!("permissions {old:o} → {new:o}"),
                    );
                    self.note(g, f, &text);
                }
            }
        }
    }

    fn note(&mut self, group: usize, file: usize, text: &str) {
        let lines = text_rows(text, self.key.width.saturating_sub(4));
        let height = lines.len().max(1);
        self.push(Item::Note { group, file, lines }, height, Mark::None);
    }

    fn failed(&mut self) {
        self.push(Item::Blank(usize::MAX), 1, Mark::None);
        self.push(Item::FailedHead, 1, Mark::None);
        for (index, (target, reason)) in self.report.failed.iter().enumerate() {
            let project = short_path(&target.project.to_string_lossy());
            let text = format!("{} in {project}: {reason}", target.skill);
            let lines = text_rows(&text, self.key.width.saturating_sub(4));
            let height = lines.len().max(1);
            self.push(Item::Failed { lines, index }, height, Mark::None);
        }
    }

    /// The rows of one file: the ones near a change, and a fold for each long stretch of the others.
    fn rows(&mut self, g: usize, f: usize, rows: &[SideRow]) {
        let near = if self.key.fold {
            near_changes(rows)
        } else {
            vec![true; rows.len()]
        };
        for (shown, range) in runs(&near) {
            let id = (g, f, range.start);
            if !shown && range.len() >= FOLD_MIN && !self.opened.contains(&id) {
                self.push(
                    Item::Fold {
                        id,
                        hidden: range.len(),
                    },
                    1,
                    Mark::None,
                );
                continue;
            }
            for (row, data) in rows.iter().enumerate().skip(range.start).take(range.len()) {
                if self.full() {
                    self.cut();
                    return;
                }
                let mark = mark_of(data);
                if mark != Mark::None && !self.after_change {
                    self.changes.push(self.top);
                }
                let height = row_height(data, self.text_w, self.split);
                self.push(
                    Item::Row {
                        group: g,
                        file: f,
                        row,
                    },
                    height,
                    mark,
                );
            }
        }
    }
}
