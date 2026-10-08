//! The setup wizard page.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use super::picker;
use crate::hit::{HitMap, Target};
use crate::num::to_u16;
use crate::screens::{Setup, Step};
use crate::theme;

pub(super) fn draw(
    frame: &mut Frame,
    hits: &mut HitMap,
    hover: Option<Target>,
    setup: &mut Setup,
    area: Rect,
) {
    let head = heading(setup);
    let [top, content] =
        Layout::vertical([Constraint::Length(to_u16(head.len())), Constraint::Fill(1)]).areas(area);
    frame.render_widget(Paragraph::new(head).wrap(Wrap { trim: false }), top);
    match setup.step {
        Step::Vault | Step::Root => picker::draw(frame, hits, &mut setup.picker, content),
        Step::Mandatory => checklist(frame, hits, setup, content),
        Step::Save | Step::Saved(_) => final_page(frame, hits, hover, setup, content),
    }
}

pub(super) fn done(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(" ✓ ", Style::new().fg(theme::SUCCESS)),
        Span::styled(format!("{label:<10}"), theme::bold()),
        Span::styled(value.to_string(), theme::muted()),
    ])
}

pub(super) fn current(label: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!(" → {label}"),
        theme::accent().add_modifier(Modifier::BOLD),
    ))
}

pub(super) fn text(line: &str) -> Line<'static> {
    Line::from(Span::raw(format!("   {line}")))
}

fn heading(setup: &Setup) -> Vec<Line<'static>> {
    let path = |p: &Option<std::path::PathBuf>| {
        p.as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default()
    };
    let mut lines = Vec::new();
    match &setup.step {
        Step::Vault => {
            lines.push(current("Vault"));
            lines.push(text(
                "Your skill collection: the git folder that holds your master skills.",
            ));
            lines.push(text("Walk to it, then press enter inside it:"));
            if setup.checking {
                lines.push(Line::from(Span::styled(
                    "   Looking at the folder…",
                    theme::muted(),
                )));
            }
        }
        Step::Root => {
            lines.push(done("Vault", &path(&setup.vault)));
            lines.push(current("Root"));
            lines.push(text(
                "The folder that contains all your projects; skillmirror scans it.",
            ));
        }
        Step::Mandatory => {
            lines.push(done("Vault", &path(&setup.vault)));
            lines.push(done("Root", &path(&setup.root)));
            lines.push(current("Mandatory"));
            lines.push(text("Skills that `push` installs into every project:"));
        }
        Step::Save | Step::Saved(_) => {
            lines.push(done("Vault", &path(&setup.vault)));
            lines.push(done("Root", &path(&setup.root)));
            lines.push(done("Mandatory", &mandatory_text(setup)));
        }
    }
    if let Some(error) = &setup.error {
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

fn mandatory_text(setup: &Setup) -> String {
    let names: Vec<&str> = setup
        .skills
        .iter()
        .filter(|s| s.checked && s.in_vault)
        .map(|s| s.name.as_str())
        .collect();
    if names.is_empty() {
        "(none)".to_string()
    } else {
        names.join(", ")
    }
}

fn checklist(frame: &mut Frame, hits: &mut HitMap, setup: &mut Setup, area: Rect) {
    setup.list.viewport = usize::from(area.height);
    setup.list.ensure_visible();
    if setup.skills.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "   (no skills found in the vault)",
                theme::muted(),
            )),
            area,
        );
        return;
    }
    let offset = setup.list.offset;
    for (slot, skill) in setup
        .skills
        .iter()
        .skip(offset)
        .take(usize::from(area.height))
        .enumerate()
    {
        let index = offset + slot;
        let row = Rect::new(area.x, area.y + to_u16(slot), area.width, 1);
        let at = index == setup.list.cursor;
        let mut spans = vec![
            Span::styled(if at { " > " } else { "   " }, theme::accent()),
            Span::styled(
                if skill.checked { "[✓]" } else { "[ ]" },
                if skill.checked {
                    theme::accent()
                } else {
                    Style::new()
                },
            ),
            Span::raw(format!(" {}", skill.name)),
        ];
        if !skill.in_vault {
            spans.push(Span::styled(
                "  not in the vault, will be removed",
                theme::warning(),
            ));
        }
        frame.render_widget(Paragraph::new(Line::from(spans)), row);
        hits.push(row, Target::CheckRow(index));
    }
}

fn final_page(
    frame: &mut Frame,
    hits: &mut HitMap,
    hover: Option<Target>,
    setup: &Setup,
    area: Rect,
) {
    let mut lines = Vec::new();
    let labels: &[&str] = match &setup.step {
        Step::Saved(Ok(())) => {
            lines.push(Line::from(Span::styled(
                "   ✓ Saved. The next scan uses the new settings.",
                Style::new().fg(theme::SUCCESS),
            )));
            &["enter  Done"]
        }
        Step::Saved(Err(message)) => {
            for line in message.lines() {
                lines.push(Line::from(Span::styled(
                    format!("   ✗ {line}"),
                    theme::error(),
                )));
            }
            &["enter  Done"]
        }
        _ => {
            let vault = setup
                .vault
                .as_ref()
                .map(|v| v.join("config.yaml"))
                .unwrap_or_default();
            lines.push(text("Save these settings?"));
            lines.push(Line::from(Span::styled(
                format!("   vault pointer      → {}", setup.pointer().display()),
                theme::muted(),
            )));
            lines.push(Line::from(Span::styled(
                format!("   root and mandatory → {}", vault.display()),
                theme::muted(),
            )));
            if let Some(note) = &setup.note {
                lines.push(Line::from(Span::styled(
                    format!("   {note}"),
                    theme::warning(),
                )));
            }
            &["y  Save", "n  Cancel"]
        }
    };
    let [message, buttons] = Layout::vertical([
        Constraint::Length(to_u16(lines.len() + 2)),
        Constraint::Length(3),
    ])
    .areas(area);
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), message);
    let row = Rect::new(
        buttons.x,
        buttons.y + 1.min(buttons.height.saturating_sub(1)),
        buttons.width,
        1,
    );
    let widths: Vec<Constraint> = labels
        .iter()
        .map(|l| Constraint::Length(to_u16(l.len() + 4)))
        .collect();
    let cells = Layout::horizontal(widths)
        .flex(Flex::Center)
        .spacing(2)
        .split(row);
    for (i, (label, cell)) in labels.iter().zip(cells.iter()).enumerate() {
        let mut style = Style::new().fg(if i == 0 {
            theme::SUCCESS
        } else {
            theme::ACCENT
        });
        if hover == Some(Target::Button(i)) {
            style = style.add_modifier(Modifier::REVERSED | Modifier::BOLD);
        }
        frame.render_widget(
            Paragraph::new(Span::styled(format!("[ {label} ]"), style))
                .alignment(Alignment::Center),
            *cell,
        );
        hits.push(*cell, Target::Button(i));
    }
}
