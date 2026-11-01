//! Where everything of the page goes: every group, file and row of the report counted in screen rows for the
//! width the page has. It is made when the page is first drawn, and again when the width, the mode or a fold
//! changes. Drawing then only has to cut out the rows in view.

mod build;
mod measure;

use std::collections::HashSet;

use skillmirror_core::ops::SideReport;

use super::{FoldId, ViewMode};
use measure::{digits, last_number};

/// Rows kept above a change the page jumps to, so that the lines before it are in view.
pub(crate) const CHANGE_MARGIN: usize = 2;
/// Narrower than this the page is one column: two columns of text would be too narrow to read.
pub(crate) const MIN_SPLIT: usize = 70;

/// What the page was laid out for. A layout is made again when any of it is no longer true.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LayoutKey {
    pub(crate) width: usize,
    pub(crate) mode: ViewMode,
    pub(crate) fold: bool,
    pub(crate) opened: usize,
}

/// What a row changes, for the ruler and for the jumps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mark {
    None,
    Added,
    Removed,
    Both,
}

impl Mark {
    pub(crate) fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::None, mark) | (mark, Self::None) => mark,
            (a, b) if a == b => a,
            _ => Self::Both,
        }
    }
}

/// One thing the page shows, in the order of the page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Item {
    /// The head of a group: the skill and its projects, one line of names each.
    Group {
        group: usize,
        names: Vec<String>,
    },
    /// Empty line before the group.
    Blank(usize),
    File {
        group: usize,
        file: usize,
    },
    /// Why a file has no lines: binary, too large, unreadable, only its permissions.
    Note {
        group: usize,
        file: usize,
        lines: Vec<String>,
    },
    /// Unchanged lines left out; a click opens them.
    Fold {
        id: FoldId,
        hidden: usize,
    },
    Row {
        group: usize,
        file: usize,
        row: usize,
    },
    FailedHead,
    /// A target that cannot be applied, with the reason.
    Failed {
        lines: Vec<String>,
        index: usize,
    },
    /// The page ends here, with the reason.
    Cut(Vec<String>),
}

/// Where an item is in the report. Items come in this order, so a position survives a new layout.
pub(crate) type Position = (usize, usize, usize);

impl Item {
    pub(crate) fn position(&self) -> Position {
        match self {
            Self::Group { group, .. } | Self::Blank(group) => (*group, 0, 0),
            Self::File { group, file } => (*group, file + 1, 0),
            Self::Note { group, file, .. } => (*group, file + 1, 1),
            Self::Fold {
                id: (g, f, first), ..
            } => (*g, f + 1, first + 1),
            Self::Row { group, file, row } => (*group, file + 1, row + 1),
            Self::FailedHead => (usize::MAX, 0, 0),
            Self::Failed { index, .. } => (usize::MAX, 0, index + 1),
            Self::Cut(_) => (usize::MAX, usize::MAX, 0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Placed {
    pub(crate) item: Item,
    /// The screen row the item starts at.
    pub(crate) top: usize,
    pub(crate) height: usize,
    pub(crate) mark: Mark,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Layout {
    pub(crate) key: LayoutKey,
    /// Whether the columns are side by side: the mode asks for it and the width allows it.
    pub(crate) split: bool,
    /// The width of a line number.
    pub(crate) num_w: usize,
    /// The width of one column of text.
    pub(crate) text_w: usize,
    pub(crate) items: Vec<Placed>,
    /// Screen rows in all.
    pub(crate) total: usize,
    /// The first row of each run of changed lines, of each file and of each group.
    pub(crate) changes: Vec<usize>,
    pub(crate) files: Vec<usize>,
    pub(crate) groups: Vec<usize>,
}

impl Layout {
    pub(crate) fn build(report: &SideReport, key: LayoutKey, opened: &HashSet<FoldId>) -> Self {
        let split = key.mode == ViewMode::Split && key.width >= MIN_SPLIT;
        let num_w = digits(last_number(report)).max(3);
        // Split: two gutters (number, marker, blanks) and the line between the columns. Inline: one gutter
        // with both numbers.
        let text_w = if split {
            key.width.saturating_sub(2 * (num_w + 3) + 1) / 2
        } else {
            key.width.saturating_sub(2 * num_w + 4)
        }
        .max(1);
        let built = build::build(report, opened, key, split, text_w);
        Self {
            key,
            split,
            num_w,
            text_w,
            total: built.total,
            items: built.items,
            changes: built.changes,
            files: built.files,
            groups: built.groups,
        }
    }

    /// The item that covers screen row `row`.
    pub(crate) fn index_at(&self, row: usize) -> usize {
        self.items.partition_point(|p| p.top + p.height <= row)
    }

    pub(crate) fn position_at(&self, row: usize) -> Option<Position> {
        self.items
            .get(self.index_at(row))
            .map(|placed| placed.item.position())
    }

    /// The screen row of the first item at or after `position`.
    pub(crate) fn top_of(&self, position: Position) -> usize {
        let index = self
            .items
            .partition_point(|placed| placed.item.position() < position);
        self.items
            .get(index)
            .map_or(self.total, |placed| placed.top)
    }
}
