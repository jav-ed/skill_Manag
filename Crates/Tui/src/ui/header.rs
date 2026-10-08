//! The top line: back arrow, program name, link to the author's site, current screen.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthStr;

use super::link::Link;
use crate::app::SITE_URL;
use crate::hit::{HitMap, Target};
use crate::num::to_u16;
use crate::theme;

const SITE_NAME: &str = "javedab.com";

pub(super) fn draw(
    frame: &mut Frame,
    hits: &mut HitMap,
    hover: Option<Target>,
    screen: Option<&str>,
    area: Rect,
) {
    let mut spans = Vec::new();
    if screen.is_some() {
        let style = if hover == Some(Target::HeaderBack) {
            theme::accent().add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            theme::accent()
        };
        spans.push(Span::styled(" ← ", style));
        hits.push(Rect::new(area.x, area.y, 3, 1), Target::HeaderBack);
    } else {
        spans.push(Span::raw("   "));
    }
    spans.push(Span::styled("skillmirror", theme::bold()));
    spans.push(Span::styled("  ·  ", theme::muted()));
    let link_x = area.x + to_u16(spans.iter().map(Span::width).sum());
    let link_width = to_u16(SITE_NAME.width());
    spans.push(Span::raw(" ".repeat(usize::from(link_width))));
    if let Some(name) = screen {
        spans.push(Span::styled(format!("  ·  {name}"), theme::muted()));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);

    let link_area = Rect::new(link_x, area.y, link_width, 1).intersection(area);
    let style = if hover == Some(Target::HeaderLink) {
        theme::accent().add_modifier(Modifier::UNDERLINED)
    } else {
        theme::muted().add_modifier(Modifier::UNDERLINED)
    };
    frame.render_widget(
        Link {
            text: SITE_NAME,
            url: SITE_URL,
            style,
        },
        link_area,
    );
    hits.push(link_area, Target::HeaderLink);
}
