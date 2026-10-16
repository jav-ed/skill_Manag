//! The two lists of the Override page: the projects to pick from, and the skills of the project.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::{edits_text, found_words, project_text};
use crate::hit::{HitMap, Target};
use crate::num::{plural, to_u16};
use crate::screens::Local;
use crate::theme;
use crate::ui::widgets::scroll;
use skillmirror_core::ops::{LocalSkill, LocalState};

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

pub(super) fn pick(frame: &mut Frame, hits: &mut HitMap, local: &mut Local, area: Rect) {
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
        let style = if at_cursor {
            theme::bold()
        } else {
            Style::new()
        };
        let text = local
            .projects
            .get(index)
            .map_or(String::new(), |p| p.display().to_string());
        vec![Span::styled(text, style)]
    });
}

pub(super) fn list(frame: &mut Frame, hits: &mut HitMap, local: &mut Local, area: Rect) {
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
                Span::styled(format!("   {}", found_words(local.found)), theme::muted()),
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
        local.skills.get(index).map_or_else(Vec::new, |skill| {
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
    let unread = if local.problems > 0 {
        format!(" · {} could not be read", plural(local.problems, "folder"))
    } else {
        String::new()
    };
    let text = format!(
        " {} can be replaced{unread}",
        plural(local.edited_count(), "skill")
    );
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
    let name_style = if at_cursor {
        theme::bold()
    } else {
        Style::new()
    };
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
