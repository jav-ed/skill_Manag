//! The page that shows what the plan of a question would change.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::diffview::{DiffLine, Tone};
use crate::screens::DiffPage;
use crate::theme;

pub(super) fn draw(frame: &mut Frame, page: &mut DiffPage, area: Rect) {
    let [head, body] = Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).areas(area);
    let what = if page.lines.is_empty() {
        "Nothing would change."
    } else {
        "- is taken from the project, + comes from the vault"
    };
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(" Changes", theme::bold())),
            Line::from(Span::styled(format!(" {what}"), theme::muted())),
        ]),
        head,
    );
    let height = usize::from(body.height);
    page.scroll = page.scroll.min(page.lines.len().saturating_sub(height));
    let visible: Vec<Line> = page
        .lines
        .iter()
        .skip(page.scroll)
        .take(height)
        .map(styled)
        .collect();
    frame.render_widget(Paragraph::new(visible), body);
}

fn styled(line: &DiffLine) -> Line<'static> {
    let style = match line.tone {
        Tone::Skill => theme::accent().add_modifier(ratatui::style::Modifier::BOLD),
        Tone::File => theme::bold(),
        Tone::Added => Style::new().fg(theme::SUCCESS),
        Tone::Removed => Style::new().fg(theme::ERROR),
        Tone::Hunk => theme::accent(),
        Tone::Plain => Style::new(),
        Tone::Note => theme::muted(),
    };
    Line::from(Span::styled(format!(" {}", line.text), style))
}
