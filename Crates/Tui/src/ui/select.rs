//! The selection table of sync, push, delete and list.

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::{Frame, symbols};
use unicode_width::UnicodeWidthStr;

use crate::hit::{HitMap, Target};
use crate::items::{Item, Mode, Tone};
use crate::num::{plural, to_u16};
use crate::screens::Work;
use crate::theme;

const NAME_MIN: usize = 18;
const NAME_MAX: usize = 36;
const DETAIL_MAX: usize = 40;

/// The colour that marks selection on this screen, as in the Go tool.
fn mode_style(mode: Mode) -> Style {
    match mode {
        Mode::Sync => Style::new().fg(theme::SUCCESS),
        Mode::Push => theme::warning(),
        Mode::Delete => Style::new().fg(theme::ERROR),
        Mode::List => theme::accent(),
    }
}

fn title(work: &Work) -> String {
    match work.mode {
        Mode::Sync => "Select skills to sync".to_string(),
        Mode::Push => "Select skills to push".to_string(),
        Mode::Delete => "Select skills to delete".to_string(),
        Mode::List if work.filter.value().is_empty() => {
            format!("Skills — {} installed", work.items.len())
        }
        Mode::List => format!(
            "Skills — {} matching {:?}",
            work.view.len(),
            work.filter.value()
        ),
    }
}

pub(super) fn draw(frame: &mut Frame, hits: &mut HitMap, work: &mut Work, area: Rect) {
    let [head, filter, columns, rows, status] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);
    let hidden = work.hidden_selected();
    let hidden_note = if hidden > 0 {
        format!(" ({hidden} hidden by the filter)")
    } else {
        String::new()
    };
    let selected = format!(
        "   {} / {} selected{hidden_note}",
        work.selected.len(),
        work.items.len()
    );
    let mut heading = vec![
        Span::styled(format!(" {}", title(work)), theme::bold()),
        Span::styled(selected, theme::muted()),
    ];
    if !work.issues.is_empty() {
        heading.push(Span::styled(
            format!("   {} (i)", plural(work.issues.len(), "scan problem")),
            theme::warning(),
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(heading)), head);
    draw_filter(frame, work, filter);
    let (name_w, detail_w) = widths(&work.items);
    let heading = format!("      {:<name_w$}  {:<detail_w$}", "skill", "projects");
    let heading = if work.mode == Mode::List {
        heading.replace("projects", "project ")
    } else {
        heading
    };
    frame.render_widget(
        Paragraph::new(Span::styled(heading, theme::muted())),
        columns,
    );
    if work.items.is_empty() || work.view.len() == 0 {
        let message = if work.items.is_empty() {
            work.empty_message()
        } else {
            "No skill matches the filter."
        };
        frame.render_widget(
            Paragraph::new(Span::styled(format!("  {message}"), theme::muted())),
            rows,
        );
        return;
    }
    draw_rows(frame, hits, work, rows, (name_w, detail_w));
    draw_status(frame, work, status);
}

fn draw_filter(frame: &mut Frame, work: &Work, area: Rect) {
    if !work.filtering {
        let text = if work.filter.value().is_empty() {
            " / to filter".to_string()
        } else {
            format!(" filter: {}  (esc to clear)", work.filter.value())
        };
        frame.render_widget(Paragraph::new(Span::styled(text, theme::muted())), area);
        return;
    }
    let width = usize::from(area.width.saturating_sub(10));
    let scroll = work.filter.visual_scroll(width);
    let line = Line::from(vec![
        Span::styled(" filter: ", theme::warning()),
        Span::raw(work.filter.value().to_string()),
    ]);
    frame.render_widget(Paragraph::new(line).scroll((0, to_u16(scroll))), area);
    let cursor = work.filter.visual_cursor().max(scroll) - scroll;
    frame.set_cursor_position((area.x + 9 + to_u16(cursor), area.y));
}

