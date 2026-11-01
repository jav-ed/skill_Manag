//! The state of the changes page: what it shows, how, and where the reader is. Drawing lives in `ui`.
//!
//! One page serves the list (`v` on a skill), the question of a sync, a push, an add or an init, and the
//! Agents page. They differ in the report they give it and in where `esc` leads back to, which the screen
//! that owns the page knows.

mod keys;
mod layout;
mod mouse;
mod wrap;

pub(crate) use layout::{CHANGE_MARGIN, Item, Layout, LayoutKey, Mark, Placed};
pub(crate) use wrap::{cut, wrap};

use std::collections::HashSet;

use skillmirror_core::ops::SideReport;

use crate::num::step;
use crate::results::Kind;

/// The buttons of the header, as the numbers of `Target::DiffButton`.
pub(crate) const PREVIOUS: usize = 0;
pub(crate) const NEXT: usize = 1;
pub(crate) const MODE: usize = 2;
pub(crate) const FOLDS: usize = 3;

/// What the page asks for after a key or a click.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiffAction {
    None,
    /// Back to where the page was opened from.
    Leave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ViewMode {
    /// The old text on the left, the new text on the right.
    Split,
    /// One column: the old lines, then the new ones.
    Inline,
}

/// Where a fold is: the group, the file, and the first row it hides. It stays the same while the page is open.
pub(crate) type FoldId = (usize, usize, usize);

/// What the two sides are called above the columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Labels {
    pub(crate) old: &'static str,
    pub(crate) new: &'static str,
}

impl Labels {
    pub(crate) fn of(kind: Kind) -> Self {
        let (old, new) = match kind {
            Kind::Sync => ("in the project", "after the sync"),
            Kind::Push => ("in the project", "after the push"),
            Kind::Agents => ("AGENTS.md now", "AGENTS.md after"),
            Kind::Add | Kind::Init | Kind::Delete => ("now", "after"),
        };
        Self { old, new }
    }
}

/// What the report holds, for the header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Totals {
    pub(crate) projects: usize,
    pub(crate) files: usize,
    pub(crate) added: usize,
    pub(crate) removed: usize,
}

pub(crate) struct DiffView {
    pub(crate) report: SideReport,
    pub(crate) labels: Labels,
    pub(crate) mode: ViewMode,
    /// Unchanged lines far from a change are folded into one line.
    pub(crate) fold: bool,
    /// The folds the reader opened.
    pub(crate) opened: HashSet<FoldId>,
    /// The first visible screen row.
    pub(crate) scroll: usize,
    /// The screen rows of the body; written by the drawing.
    pub(crate) viewport: usize,
    /// Written by the drawing, when the width, the mode or a fold is not the one it was made for.
    pub(crate) layout: Option<Layout>,
}

impl DiffView {
    pub(crate) fn new(report: SideReport, labels: Labels) -> Self {
        Self {
            report,
            labels,
            mode: ViewMode::Split,
            fold: true,
            opened: HashSet::new(),
            scroll: 0,
            viewport: 0,
            layout: None,
        }
    }

    pub(crate) fn totals(&self) -> Totals {
        let files = || self.report.groups.iter().flat_map(|g| g.files.iter());
        let projects: HashSet<&std::path::Path> = self
            .report
            .groups
            .iter()
            .flat_map(|g| &g.targets)
            .map(|target| target.project.as_path())
            .collect();
        Totals {
            projects: projects.len(),
            files: files().count(),
            added: files().map(|f| f.added).sum(),
            removed: files().map(|f| f.removed).sum(),
        }
    }

    /// The most the page scrolls: the last row at the bottom of the body.
    pub(crate) fn max_scroll(&self) -> usize {
        self.layout
            .as_ref()
            .map_or(0, |layout| layout.total.saturating_sub(self.viewport))
    }

    pub(crate) fn scroll_by(&mut self, delta: isize) {
        self.scroll = step(self.scroll, delta, self.max_scroll() + 1);
    }

    pub(crate) fn scroll_to(&mut self, row: usize) {
        self.scroll = row.min(self.max_scroll());
    }

    /// Makes the layout fit a body of `width` columns and `height` rows. The row at the top stays at the top
    /// however the rows below it break again.
    pub(crate) fn fit(&mut self, width: usize, height: usize) {
        self.viewport = height;
        let key = LayoutKey {
            width,
            mode: self.mode,
            fold: self.fold,
            opened: self.opened.len(),
        };
        if self.layout.as_ref().is_none_or(|layout| layout.key != key) {
            let anchor = self
                .layout
                .as_ref()
                .and_then(|layout| layout.position_at(self.scroll));
            let layout = Layout::build(&self.report, key, &self.opened);
            self.scroll = anchor.map_or(0, |position| layout.top_of(position));
            self.layout = Some(layout);
        }
        self.scroll = self.scroll.min(self.max_scroll());
    }
}
