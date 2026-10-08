//! The history page: the runs in the backup store, the question before an undo, and what the undo did.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::{boxed, dialog, page, scroll};
use crate::hit::{HitMap, Target};
use crate::num::{plural, to_u16};
use crate::results::short_path;
use crate::screens::{History, HistoryPhase};
use crate::theme;
use crate::undo::{Step, UndoLine, UndoView};

/// Folders named in the question; the rest are counted.
const LINES_SHOWN: usize = 6;

pub(super) fn draw(
    frame: &mut Frame,
    hits: &mut HitMap,
    hover: Option<Target>,
    tick: usize,
    history: &mut History,
    area: Rect,
) {
    match &history.phase {
        HistoryPhase::Loading => page::waiting(frame, tick, "Reading the backups…", area),
        HistoryPhase::Failed(message) => page::failed(frame, message, area),
        HistoryPhase::List => list(frame, hits, history, area),
        HistoryPhase::Planning => {
            page::waiting(frame, tick, "Checking what the undo will change…", area);
        }
        HistoryPhase::Confirm(_) => {
            list(frame, hits, history, area);
            if let HistoryPhase::Confirm(view) = &history.phase {
                confirm(frame, hits, hover, view, area);
            }
        }
        HistoryPhase::Running { done, total } => {
            page::progress(frame, "Undoing", *done, *total, area);
        }
        HistoryPhase::Done(view) => {
            let mut scroll = history.scroll;
            let all = lines(view, usize::from(area.width));
            let height = usize::from(area.height);
            scroll = scroll.min(all.len().saturating_sub(height));
            let visible: Vec<Line> = all.into_iter().skip(scroll).take(height).collect();
            frame.render_widget(Paragraph::new(visible), area);
            history.scroll = scroll;
        }
    }
}

fn list(frame: &mut Frame, hits: &mut HitMap, history: &mut History, area: Rect) {
    let [head, columns, rows, status] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" History", theme::bold()),
            Span::styled(
                format!("   {}", plural(history.runs.len(), "run")),
                theme::muted(),
            ),
        ])),
        head,
    );
    if history.runs.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "  No backups yet. A sync, push, add, init or delete that replaces or removes a skill keeps its old copy here.",
                theme::muted(),
            ))
            .wrap(ratatui::widgets::Wrap { trim: true }),
            rows,
        );
        return;
    }
    frame.render_widget(
        Paragraph::new(Span::styled(
            format!("    {:<24}{:<9}{:<12}projects", "when", "command", "skills"),
            theme::muted(),
        )),
        columns,
    );
    history.view.viewport = usize::from(rows.height);
    history.view.ensure_visible();
    hits.push(rows, Target::ListArea);
    let offset = history.view.offset;
    for (slot, (index, _)) in history
        .view
        .filtered
        .iter()
        .skip(offset)
        .take(usize::from(rows.height))
        .enumerate()
    {
        let Some(run) = history.runs.get(*index) else {
            continue;
        };
        let position = offset + slot;
        let rect = Rect::new(
            rows.x,
            rows.y + to_u16(slot),
            rows.width.saturating_sub(1),
            1,
        );
        let at_cursor = history.view.cursor == position;
        let mut spans = vec![Span::styled(
            if at_cursor { "> " } else { "  " },
            theme::accent(),
        )];
        let label = if at_cursor {
            theme::bold()
        } else {
            Style::new()
        };
        spans.push(Span::styled(format!("  {:<24}", run.date), label));
        if let Some(error) = &run.error {
            spans.push(Span::styled(
                format!("cannot be read: {error}"),
                theme::error(),
            ));
        } else {
            spans.push(Span::styled(
                format!("{:<9}{:<12}", run.command, run.skills),
                Style::new(),
            ));
            spans.push(Span::styled(run.projects.to_string(), theme::muted()));
        }
        frame.render_widget(Paragraph::new(Line::from(spans)), rect);
        hits.push(rect, Target::Row(position));
    }
    if history.view.len() > usize::from(rows.height) {
        scroll::bar(frame, hits, &history.view, rows);
    }
    let len = history.view.len();
    let text = format!(
        " runs {}–{} of {}",
        (history.view.offset + 1).min(len),
        (history.view.offset + history.view.viewport).min(len),
        len
    );
    frame.render_widget(Paragraph::new(Span::styled(text, theme::muted())), status);
}

