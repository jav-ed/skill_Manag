//! The page that shows what a plan would change: the old text on the left and the new text on the right, as
//! in the diff editor of an editor. Only the rows in view are turned into lines; the layout (in `screens`)
//! has counted them all.

mod head;
mod inline;
mod ruler;
mod split;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use std::ops::Range;

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use skillmirror_core::ops::{SideReport, SideRow};

use crate::hit::{HitMap, Target};
use crate::screens::DiffPage;
use crate::screens::diff::{DiffView, Item, Layout as Placing, Placed, cut};
use crate::theme;

/// Narrower than this nothing sensible fits.
const MIN_WIDTH: u16 = 30;

pub(in crate::ui) fn draw(
    frame: &mut Frame,
    hits: &mut HitMap,
    hover: Option<Target>,
    page: &mut DiffPage,
    area: Rect,
) {
    let view = &mut page.view;
    let [title, place, labels, body] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(area);
    head::title(frame, hits, hover, view, title);
    if area.width < MIN_WIDTH {
        message(frame, "The terminal is too narrow for the changes.", body);
        return;
    }
    if view.report.is_empty() {
        message(frame, "Nothing would change.", body);
        return;
    }
    let [text, bar] = Layout::horizontal([Constraint::Fill(1), Constraint::Length(1)]).areas(body);
    view.fit(usize::from(text.width), usize::from(text.height));
    head::place(frame, view, place);
    head::labels(frame, view, labels);
    let Some(layout) = view.layout.as_ref() else {
        return;
    };
    let height = usize::from(text.height);
    let mut lines: Vec<Line<'static>> = Vec::with_capacity(height);
    let first = layout.index_at(view.scroll);
    for (index, placed) in layout.items.iter().enumerate().skip(first) {
        if lines.len() >= height {
            break;
        }
        if matches!(placed.item, Item::Fold { .. }) {
            let y = text.y + crate::num::to_u16(lines.len());
            hits.push(Rect::new(text.x, y, text.width, 1), Target::DiffFold(index));
        }
        let skip = view.scroll.saturating_sub(placed.top);
        let room = height - lines.len();
        lines.extend(
            item_lines(view, layout, placed)
                .into_iter()
                .skip(skip)
                .take(room),
        );
    }
    frame.render_widget(Paragraph::new(lines), text);
    ruler::draw(frame, hits, view, bar);
}

fn message(frame: &mut Frame, text: &str, area: Rect) {
    frame.render_widget(
        Paragraph::new(Span::styled(format!(" {text}"), theme::muted())),
        area,
    );
}

/// The screen rows of one item: as many lines as the layout reserved for it.
fn item_lines(view: &DiffView, layout: &Placing, placed: &Placed) -> Vec<Line<'static>> {
    match &placed.item {
        Item::Group { group, names } => head::group(view, *group, names),
        Item::Blank(_) => vec![Line::raw("")],
        Item::File { group, file } => head::file(view, *group, *file),
        Item::Note { lines, .. } | Item::Cut(lines) => head::note(lines),
        Item::Fold { hidden, .. } => head::fold(*hidden, layout),
        Item::FailedHead => head::failed_head(),
        Item::Failed { lines, .. } => head::failed(lines),
        Item::Row { group, file, row } => match row_of(&view.report, *group, *file, *row) {
            Some(data) if layout.split => split::lines(data, layout, placed.height),
            Some(data) => inline::lines(data, layout, placed.height),
            None => vec![Line::from(Span::styled(
                " internal error: the row is not in the report",
                theme::error(),
            ))],
        },
    }
}

fn row_of(report: &SideReport, group: usize, file: usize, row: usize) -> Option<&SideRow> {
    match &report.groups.get(group)?.files.get(file)?.body {
        skillmirror_core::ops::SideBody::Rows(rows) => rows.get(row),
        _ => None,
    }
}

/// How one kind of line is painted: the line, the words marked in it, the sign before it.
pub(super) struct Paint {
    pub(super) line: Style,
    pub(super) word: Style,
    pub(super) sign: &'static str,
    pub(super) sign_color: Color,
}

impl Paint {
    pub(super) fn same() -> Self {
        Self {
            line: Style::new(),
            word: Style::new(),
            sign: " ",
            sign_color: theme::MUTED,
        }
    }

    pub(super) fn old() -> Self {
        Self {
            line: Style::new().bg(theme::REMOVED_LINE),
            word: Style::new().bg(theme::REMOVED_WORD),
            sign: "-",
            sign_color: theme::ERROR,
        }
    }

    pub(super) fn new() -> Self {
        Self {
            line: Style::new().bg(theme::ADDED_LINE),
            word: Style::new().bg(theme::ADDED_WORD),
            sign: "+",
            sign_color: theme::SUCCESS,
        }
    }
}

/// The part `range` of `text` as spans: the byte ranges `marks` in the style of a mark, the rest in the
/// style of the line.
pub(super) fn marked(
    text: &str,
    range: Range<usize>,
    marks: &[Range<usize>],
    paint: &Paint,
) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut at = range.start;
    for mark in marks
        .iter()
        .filter(|mark| mark.end > range.start && mark.start < range.end)
    {
        let (from, to) = (mark.start.max(range.start), mark.end.min(range.end));
        if from > at {
            spans.push(Span::styled(cut(text, at..from).to_string(), paint.line));
        }
        spans.push(Span::styled(cut(text, from..to).to_string(), paint.word));
        at = to;
    }
    if at < range.end {
        spans.push(Span::styled(
            cut(text, at..range.end).to_string(),
            paint.line,
        ));
    }
    spans
}
