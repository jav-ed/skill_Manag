//! The main menu: one entry per screen and the long description of the selected entry below.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use crate::hit::{HitMap, Target};
use crate::num::to_u16;
use crate::screens::{ENTRIES, Menu};
use crate::theme;

const TAGLINE: &str = "Sync agent skills across all your projects from a single vault.";
const ROWS_PER_ENTRY: u16 = 3;
/// Lines the long text under the entries needs: the divider and a few lines of text.
const DETAIL_ROWS: u16 = 5;

pub(super) fn draw(frame: &mut Frame, hits: &mut HitMap, menu: &Menu, area: Rect) {
    frame.render_widget(
        Paragraph::new(Span::styled(format!("  {TAGLINE}"), theme::muted())),
        Rect::new(area.x, area.y, area.width, 1),
    );
    let top = area.y + 2;
    // Two lines per entry when the long text below still fits, else one line each.
    let roomy = area.height >= 2 + ROWS_PER_ENTRY * to_u16(ENTRIES.len()) + DETAIL_ROWS;
    let rows_per_entry = if roomy { ROWS_PER_ENTRY } else { 1 };
    let label_w = ENTRIES.iter().map(|e| e.label.len()).max().unwrap_or(0) + 2;
    for (i, entry) in ENTRIES.iter().enumerate() {
        let row = Rect::new(
            area.x + 2,
            top + to_u16(i) * rows_per_entry,
            area.width.saturating_sub(4),
            if roomy { 2 } else { 1 },
        );
        if row.bottom() > area.bottom() {
            break;
        }
        let selected = menu.cursor == i;
        let (bar, label) = if selected {
            (
                Span::styled("│ ", theme::accent()),
                Span::styled(
                    entry.label,
                    theme::accent().add_modifier(ratatui::style::Modifier::BOLD),
                ),
            )
        } else {
            (Span::raw("  "), Span::raw(entry.label))
        };
        let lines = if roomy {
            vec![
                Line::from(vec![bar.clone(), label]),
                Line::from(vec![bar, Span::styled(entry.blurb, theme::muted())]),
            ]
        } else {
            let pad = " ".repeat(label_w.saturating_sub(entry.label.len()));
            vec![Line::from(vec![
                bar,
                label,
                Span::raw(pad),
                Span::styled(entry.blurb, theme::muted()),
            ])]
        };
        frame.render_widget(Paragraph::new(lines), row);
        hits.push(row, Target::MenuItem(i));
    }
    draw_detail(
        frame,
        menu,
        area,
        top + to_u16(ENTRIES.len()) * rows_per_entry,
    );
}

fn draw_detail(frame: &mut Frame, menu: &Menu, area: Rect, y: u16) {
    if y >= area.bottom() {
        return;
    }
    let width = area.width.saturating_sub(4);
    let divider = Rect::new(area.x + 2, y, width, 1);
    frame.render_widget(
        Paragraph::new(Span::styled("─".repeat(usize::from(width)), theme::muted())),
        divider,
    );
    let Some(entry) = ENTRIES.get(menu.cursor) else {
        return;
    };
    let text = Rect::new(
        area.x + 2,
        y + 1,
        width,
        area.bottom().saturating_sub(y + 1),
    );
    frame.render_widget(Paragraph::new(entry.detail).wrap(Wrap { trim: true }), text);
}