fn widths(items: &[Item]) -> (usize, usize) {
    let name = items.iter().map(|i| i.name.width()).max().unwrap_or(0);
    let detail = items.iter().map(|i| i.detail.width()).max().unwrap_or(0);
    (name.clamp(NAME_MIN, NAME_MAX), detail.clamp(8, DETAIL_MAX))
}

fn draw_rows(
    frame: &mut Frame,
    hits: &mut HitMap,
    work: &mut Work,
    area: Rect,
    (name_w, detail_w): (usize, usize),
) {
    work.view.viewport = usize::from(area.height);
    work.view.ensure_visible();
    hits.push(area, Target::ListArea);
    let accent = mode_style(work.mode);
    let mark = if work.mode == Mode::Delete {
        "[✗]"
    } else {
        "[✓]"
    };
    let offset = work.view.offset;
    for (slot, (index, matched)) in work
        .view
        .filtered
        .iter()
        .skip(offset)
        .take(usize::from(area.height))
        .enumerate()
    {
        let Some(item) = work.items.get(*index) else {
            continue;
        };
        let position = offset + slot;
        let rect = Rect::new(
            area.x,
            area.y + to_u16(slot),
            area.width.saturating_sub(1),
            1,
        );
        let at_cursor = work.view.cursor == position;
        let on = work.selected.contains(index);
        let mut spans = vec![
            Span::styled(if at_cursor { "> " } else { "  " }, accent),
            Span::styled(
                if on { mark } else { "[ ]" },
                if on { accent } else { Style::new() },
            ),
            Span::raw(" "),
        ];
        spans.extend(highlight(&item.name, matched, at_cursor));
        let pad = name_w.saturating_sub(item.name.width()) + 2;
        spans.push(Span::raw(" ".repeat(pad)));
        spans.push(Span::styled(
            format!("{:<detail_w$}", item.detail),
            theme::muted(),
        ));
        if let Some(note) = &item.note {
            spans.push(Span::styled(
                format!("  {}", note.text),
                tone_style(note.tone),
            ));
        }
        frame.render_widget(Paragraph::new(Line::from(spans)), rect);
        hits.push(rect, Target::Row(position));
    }
    if work.view.len() > usize::from(area.height) {
        draw_scrollbar(frame, hits, work, area);
    }
}

fn draw_scrollbar(frame: &mut Frame, hits: &mut HitMap, work: &Work, area: Rect) {
    let track = Rect::new(area.right().saturating_sub(1), area.y, 1, area.height);
    hits.push(track, Target::ScrollTrack);
    let mut state = ScrollbarState::new(work.view.len())
        .position(work.view.offset)
        .viewport_content_length(usize::from(area.height));
    let bar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None)
        .track_symbol(Some(symbols::line::VERTICAL));
    frame.render_stateful_widget(bar, track, &mut state);
}

fn tone_style(tone: Tone) -> Style {
    match tone {
        Tone::Change => theme::accent(),
        Tone::Quiet => theme::muted(),
        Tone::Problem => theme::error(),
    }
}

/// The name with the fuzzy-matched characters in bold yellow.
fn highlight(name: &str, matched: &[u32], at_cursor: bool) -> Vec<Span<'static>> {
    let base = if at_cursor {
        Style::new().add_modifier(Modifier::BOLD)
    } else {
        Style::new()
    };
    name.chars()
        .enumerate()
        .map(|(i, c)| {
            let hit = u32::try_from(i).is_ok_and(|i| matched.binary_search(&i).is_ok());
            let style = if hit {
                base.fg(theme::WARNING).add_modifier(Modifier::BOLD)
            } else {
                base
            };
            Span::styled(c.to_string(), style)
        })
        .collect()
}

fn draw_status(frame: &mut Frame, work: &Work, area: Rect) {
    let view = &work.view;
    let len = view.len();
    let text = format!(
        " rows {}–{} of {}",
        (view.offset + 1).min(len),
        (view.offset + view.viewport).min(len),
        len
    );
    frame.render_widget(Paragraph::new(Span::styled(text, theme::muted())), area);
}
