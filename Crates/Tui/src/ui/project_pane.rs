//! The pane of the sync, push and delete pages: the projects of the skill under the cursor, with the
//! views, the path style and the filter in a header that stays put while the projects scroll.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthStr;

use super::pane_text::{Row, fitted, wrap};
use super::skill_card::{Place, pane_block};
use crate::hit::{HitMap, Target};
use crate::items::{Note, Tone};
use crate::items_projects::{Kind, PaneView, PathStyle, ProjectPane, ProjectRow};
use crate::num::to_u16;
use crate::screens::Work;
use crate::theme;

/// The views in the order of their counts and of `Target::PaneTab`.
const TABS: [(PaneView, &str); 3] = [
    (PaneView::Affected, "Affected"),
    (PaneView::Others, "Others"),
    (PaneView::All, "All"),
];

/// Text in a header row and the rectangle it takes, so that what is drawn is what can be clicked.
struct Spot {
    x: u16,
    width: u16,
}

pub(super) fn draw(
    frame: &mut Frame,
    hits: &mut HitMap,
    work: &mut Work,
    area: Rect,
    place: Place,
) {
    let block = pane_block(place);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    hits.push(area, Target::Pane);
    work.follow_pane();
    let Some(pane) = work
        .view
        .current()
        .and_then(|index| work.items.get(index))
        .and_then(|item| item.pane.as_deref())
    else {
        return;
    };

    let filter = work.pane.filter.value();
    let (view, style) = (work.pane.view, work.pane.paths);
    let filtering = work.pane.filtering;
    let visible = pane.visible(view, filter);

    // The header: name, what the run does, the views, the options, and the filter when there is one.
    let mut head = vec![
        Line::from(Span::styled(pane.skill.clone(), theme::bold())),
        Line::from(Span::styled(pane.summary.clone(), theme::muted())),
    ];
    if pane.views {
        let counts = pane.counts(filter);
        let (line, spots) = tabs(view, counts);
        push_hits(hits, &spots, inner, head.len(), Target::PaneTab);
        head.push(line);
    }
    let (line, toggle, hint) = options(style, filter.is_empty() && !filtering);
    push_hits(hits, &[toggle], inner, head.len(), |_| Target::PanePaths);
    if let Some(hint) = hint {
        push_hits(hits, &[hint], inner, head.len(), |_| Target::PaneFilter);
    }
    head.push(line);
    let filter_row = (filtering || !filter.is_empty()).then_some(head.len());
    if filter_row.is_some() {
        push_hits(
            hits,
            &[Spot {
                x: 0,
                width: inner.width,
            }],
            inner,
            head.len(),
            |_| Target::PaneFilter,
        );
        head.push(filter_line(filter, filtering));
    }

    let [head_area, body_area] =
        Layout::vertical([Constraint::Length(to_u16(head.len())), Constraint::Fill(1)])
            .areas(inner);
    frame.render_widget(Paragraph::new(head), head_area);
    if let (Some(row), true) = (filter_row, filtering) {
        let cursor = to_u16(work.pane.filter.visual_cursor()).min(inner.width.saturating_sub(9));
        frame.set_cursor_position((inner.x + 8 + cursor, inner.y + to_u16(row)));
    }

    let notes = body(pane, &visible, view, style, !filter.is_empty());
    let rows: Vec<Row> = notes
        .iter()
        .flat_map(|note| wrap(note, usize::from(inner.width)))
        .collect();
    let (lines, scroll) = fitted(&rows, usize::from(body_area.height), work.pane.scroll);
    work.pane.scroll = scroll;
    frame.render_widget(Paragraph::new(lines), body_area);
}

/// Records a click target for each spot of the header row `row`.
fn push_hits(
    hits: &mut HitMap,
    spots: &[Spot],
    inner: Rect,
    row: usize,
    target: impl Fn(usize) -> Target,
) {
    let y = inner.y + to_u16(row);
    if y >= inner.bottom() {
        return;
    }
    for (n, spot) in spots.iter().enumerate() {
        let x = (inner.x + spot.x).min(inner.right());
        let width = spot.width.min(inner.right() - x);
        hits.push(Rect::new(x, y, width, 1), target(n));
    }
}

