//! The lines around the rows: the header with its buttons, where the page is, the names of the two sides, and
//! the heads of groups and files, the notes, the folds and the targets that failed.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use skillmirror_core::ops::DiffKind;
use unicode_width::UnicodeWidthStr;

use crate::hit::{HitMap, Target};
use crate::num::{plural, to_u16};
use crate::results::short_path;
use crate::screens::diff::{
    CHANGE_MARGIN, DiffView, FOLDS, Item, Layout, MODE, NEXT, PREVIOUS, ViewMode,
};
use crate::theme;

/// A button: what it is called in each state, which of them is the state now, and what it does.
struct Button {
    id: usize,
    /// The words, the one that is on first. A button with one word does one thing.
    words: Vec<&'static str>,
    on: usize,
}

fn buttons(view: &DiffView) -> Vec<Button> {
    vec![
        Button {
            id: PREVIOUS,
            words: vec!["◀ prev"],
            on: 0,
        },
        Button {
            id: NEXT,
            words: vec!["next ▶"],
            on: 0,
        },
        Button {
            id: MODE,
            words: vec!["split", "inline"],
            on: usize::from(view.mode == ViewMode::Inline),
        },
        Button {
            id: FOLDS,
            words: vec!["folded", "whole"],
            on: usize::from(!view.fold),
        },
    ]
}

