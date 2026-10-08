//! The list of what the scan could not read.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::boxed;
use crate::hit::HitMap;
use crate::session::Issue;
use crate::theme;

/// Problems named in the box; the rest are counted, and `doctor` lists them all.
const SHOWN: usize = 5;

pub(super) fn draw(frame: &mut Frame, hits: &mut HitMap, issues: &[Issue], area: Rect) {
    let mut body = Vec::new();
    for issue in issues.iter().take(SHOWN) {
        body.push(Line::from(Span::styled(
            issue.path.clone(),
            theme::accent(),
        )));
        body.push(Line::from(Span::styled(
            format!("  {}", issue.message),
            theme::muted(),
        )));
    }
    if issues.len() > SHOWN {
        body.push(Line::from(Span::styled(
            format!(
                "and {} more; `skillmirror doctor` lists them all",
                issues.len() - SHOWN
            ),
            theme::muted(),
        )));
    }
    let hint = boxed::draw(frame, hits, area, "Scan problems", theme::WARNING, body, 1);
    frame.render_widget(
        Paragraph::new(Span::styled(" any key closes", theme::muted())),
        hint,
    );
}
