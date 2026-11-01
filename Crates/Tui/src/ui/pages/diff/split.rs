//! A row with the old text on the left and the new text on the right. A side with no line, or with fewer
//! screen rows than the other side, is hatched.

use std::ops::Range;

use ratatui::style::Style;
use ratatui::text::{Line, Span};
use skillmirror_core::ops::{SideLine, SideRow};
use unicode_width::UnicodeWidthStr;

use super::{Paint, marked};
use crate::screens::diff::{Layout, cut, wrap};
use crate::theme;

/// One side of a row.
struct Piece<'a> {
    no: usize,
    text: &'a str,
    marks: &'a [Range<usize>],
    /// The screen rows its text breaks into.
    rows: Vec<Range<usize>>,
}

impl<'a> Piece<'a> {
    fn new(no: usize, text: &'a str, marks: &'a [Range<usize>], width: usize) -> Self {
        Self {
            no,
            text,
            marks,
            rows: wrap(text, width),
        }
    }

    fn of(line: &'a SideLine, width: usize) -> Self {
        Self::new(line.no, &line.text, &line.marks, width)
    }
}

pub(super) fn lines(row: &SideRow, layout: &Layout, height: usize) -> Vec<Line<'static>> {
    let width = layout.text_w;
    let (old, new, paints) = match row {
        SideRow::Same {
            old_no,
            new_no,
            text,
        } => (
            Some(Piece::new(*old_no, text, &[], width)),
            Some(Piece::new(*new_no, text, &[], width)),
            (Paint::same(), Paint::same()),
        ),
        SideRow::Change { old, new } => (
            old.as_ref().map(|line| Piece::of(line, width)),
            new.as_ref().map(|line| Piece::of(line, width)),
            (Paint::old(), Paint::new()),
        ),
    };
    // What the two halves and the line between them leave of the width: the spare cell of an odd width.
    let spare = layout
        .key
        .width
        .saturating_sub(2 * (layout.num_w + 3 + width) + 1);
    (0..height)
        .map(|k| {
            let mut spans = half(old.as_ref(), k, &paints.0, layout);
            spans.push(Span::styled("│", theme::muted()));
            spans.extend(half(new.as_ref(), k, &paints.1, layout));
            spans.push(Span::raw(" ".repeat(spare)));
            Line::from(spans)
        })
        .collect()
}

/// The gutter and the text of screen row `k` of one side.
fn half(piece: Option<&Piece>, k: usize, paint: &Paint, layout: &Layout) -> Vec<Span<'static>> {
    let gutter = layout.num_w + 3;
    let hatching = || {
        vec![
            Span::raw(" ".repeat(gutter)),
            Span::styled("╱".repeat(layout.text_w), Style::new().fg(theme::HATCH)),
        ]
    };
    let Some((piece, range)) = piece.and_then(|piece| Some((piece, piece.rows.get(k)?))) else {
        return hatching();
    };
    let first = k == 0;
    let number = if first {
        format!("{:>w$} ", piece.no, w = layout.num_w)
    } else {
        " ".repeat(layout.num_w + 1)
    };
    let mut spans = vec![
        Span::styled(number, paint.line.fg(theme::MUTED)),
        Span::styled(
            if first { paint.sign } else { " " },
            paint.line.fg(paint.sign_color),
        ),
        Span::styled(" ", paint.line),
    ];
    spans.extend(marked(piece.text, range.clone(), piece.marks, paint));
    let used = cut(piece.text, range.clone()).width();
    spans.push(Span::styled(
        " ".repeat(layout.text_w.saturating_sub(used)),
        paint.line,
    ));
    spans
}