/// ` Changes   3 projects, 4 files, +5 −3` and the buttons at the right end, when they fit.
pub(super) fn title(
    frame: &mut Frame,
    hits: &mut HitMap,
    hover: Option<Target>,
    view: &DiffView,
    area: Rect,
) {
    let totals = view.totals();
    let summary = if view.report.groups.is_empty() {
        String::new()
    } else {
        format!(
            "   {} in {}, +{} −{}",
            plural(totals.files, "file"),
            plural(totals.projects, "project"),
            totals.added,
            totals.removed
        )
    };
    let mut spans = vec![
        Span::styled(" Changes", theme::bold()),
        Span::styled(summary, theme::muted()),
    ];
    let used: usize = spans.iter().map(|s| s.content.width()).sum();
    let mut parts: Vec<(Vec<Span<'static>>, usize, usize)> = Vec::new();
    if !view.report.groups.is_empty() {
        for button in buttons(view) {
            let mut inner = Vec::new();
            for (i, word) in button.words.iter().enumerate() {
                if i > 0 {
                    inner.push(Span::raw(" "));
                }
                let on = button.words.len() > 1 && i == button.on;
                let text = if on {
                    format!("[{word}]")
                } else {
                    (*word).to_string()
                };
                let hovered = hover == Some(Target::DiffButton(button.id));
                let mut style = if on || button.words.len() == 1 {
                    theme::accent()
                } else {
                    theme::muted()
                };
                if hovered {
                    style = style.add_modifier(Modifier::UNDERLINED);
                }
                inner.push(Span::styled(text, style));
            }
            let width = inner.iter().map(|s| s.content.width()).sum();
            parts.push((inner, button.id, width));
        }
    }
    let gaps = 3 * parts.len().saturating_sub(1);
    let wide: usize = parts.iter().map(|part| part.2).sum::<usize>() + gaps + 1;
    if used + 2 + wide <= usize::from(area.width) {
        let mut x = usize::from(area.width) - wide;
        spans.push(Span::raw(" ".repeat(x - used)));
        for (n, (inner, id, width)) in parts.into_iter().enumerate() {
            if n > 0 {
                spans.push(Span::raw("   "));
                x += 3;
            }
            hits.push(
                Rect::new(area.x + to_u16(x), area.y, to_u16(width), 1),
                Target::DiffButton(id),
            );
            x += width;
            spans.extend(inner);
        }
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// Where the page is: the skill and the file at the top of the window, and the change it has got to.
pub(super) fn place(frame: &mut Frame, view: &DiffView, area: Rect) {
    let Some(layout) = view.layout.as_ref() else {
        return;
    };
    let (group, file) = match layout
        .items
        .get(layout.index_at(view.scroll))
        .map(|p| &p.item)
    {
        Some(Item::Group { group, .. } | Item::Blank(group)) => (Some(*group), None),
        Some(
            Item::File { group, file }
            | Item::Note { group, file, .. }
            | Item::Row { group, file, .. }
            | Item::Fold {
                id: (group, file, _),
                ..
            },
        ) => (Some(*group), Some(*file)),
        _ => (None, None),
    };
    let group = group.and_then(|g| view.report.groups.get(g));
    let mut spans = vec![Span::raw(" ")];
    if let Some(group) = group {
        spans.push(Span::styled(group.skill.clone(), theme::accent()));
        if let Some(file) = file.and_then(|f| group.files.get(f)) {
            spans.push(Span::styled(
                format!("  {}", file.path.display()),
                theme::muted(),
            ));
        }
    }
    let changes = layout.changes.len();
    if changes > 0 {
        let text = if view.max_scroll() == 0 {
            // Nothing scrolls, so no jump has walked anywhere: all of them are in view.
            plural(changes, "change")
        } else if view.scroll >= view.max_scroll() {
            // At the end every change was reached, also those the last jump could not scroll up to.
            format!("change {changes} of {changes}")
        } else {
            // The changes the jumps have passed: those whose place is at or above the top of the window.
            let seen = layout
                .changes
                .partition_point(|top| top.saturating_sub(CHANGE_MARGIN) <= view.scroll);
            if seen == 0 {
                plural(changes, "change")
            } else {
                format!("change {seen} of {changes}")
            }
        };
        spans.push(Span::styled(format!("   {text}"), theme::muted()));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// What the two sides are, over their columns.
pub(super) fn labels(frame: &mut Frame, view: &DiffView, area: Rect) {
    let Some(layout) = view.layout.as_ref() else {
        return;
    };
    let red = Style::new().fg(theme::ERROR);
    let green = Style::new().fg(theme::SUCCESS);
    let labels = view.labels;
    let line = if layout.split {
        let gutter = layout.num_w + 3;
        let old = format!("− {}", labels.old);
        let to_new = (layout.text_w + 1 + gutter).saturating_sub(old.width());
        Line::from(vec![
            Span::raw(" ".repeat(gutter)),
            Span::styled("−", red),
            Span::styled(format!(" {}", labels.old), theme::muted()),
            Span::raw(" ".repeat(to_new)),
            Span::styled("+", green),
            Span::styled(format!(" {}", labels.new), theme::muted()),
        ])
    } else {
        Line::from(vec![
            Span::raw(" "),
            Span::styled("−", red),
            Span::styled(format!(" {}   ", labels.old), theme::muted()),
            Span::styled("+", green),
            Span::styled(format!(" {}", labels.new), theme::muted()),
        ])
    };
    frame.render_widget(Paragraph::new(line), area);
}

fn missing() -> Vec<Line<'static>> {
    vec![Line::from(Span::styled(
        " internal error: the group is not in the report",
        theme::error(),
    ))]
}

/// `▌ skill   the same change in 18 projects`, and under it the names of the projects.
pub(super) fn group(view: &DiffView, group: usize, names: &[String]) -> Vec<Line<'static>> {
    let Some(group) = view.report.groups.get(group) else {
        return missing();
    };
    let what = match group.targets.as_slice() {
        [only] => format!("   in {}", short_path(&only.project.to_string_lossy())),
        many => format!("   the same change in {} projects", many.len()),
    };
    let mut lines = vec![Line::from(vec![
        Span::styled("▌ ", theme::accent()),
        Span::styled(
            group.skill.clone(),
            theme::accent().add_modifier(Modifier::BOLD),
        ),
        Span::styled(what, theme::muted()),
    ])];
    lines.extend(
        names
            .iter()
            .map(|name| Line::from(Span::styled(format!("  {name}"), theme::muted()))),
    );
    lines
}

/// `  Repository/what_Must_Hold.md  changed  +3 −3`.
pub(super) fn file(view: &DiffView, group: usize, file: usize) -> Vec<Line<'static>> {
    let Some(file) = view
        .report
        .groups
        .get(group)
        .and_then(|group| group.files.get(file))
    else {
        return missing();
    };
    let what = match file.kind {
        DiffKind::Added => "new file",
        DiffKind::Modified => "changed",
        DiffKind::ModeChanged => "permissions",
        DiffKind::Removed => "removed",
    };
    let mut spans = vec![
        Span::raw("  "),
        Span::styled(file.path.display().to_string(), theme::bold()),
        Span::styled(format!("  {what}"), theme::muted()),
    ];
    if file.added + file.removed > 0 {
        spans.push(Span::styled(
            format!("  +{}", file.added),
            Style::new().fg(theme::SUCCESS),
        ));
        spans.push(Span::styled(
            format!(" −{}", file.removed),
            Style::new().fg(theme::ERROR),
        ));
    }
    vec![Line::from(spans)]
}

/// Lines of explanation, indented: why a file has no lines, or where the page stops.
pub(super) fn note(lines: &[String]) -> Vec<Line<'static>> {
    lines
        .iter()
        .map(|line| Line::from(Span::styled(format!("    {line}"), theme::muted())))
        .collect()
}

/// `⋯ 42 lines unchanged`, over the whole width, for a click to open.
pub(super) fn fold(hidden: usize, layout: &Layout) -> Vec<Line<'static>> {
    let text = format!(
        "{}⋯ {} unchanged  (click to show)",
        " ".repeat(layout.num_w + 3),
        plural(hidden, "line")
    );
    let pad = layout.key.width.saturating_sub(text.width());
    vec![Line::from(Span::styled(
        format!("{text}{}", " ".repeat(pad)),
        Style::new().fg(theme::MUTED).bg(theme::FOLD_LINE),
    ))]
}

pub(super) fn failed_head() -> Vec<Line<'static>> {
    vec![Line::from(Span::styled(
        " Cannot be applied",
        theme::error(),
    ))]
}

pub(super) fn failed(lines: &[String]) -> Vec<Line<'static>> {
    lines
        .iter()
        .map(|line| Line::from(Span::styled(format!("    {line}"), theme::warning())))
        .collect()
}
