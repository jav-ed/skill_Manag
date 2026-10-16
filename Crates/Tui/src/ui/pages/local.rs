//! The Override page: the skills of a project against the vault, the question before the vault's copies are
//! replaced, and what was done.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::states;
use crate::hit::{HitMap, Target};
use crate::num::{plural, to_u16};
use crate::screens::local::{Done, Pending};
use crate::screens::{Local, LocalPhase};
use crate::theme;
use crate::ui::widgets::{boxed, dialog, scroll};
use skillmirror_core::ops::{Edit, EditKind, Found, LocalSkill, LocalState};

/// Files named in a row or in the question; the rest are counted.
const NAMED: usize = 3;

pub(in crate::ui) fn draw(
    frame: &mut Frame,
    hits: &mut HitMap,
    hover: Option<Target>,
    tick: usize,
    local: &mut Local,
    area: Rect,
) {
    match &local.phase {
        LocalPhase::Loading => states::waiting(
            frame,
            tick,
            "Looking for the project and comparing its skills…",
            area,
        ),
        LocalPhase::Failed(message) => states::failed(frame, message, area),
        LocalPhase::Pick => pick(frame, hits, local, area),
        LocalPhase::List => list(frame, hits, local, area),
        LocalPhase::Planning => {
            states::waiting(frame, tick, "Checking the skills against the vault…", area);
        }
        LocalPhase::Confirm(_) => {
            list(frame, hits, local, area);
            if let LocalPhase::Confirm(pending) = &local.phase {
                confirm(frame, hits, hover, pending, local, area);
            }
        }
        LocalPhase::Running { done, total } => {
            states::progress(frame, "Replacing", *done, *total, area);
        }
        LocalPhase::Done(done) => {
            let all = lines(done, local);
            let height = usize::from(area.height);
            let scroll = local.scroll.min(all.len().saturating_sub(height));
            let visible: Vec<Line> = all.into_iter().skip(scroll).take(height).collect();
            frame.render_widget(Paragraph::new(visible), area);
            local.scroll = scroll;
        }
    }
}

fn found_words(found: Option<Found>) -> &'static str {
    match found {
        Some(Found::Here) => "the project in this folder",
        Some(Found::Above) => "the project around this folder",
        Some(Found::Below) => "the only project below this folder",
        None => "the project you picked",
    }
}

fn project_text(local: &Local) -> String {
    local
        .project
        .as_ref()
        .map_or(String::new(), |p| p.display().to_string())
}

/// The rows of a list with a cursor: `spans` says what one row looks like.
fn rows(
    frame: &mut Frame,
    hits: &mut HitMap,
    local: &mut Local,
    area: Rect,
    spans: impl Fn(&Local, usize, bool) -> Vec<Span<'static>>,
) {
    local.view.viewport = usize::from(area.height);
    local.view.ensure_visible();
    hits.push(area, Target::ListArea);
    let offset = local.view.offset;
    for (slot, (index, _)) in local
        .view
        .filtered
        .iter()
        .skip(offset)
        .take(usize::from(area.height))
        .enumerate()
    {
        let position = offset + slot;
        let rect = Rect::new(
            area.x,
            area.y + to_u16(slot),
            area.width.saturating_sub(1),
            1,
        );
        let mut line = vec![Span::styled(
            if local.view.cursor == position {
                "> "
            } else {
                "  "
            },
            theme::accent(),
        )];
        line.extend(spans(local, *index, local.view.cursor == position));
        frame.render_widget(Paragraph::new(Line::from(line)), rect);
        hits.push(rect, Target::Row(position));
    }
    if local.view.len() > usize::from(area.height) {
        scroll::bar(frame, hits, &local.view, area);
    }
}

fn pick(frame: &mut Frame, hits: &mut HitMap, local: &mut Local, area: Rect) {
    let [head, list_area] =
        Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).areas(area);
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(" Override", theme::bold())),
            Line::from(Span::styled(
                format!(
                    "  {} below this folder: pick the one to look at",
                    plural(local.projects.len(), "project")
                ),
                theme::muted(),
            )),
        ]),
        head,
    );
    rows(frame, hits, local, list_area, |local, index, at_cursor| {
        let style = if at_cursor { theme::bold() } else { Style::new() };
        let text = local
            .projects
            .get(index)
            .map_or(String::new(), |p| p.display().to_string());
        vec![Span::styled(text, style)]
    });
}

