//! The question before the vault's copies are replaced.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::text::{Line, Span};

use super::{edits_text, project_text};
use crate::hit::{HitMap, Target};
use crate::num::plural;
use crate::screens::Local;
use crate::screens::local::Pending;
use crate::theme;
use crate::ui::widgets::{boxed, dialog};

pub(super) fn confirm(
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
            "Then it updates {} that {} these skills.",
            plural(n, "other project"),
            if n == 1 { "has" } else { "have" }
        ))),
        None => {}
    }
    out
}
