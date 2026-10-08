//! The page after a run: one entry per skill, failures spelled out, details on request.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthStr;

use crate::num::plural;
use crate::results::{Kind, Results, SkillResult, short_path};
use crate::theme;

pub(super) fn draw(
    frame: &mut Frame,
    results: &Results,
    scroll: &mut usize,
    details: bool,
    area: Rect,
) {
    let lines = lines(results, details);
    let height = usize::from(area.height);
    *scroll = (*scroll).min(lines.len().saturating_sub(height));
    let visible: Vec<Line> = lines.into_iter().skip(*scroll).take(height).collect();
    frame.render_widget(Paragraph::new(visible), area);
}

pub(super) fn lines(results: &Results, details: bool) -> Vec<Line<'static>> {
    let mut out = vec![
        Line::from(Span::styled(
            format!(" {}", results.kind.title()),
            theme::bold(),
        )),
        Line::raw(""),
    ];
    if results.skills.is_empty() {
        out.push(Line::from(Span::styled(
            "  Nothing was selected.",
            theme::muted(),
        )));
    }
    let name_w = results
        .skills
        .iter()
        .map(|s| s.name.width())
        .max()
        .unwrap_or(0)
        .max(18);
    for skill in &results.skills {
        out.push(summary(results.kind, skill, name_w));
        for line in skill.lines.iter().filter(|l| details || !l.ok) {
            let (mark, style) = if line.ok {
                ("✓", Style::new().fg(theme::SUCCESS))
            } else {
                ("✗", theme::error())
            };
            out.push(Line::from(vec![
                Span::styled(format!("    {mark} "), style),
                Span::raw(short_path(&line.project)),
                Span::styled(
                    format!("  {}", line.text.replace('\n', "  ")),
                    theme::muted(),
                ),
            ]));
        }
    }
    out.push(Line::raw(""));
    for warning in &results.warnings {
        out.push(Line::from(Span::styled(
            format!("  ! {warning}"),
            theme::warning(),
        )));
    }
    if let Some(id) = &results.backup {
        out.push(Line::from(Span::styled(
            format!("  Backup: run {id} (undo with: skillmirror undo)"),
            theme::muted(),
        )));
    }
    let failed = results.failed();
    if failed > 0 {
        out.push(Line::from(Span::styled(
            format!(
                "  {} failed; nothing was left half written.",
                plural(failed, "project")
            ),
            theme::error(),
        )));
    }
    out
}

fn summary(kind: Kind, skill: &SkillResult, name_w: usize) -> Line<'static> {
    let (icon, style) = if skill.failed == 0 {
        ("✓", Style::new().fg(theme::SUCCESS))
    } else if skill.ok + skill.unchanged == 0 {
        ("✗", theme::error())
    } else {
        ("!", theme::warning())
    };
    let pad = " ".repeat(name_w.saturating_sub(skill.name.width()));
    let what = match kind {
        Kind::Delete => format!("{} {}", kind.verb(), plural(skill.ok, "project")),
        _ if skill.ok > 0 => format!(
            "{} {} ({})",
            kind.verb(),
            plural(skill.ok, "project"),
            plural(skill.files, "file")
        ),
        _ if skill.failed == 0 => {
            format!(
                "already up to date in {}",
                plural(skill.unchanged, "project")
            )
        }
        _ => String::new(),
    };
    let mut spans = vec![
        Span::styled(format!("  {icon} "), style),
        Span::styled(skill.name.clone(), theme::accent()),
        Span::raw(format!("{pad}  {what}")),
    ];
    if kind != Kind::Delete && skill.ok > 0 && skill.unchanged > 0 {
        spans.push(Span::styled(
            format!("  · {} up to date", skill.unchanged),
            theme::muted(),
        ));
    }
    if skill.failed > 0 {
        spans.push(Span::styled(
            format!("  {} failed", skill.failed),
            theme::error(),
        ));
    }
    Line::from(spans)
}
