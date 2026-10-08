//! The page that asks where add writes to, or where init makes the new project.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use super::picker;
use super::setup::{current, done, text};
use crate::hit::HitMap;
use crate::num::to_u16;
use crate::screens::{Place, Purpose, Stage};
use crate::theme;

pub(super) fn draw(frame: &mut Frame, hits: &mut HitMap, place: &mut Place, area: Rect) {
    let head = heading(place);
    let [top, content] =
        Layout::vertical([Constraint::Length(to_u16(head.len())), Constraint::Fill(1)]).areas(area);
    frame.render_widget(Paragraph::new(head).wrap(Wrap { trim: false }), top);
    match place.stage {
        Stage::Folder => picker::draw(frame, hits, &mut place.picker, content),
        Stage::Name => name_field(frame, place, content),
    }
}

fn heading(place: &Place) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    match (place.purpose, place.stage) {
        (Purpose::Add, _) => {
            lines.push(current("Project"));
            lines.push(text("The project that gets the skills."));
            lines.push(text("Walk to it, then press enter inside it:"));
        }
        (Purpose::Init, Stage::Folder) => {
            lines.push(current("Where"));
            lines.push(text("The folder the new project goes in."));
            lines.push(text("Walk to it, then press enter inside it:"));
        }
        (Purpose::Init, Stage::Name) => {
            let parent = place
                .chosen
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            lines.push(done("Where", &parent));
            lines.push(current("Name"));
            lines.push(text("The name of the new project folder:"));
        }
    }
    if place.checking {
        lines.push(Line::from(Span::styled(
            "   Looking at the folder…",
            theme::muted(),
        )));
    }
    if let Some(error) = &place.error {
        for line in error.lines() {
            lines.push(Line::from(Span::styled(
                format!("   ✗ {line}"),
                theme::error(),
            )));
        }
    }
    lines.push(Line::raw(""));
    lines
}

fn name_field(frame: &mut Frame, place: &Place, area: Rect) {
    let width = usize::from(area.width.saturating_sub(6));
    let scroll = place.name.visual_scroll(width);
    let line = Line::from(vec![
        Span::styled("   > ", theme::accent()),
        Span::raw(place.name.value().to_string()),
    ]);
    frame.render_widget(
        Paragraph::new(line).scroll((0, to_u16(scroll))),
        Rect::new(area.x, area.y, area.width, 1),
    );
    let cursor = place.name.visual_cursor().max(scroll) - scroll;
    frame.set_cursor_position((area.x + 5 + to_u16(cursor), area.y));
    if let Some(target) = place.target() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!("   Makes {}", target.display()),
                theme::muted(),
            )),
            Rect::new(area.x, area.y + 2, area.width, 1),
        );
    }
}
