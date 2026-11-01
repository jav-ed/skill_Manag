//! A row in one column: the old line (red), then the new line (green), with both numbers in the gutter.

use ratatui::text::{Line, Span};
use skillmirror_core::ops::SideRow;
use unicode_width::UnicodeWidthStr;

use super::{Paint, marked};
use crate::screens::diff::{Layout, cut, wrap};
use crate::theme;

/// Which numbers the gutter of a line shows.
#[derive(Clone, Copy)]
enum Numbers {
    Both(usize, usize),
    Old(usize),
    New(usize),
}

pub(super) fn lines(row: &SideRow, layout: &Layout, height: usize) -> Vec<Line<'static>> {
    let mut out = Vec::with_capacity(height);
    match row {
        SideRow::Same {
            old_no,
            new_no,
            text,
        } => out.extend(block(
            text,
            &[],
            Numbers::Both(*old_no, *new_no),
            &Paint::same(),
            layout,
        )),
        SideRow::Change { old, new } => {
            if let Some(line) = old {
                let numbers = Numbers::Old(line.no);
                out.extend(block(
                    &line.text,
                    &line.marks,
                    numbers,
                    &Paint::old(),
                    layout,
                ));
            }
            if let Some(line) = new {
                let numbers = Numbers::New(line.no);
                out.extend(block(
                    &line.text,
                    &line.marks,
                    numbers,
                    &Paint::new(),
                    layout,
                ));
            }
        }
    }
    out
}

/// The screen rows of one line of text.
fn block(
    text: &str,
    marks: &[std::ops::Range<usize>],
    numbers: Numbers,
    paint: &Paint,
    layout: &Layout,
) -> Vec<Line<'static>> {
    let w = layout.num_w;
    wrap(text, layout.text_w)
        .into_iter()
        .enumerate()
        .map(|(k, range)| {
            let (old, new) = match (k, numbers) {
                (0, Numbers::Both(old, new)) => (format!("{old:>w$}"), format!("{new:>w$}")),
                (0, Numbers::Old(old)) => (format!("{old:>w$}"), " ".repeat(w)),
                (0, Numbers::New(new)) => (" ".repeat(w), format!("{new:>w$}")),
                _ => (" ".repeat(w), " ".repeat(w)),
            };
            let mut spans = vec![
                Span::styled(format!("{old} {new} "), paint.line.fg(theme::MUTED)),
                Span::styled(
                    if k == 0 { paint.sign } else { " " },
                    paint.line.fg(paint.sign_color),
                ),
                Span::styled(" ", paint.line),
            ];
            spans.extend(marked(text, range.clone(), marks, paint));
            let used = cut(text, range).width();
            spans.push(Span::styled(
                " ".repeat(layout.text_w.saturating_sub(used)),
                paint.line,
            ));
            Line::from(spans)
        })
        .collect()
}