fn confirm(
    frame: &mut Frame,
    hits: &mut HitMap,
    hover: Option<Target>,
    view: &UndoView,
    area: Rect,
) {
    let title = format!("Undo the {} of {}?", view.command, view.date);
    let buttons = boxed::draw(frame, hits, area, &title, theme::WARNING, question(view), 1);
    let [yes, no] = Layout::horizontal([Constraint::Length(18), Constraint::Length(18)])
        .flex(Flex::Center)
        .spacing(2)
        .areas(buttons);
    dialog::draw_button(frame, hits, hover, 0, yes, "y  Undo", theme::WARNING);
    dialog::draw_button(frame, hits, hover, 1, no, "n  Cancel", theme::ACCENT);
}

fn question(view: &UndoView) -> Vec<Line<'static>> {
    let restores = count(view, |s| matches!(s, Step::Restore));
    let removes = count(view, |s| matches!(s, Step::Remove));
    let mut doing = Vec::new();
    if restores > 0 {
        doing.push(format!("puts back {}", plural(restores, "skill folder")));
    }
    if removes > 0 {
        doing.push(format!(
            "removes {}",
            plural(removes, "skill the run created")
        ));
    }
    let mut out = vec![Line::raw(format!(
        "{} in {}.",
        capitalized(&doing.join(" and ")),
        plural(view.projects(), "project")
    ))];
    for line in view.lines.iter().take(LINES_SHOWN) {
        out.push(Line::from(Span::styled(
            format!("  {}", describe(line, false)),
            theme::muted(),
        )));
    }
    if view.lines.len() > LINES_SHOWN {
        out.push(Line::from(Span::styled(
            format!("  and {} more", view.lines.len() - LINES_SHOWN),
            theme::muted(),
        )));
    }
    out.push(Line::raw(
        "What the undo replaces is saved first, so undoing it again brings it back.",
    ));
    out
}

fn count(view: &UndoView, which: fn(&Step) -> bool) -> usize {
    view.lines.iter().filter(|l| which(&l.step)).count()
}

fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

/// One folder in words: `restores coding in parent/project`, or `restored` once it happened.
fn describe(line: &UndoLine, done: bool) -> String {
    let place = short_path(&line.project);
    match (&line.step, done) {
        (Step::Restore, false) => format!("restores {} in {place}", line.skill),
        (Step::Restore, true) => format!("restored {} in {place}", line.skill),
        (Step::Remove, false) => format!("removes {} in {place} (the run created it)", line.skill),
        (Step::Remove, true) => format!("removed {} in {place}", line.skill),
        (Step::Gone, _) => format!("{} in {place} is already gone", line.skill),
        (Step::Failed(message), _) => {
            format!("{} in {place}: {}", line.skill, message.replace('\n', "  "))
        }
    }
}

fn lines(view: &UndoView, width: usize) -> Vec<Line<'static>> {
    let title = if view.applied {
        format!(" Undo results: the {} of {}", view.command, view.date)
    } else {
        format!(
            " Nothing can be undone: the {} of {}",
            view.command, view.date
        )
    };
    let mut out = vec![
        Line::from(Span::styled(title, theme::bold())),
        Line::raw(""),
    ];
    for line in &view.lines {
        let (mark, style) = match &line.step {
            Step::Failed(_) => ("✗", theme::error()),
            Step::Gone => ("·", theme::muted()),
            Step::Restore | Step::Remove => ("✓", Style::new().fg(theme::SUCCESS)),
        };
        let mut parts = wrap(&describe(line, view.applied), width.saturating_sub(4)).into_iter();
        if let Some(first) = parts.next() {
            out.push(Line::from(vec![
                Span::styled(format!("  {mark} "), style),
                Span::raw(first),
            ]));
        }
        out.extend(parts.map(|part| Line::raw(format!("    {part}"))));
    }
    out.push(Line::raw(""));
    for warning in &view.warnings {
        out.push(Line::from(Span::styled(
            format!("  ! {warning}"),
            theme::warning(),
        )));
    }
    if let Some(id) = &view.saved_as {
        out.push(Line::from(Span::styled(
            format!("  Saved what was replaced as run {id}"),
            theme::muted(),
        )));
        out.push(Line::from(Span::styled(
            "  (undo it again to redo this undo)",
            theme::muted(),
        )));
    }
    if view.failed() > 0 {
        out.push(Line::from(Span::styled(
            format!(
                "  {} failed; nothing was left half done.",
                plural(view.failed(), "folder")
            ),
            theme::error(),
        )));
    }
    out
}

/// Breaks text at spaces into lines of at most `width` characters; a word longer than that stays whole.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let joined = current.chars().count() + 1 + word.chars().count();
        if !current.is_empty() && joined > width {
            lines.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() || lines.is_empty() {
        lines.push(current);
    }
    lines
}
