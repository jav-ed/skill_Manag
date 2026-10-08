//! The directory picker.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};

use crate::hit::{HitMap, Target};
use crate::num::to_u16;
use crate::screens::Picker;
use crate::theme;

pub(super) fn draw(frame: &mut Frame, hits: &mut HitMap, picker: &mut Picker, area: Rect) {
    let [bar, list, foot] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);
    let up = Rect::new(bar.x, bar.y, 4.min(bar.width), 1);
    frame.render_widget(Paragraph::new(Span::styled(" ⬑  ", theme::accent())), up);
    hits.push(up, Target::PickerUp);
    let path = Rect::new(
        bar.x + 4.min(bar.width),
        bar.y,
        bar.width.saturating_sub(4),
        1,
    );
    frame.render_widget(Paragraph::new(picker.cwd.display().to_string()), path);

    picker.view.viewport = usize::from(list.height);
    picker.view.ensure_visible();
    let offset = picker.view.offset;
    for (slot, name) in picker
        .names
        .iter()
        .skip(offset)
        .take(usize::from(list.height))
        .enumerate()
    {
        let index = offset + slot;
        let row = Rect::new(
            list.x,
            list.y + to_u16(slot),
            list.width.saturating_sub(1),
            1,
        );
        let at = index == picker.view.cursor;
        let style = if at {
            theme::accent().add_modifier(Modifier::BOLD)
        } else {
            ratatui::style::Style::new()
        };
        let text = format!("{}{name}/", if at { "> " } else { "  " });
        frame.render_widget(Paragraph::new(Line::styled(text, style)), row);
        hits.push(row, Target::PickerRow(index));
    }
    if picker.names.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled("  (no sub-directories)", theme::muted())),
            list,
        );
    }
    if picker.names.len() > usize::from(list.height) {
        let track = Rect::new(list.right().saturating_sub(1), list.y, 1, list.height);
        let mut state = ScrollbarState::new(picker.names.len())
            .position(offset)
            .viewport_content_length(usize::from(list.height));
        let bar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None);
        frame.render_stateful_widget(bar, track, &mut state);
    }
    draw_footer(frame, hits, picker, foot);
}

fn draw_footer(frame: &mut Frame, hits: &mut HitMap, picker: &Picker, area: Rect) {
    let hidden = Rect::new(area.x, area.y, 18.min(area.width), 1);
    let mark = if picker.show_hidden { "[x]" } else { "[ ]" };
    frame.render_widget(Paragraph::new(format!(" {mark} hidden (.)")), hidden);
    hits.push(hidden, Target::PickerHidden);
    let rest = Rect::new(
        area.x + hidden.width,
        area.y,
        area.width.saturating_sub(hidden.width),
        1,
    );
    if let Some(error) = &picker.error {
        frame.render_widget(
            Paragraph::new(Span::styled(format!("error: {error}"), theme::error())),
            rest,
        );
    } else {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "[ choose this folder: enter ]",
                theme::accent(),
            )),
            rest,
        );
        hits.push(rest, Target::PickerSelect);
    }
}