fn list(frame: &mut Frame, hits: &mut HitMap, local: &mut Local, area: Rect) {
    let [head, columns, body, spread, status] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled(" Override", theme::bold()),
                Span::styled(
                    format!("   {}", found_words(local.found)),
                    theme::muted(),
                ),
            ]),
            Line::from(Span::styled(
                format!("  {}", project_text(local)),
                theme::accent(),
            )),
        ]),
        head,
    );
    if local.skills.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "  This project has no skill folders yet.",
                theme::muted(),
            )),
            body,
        );
        return;
    }
    frame.render_widget(
        Paragraph::new(Span::styled(
            format!("      {:<24}{:<9}what differs", "skill", "state"),
            theme::muted(),
        )),
        columns,
    );
    rows(frame, hits, local, body, |local, index, at_cursor| {
        local
            .skills
            .get(index)
            .map_or_else(Vec::new, |skill| {
                row(skill, local.selected.contains(&index), at_cursor)
            })
    });
    let mark = if local.spread { "[x]" } else { "[ ]" };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(format!("  {mark} s "), theme::accent()),
            Span::styled(
                "then update the other projects that have the replaced skills",
                theme::muted(),
            ),
        ])),
        spread,
    );
    let mut text = format!(" {}", plural(local.edited_count(), "skill"));
    text.push_str(" can be replaced");
    if local.problems > 0 {
        text.push_str(&format!(
            " · {} could not be read",
            plural(local.problems, "folder")
        ));
    }
    frame.render_widget(Paragraph::new(Span::styled(text, theme::muted())), status);
}

fn row(skill: &LocalSkill, ticked: bool, at_cursor: bool) -> Vec<Span<'static>> {
    let (label, style, what) = match &skill.state {
        LocalState::Edited(edits) => (
            "edited",
            theme::warning(),
            format!("{}{}", edits_text(edits), group_text(skill)),
        ),
        LocalState::Behind { commits } => (
            "behind",
            theme::muted(),
            if *commits == 0 {
                "the vault's newest change is not committed yet; sync updates it".to_string()
            } else {
                format!(
                    "older than the vault's by {}; sync updates it",
                    plural(*commits, "change")
                )
            },
        ),
        LocalState::New => ("new", theme::muted(), "not in the vault".to_string()),
        LocalState::Same => ("same", theme::muted(), String::new()),
        LocalState::Unusable(why) => (
            "unusable",
            theme::error(),
            why.lines().next().unwrap_or_default().to_string(),
        ),
    };
    let editable = matches!(skill.state, LocalState::Edited(_));
    let name_style = if at_cursor { theme::bold() } else { Style::new() };
    vec![
        Span::styled(
            match (editable, ticked) {
                (true, true) => "[x] ",
                (true, false) => "[ ] ",
                (false, _) => "    ",
            },
            theme::accent(),
        ),
        Span::styled(format!("{:<24}", skill.name), name_style),
        Span::styled(format!("{label:<9}"), style),
        Span::styled(what, theme::muted()),
    ]
}

fn group_text(skill: &LocalSkill) -> String {
    if skill.group.is_empty() {
        String::new()
    } else {
        format!(" · vault: {}", skill.group.join("/"))
    }
}

/// `SKILL.md modified, notes.md added, 2 more`
fn edits_text(edits: &[Edit]) -> String {
    let mut parts: Vec<String> = edits
        .iter()
        .take(NAMED)
        .map(|e| {
            let what = match e.kind {
                EditKind::Added => "added",
                EditKind::Modified => "modified",
                EditKind::Removed => "removed",
            };
            format!("{} {what}", e.rel.display())
        })
        .collect();
    if edits.len() > NAMED {
        parts.push(format!("{} more", edits.len() - NAMED));
    }
    parts.join(", ")
}