/// `[Affected 1]  Others 46  All 47`: the open view in brackets and bold.
fn tabs(open: PaneView, counts: [usize; 3]) -> (Line<'static>, Vec<Spot>) {
    let mut spans = Vec::new();
    let mut spots = Vec::new();
    let mut x = 0;
    for ((view, name), count) in TABS.iter().zip(counts) {
        if x > 0 {
            spans.push(Span::raw("  "));
            x += 2;
        }
        let label = if *view == open {
            format!("[{name} {count}]")
        } else {
            format!("{name} {count}")
        };
        let width = to_u16(label.width());
        spots.push(Spot { x, width });
        let style = if *view == open {
            theme::bold()
        } else {
            theme::muted()
        };
        spans.push(Span::styled(label, style));
        x += width;
    }
    (Line::from(spans), spots)
}

/// `↔ [short]  long   f filter`: the path style, and a hint for the filter while there is none.
fn options(style: PathStyle, hint: bool) -> (Line<'static>, Spot, Option<Spot>) {
    let mut spans = vec![Span::styled("↔ ", theme::accent())];
    let mut x = 2;
    for (this, name) in [(PathStyle::Short, "short"), (PathStyle::Long, "long")] {
        if this == PathStyle::Long {
            spans.push(Span::raw("  "));
            x += 2;
        }
        let label = if this == style {
            format!("[{name}]")
        } else {
            name.to_string()
        };
        x += to_u16(label.width());
        let text_style = if this == style {
            theme::bold()
        } else {
            theme::muted()
        };
        spans.push(Span::styled(label, text_style));
    }
    let toggle = Spot { x: 0, width: x };
    if !hint {
        return (Line::from(spans), toggle, None);
    }
    spans.push(Span::raw("   "));
    spans.push(Span::styled("f", theme::accent()));
    spans.push(Span::styled(" filter", theme::muted()));
    let spot = Spot { x: x + 3, width: 8 };
    (Line::from(spans), toggle, Some(spot))
}

fn filter_line(filter: &str, filtering: bool) -> Line<'static> {
    let label = if filtering {
        Span::styled("filter: ", theme::warning())
    } else {
        Span::styled("filter: ", theme::muted())
    };
    let mut spans = vec![label, Span::raw(filter.to_string())];
    if !filtering {
        spans.push(Span::styled("  (f edit, esc clear)", theme::muted()));
    }
    Line::from(spans)
}

/// The lines under the header: the rows of the open view, in groups for `All`.
fn body(
    pane: &ProjectPane,
    visible: &[&ProjectRow],
    view: PaneView,
    style: PathStyle,
    filtered: bool,
) -> Vec<Note> {
    if visible.is_empty() {
        let text = match (filtered, view) {
            (true, _) => "No project matches the filter.",
            (false, PaneView::Affected) => "Nothing would change in any project.",
            (false, PaneView::Others) => "Every project is affected.",
            (false, PaneView::All) => "No project.",
        };
        return vec![note(text, Tone::Quiet)];
    }
    let row_note = |row: &&ProjectRow| {
        note(
            format!("{}  {}", row.path(style), row.text).trim_end(),
            row.tone(),
        )
    };
    if !(pane.views && view == PaneView::All) {
        return visible.iter().map(row_note).collect();
    }
    let mut out = Vec::new();
    for (kind, heading) in [
        (Kind::Changing, "Would change"),
        (Kind::Failing, "Cannot be applied"),
        (Kind::Same, "Up to date"),
    ] {
        let members: Vec<&&ProjectRow> = visible.iter().filter(|r| r.kind == kind).collect();
        if members.is_empty() {
            continue;
        }
        out.push(note(
            format!("{heading} ({})", members.len()),
            Tone::Heading,
        ));
        out.extend(members.into_iter().map(row_note));
    }
    out
}

fn note(text: impl Into<String>, tone: Tone) -> Note {
    Note {
        text: text.into(),
        tone,
    }
}
