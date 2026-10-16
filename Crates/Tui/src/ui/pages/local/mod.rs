//! The Override page: the skills of a project against the vault, the question before the vault's copies are
//! replaced, and what was done.

mod confirm;
mod done;
mod list;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use super::states;
use crate::hit::{HitMap, Target};
use crate::screens::{Local, LocalPhase};
use skillmirror_core::ops::{Edit, EditKind, Found};

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
        LocalPhase::Pick => list::pick(frame, hits, local, area),
        LocalPhase::List => list::list(frame, hits, local, area),
        LocalPhase::Planning => {
            states::waiting(frame, tick, "Checking the skills against the vault…", area);
        }
        LocalPhase::Confirm(_) => {
            list::list(frame, hits, local, area);
            if let LocalPhase::Confirm(pending) = &local.phase {
                confirm::confirm(frame, hits, hover, pending, local, area);
            }
        }
        LocalPhase::Running { done, total } => {
            states::progress(frame, "Replacing", *done, *total, area);
        }
        LocalPhase::Done(done) => {
            let all = done::lines(done, local);
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

pub(super) fn project_text(local: &Local) -> String {
    local
        .project
        .as_ref()
        .map_or(String::new(), |p| p.display().to_string())
}

/// `SKILL.md modified, notes.md added, 2 more`
pub(super) fn edits_text(edits: &[Edit]) -> String {
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
