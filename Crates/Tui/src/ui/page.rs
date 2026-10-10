//! The pages that are not tables: scanning, running and failed.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{LineGauge, Paragraph, Wrap};

use crate::num::ratio;
use crate::results::Kind;
use crate::theme;

const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub(super) fn loading(frame: &mut Frame, tick: usize, area: Rect) {
    waiting(frame, tick, "Scanning projects…", area);
}

/// A spinner and a line of what is being waited for.
pub(super) fn waiting(frame: &mut Frame, tick: usize, what: &str, area: Rect) {
    let glyph = SPINNER.get(tick % SPINNER.len()).copied().unwrap_or("·");
    let line = Line::from(vec![
        Span::styled(format!("  {glyph}  "), theme::accent()),
        Span::styled(what.to_string(), theme::muted()),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

/// A gauge with a verb and the count, for the jobs that write.
pub(super) fn progress(frame: &mut Frame, verb: &str, done: usize, total: usize, area: Rect) {
    let [label, gauge] =
        Layout::vertical([Constraint::Length(2), Constraint::Length(1)]).areas(area);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!("  {verb}  {done} / {total}"),
            theme::bold(),
        ))),
        label,
    );
    frame.render_widget(
        LineGauge::default()
            .filled_style(theme::accent())
            .unfilled_style(theme::muted())
            .label("")
            .ratio(ratio(done, total)),
        gauge,
    );
}

pub(super) fn planning(frame: &mut Frame, kind: Kind, tick: usize, area: Rect) {
    let what = match kind {
        Kind::Sync => "the sync",
        Kind::Push => "the push",
        Kind::Delete => "the delete",
        Kind::Agents => "the AGENTS.md write",
        Kind::Add => "the add",
        Kind::Init => "the new project",
    };
    waiting(
        frame,
        tick,
        &format!("Checking what {what} will change…"),
        area,
    );
}

pub(super) fn failed(frame: &mut Frame, message: &str, area: Rect) {
    let mut lines: Vec<Line> = message
        .lines()
        .enumerate()
        .map(|(i, text)| {
            let style = if i == 0 {
                theme::error()
            } else {
                theme::muted()
            };
            Line::from(Span::styled(format!("  {text}"), style))
        })
        .collect();
    if let Some(first) = lines.first_mut() {
        first.spans.insert(0, Span::styled("✗", theme::error()));
    }
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
}

pub(super) fn running(frame: &mut Frame, kind: Kind, done: usize, total: usize, area: Rect) {
    let verb = match kind {
        Kind::Sync => "Syncing",
        Kind::Push => "Pushing",
        Kind::Delete => "Deleting",
        Kind::Agents => "Writing",
        Kind::Add => "Adding",
        Kind::Init => "Installing",
    };
    progress(frame, verb, done, total, area);
}