fn confirm(
    frame: &mut Frame,
    hits: &mut HitMap,
    hover: Option<Target>,
    pending: &Pending,
    local: &Local,
    area: Rect,
) {
    let title = format!(
        "Replace {} in the vault?",
        plural(pending.plan.skills.len(), "skill")
    );
    let buttons = boxed::draw(
        frame,
        hits,
        area,
        &title,
        theme::WARNING,
        question(pending, local),
        1,
    );
    let [yes, no] = Layout::horizontal([Constraint::Length(18), Constraint::Length(18)])
        .flex(Flex::Center)
        .spacing(2)
        .areas(buttons);
    dialog::draw_button(frame, hits, hover, 0, yes, "y  Replace", theme::WARNING);
    dialog::draw_button(frame, hits, hover, 1, no, "n  Cancel", theme::ACCENT);
}

fn question(pending: &Pending, local: &Local) -> Vec<Line<'static>> {
    let mut out = vec![Line::raw(format!(
        "The vault's copy is replaced by the one in {}:",
        project_text(local)
    ))];
    for skill in &pending.plan.skills {
        out.push(Line::from(Span::styled(
            format!("  {}  {}", skill.name, edits_text(&skill.edits)),
            theme::muted(),
        )));
    }
    out.push(Line::raw(
        "The old copy is kept, so History can undo this. The new one is staged in git, not committed.",
    ));
    match pending.spread {
        Some(0) => out.push(Line::raw("No other project has these skills.")),
        Some(n) => out.push(Line::raw(format!(
            "Then {} that have these skills {} updated from the vault.",
            plural(n, "other project"),
            if n == 1 { "is" } else { "are" }
        ))),
        None => {}
    }
    out
}

fn lines(done: &Done, local: &Local) -> Vec<Line<'static>> {
    let mut out = vec![
        Line::from(Span::styled(
            format!(" Override results: {}", project_text(local)),
            theme::bold(),
        )),
        Line::raw(""),
    ];
    for outcome in &done.outcomes {
        match &outcome.result {
            Ok(files) => out.push(Line::from(vec![
                Span::styled("  ✓ ", Style::new().fg(theme::SUCCESS)),
                Span::raw(format!(
                    "{} replaced in the vault ({})",
                    outcome.name,
                    plural(*files, "file")
                )),
            ])),
            Err(message) => out.push(Line::from(vec![
                Span::styled("  ✗ ", theme::error()),
                Span::raw(format!(
                    "{} not replaced: {}",
                    outcome.name,
                    message.replace('\n', "  ")
                )),
            ])),
        }
        if outcome.left_out > 0 {
            out.push(Line::from(Span::styled(
                format!(
                    "    {} not taken by git (the vault's .gitignore?)",
                    plural(outcome.left_out, "file")
                ),
                theme::warning(),
            )));
        }
    }
    out.push(Line::raw(""));
    if done.outcomes.iter().any(|o| o.result.is_ok()) {
        out.push(Line::from(Span::styled(
            "  Staged in git, not committed. Commit it in the vault when it is ready.",
            theme::muted(),
        )));
    }
    if let Some(id) = &done.backup {
        out.push(Line::from(Span::styled(
            format!("  The old copies are saved as run {id} (History can undo it)."),
            theme::muted(),
        )));
    }
    if let Some(spread) = &done.spread {
        out.push(Line::raw(""));
        match spread {
            Ok(spread) => {
                out.push(Line::raw(format!(
                    "  Other projects: {} written in {}.",
                    plural(spread.written, "skill folder"),
                    plural(spread.projects, "project")
                )));
                for failure in &spread.failed {
                    out.push(Line::from(vec![
                        Span::styled("  ✗ ", theme::error()),
                        Span::raw(failure.replace('\n', "  ")),
                    ]));
                }
            }
            Err(message) => out.push(Line::from(Span::styled(
                format!("  The other projects were not updated: {}", message.replace('\n', "  ")),
                theme::error(),
            ))),
        }
    }
    out
}
