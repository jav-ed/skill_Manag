//! The mouse of the selection screens.

use ratatui::layout::Position;

use super::work::{Action, Phase, Work};
use crate::hit::{HitMap, Target};
use crate::input::{Button, Mouse, MouseKind};
use crate::items_projects::PaneView;

impl Work {
    pub(crate) fn on_mouse(&mut self, mouse: Mouse, hits: &HitMap) -> Action {
        let target = hits.at(Position::new(mouse.column, mouse.row));
        if self.issues_open {
            if matches!(mouse.kind, MouseKind::Down(Button::Left)) {
                self.issues_open = false;
            }
            return Action::None;
        }
        match self.phase {
            Phase::Select => self.select_mouse(mouse, target, hits),
            Phase::Confirm(_) => self.confirm_mouse(mouse, target),
            Phase::Diff(_) => {
                self.diff_mouse(mouse);
                Action::None
            }
            Phase::Done(_) => {
                match mouse.kind {
                    MouseKind::ScrollUp => self.scroll = self.scroll.saturating_sub(3),
                    MouseKind::ScrollDown => self.scroll = self.scroll.saturating_add(3),
                    _ => {}
                }
                Action::None
            }
            _ => Action::None,
        }
    }

    fn diff_mouse(&mut self, mouse: Mouse) {
        if let Phase::Diff(page) = &mut self.phase {
            match mouse.kind {
                MouseKind::ScrollUp => page.scroll = page.scroll.saturating_sub(3),
                MouseKind::ScrollDown => page.scroll = page.scroll.saturating_add(3),
                _ => {}
            }
        }
    }

    fn select_mouse(&mut self, mouse: Mouse, target: Option<Target>, hits: &HitMap) -> Action {
        let over_list = matches!(
            target,
            Some(Target::Row(_) | Target::ListArea | Target::ScrollTrack)
        );
        match (mouse.kind, target) {
            (MouseKind::Moved, Some(Target::Row(row))) => self.view.cursor = row,
            (MouseKind::Down(Button::Left), Some(Target::Row(row))) => {
                self.view.cursor = row;
                self.toggle_current();
            }
            (MouseKind::Down(Button::Left), Some(Target::ScrollTrack)) => {
                if let Some(track) = hits.rect_of(Target::ScrollTrack) {
                    let from_top = usize::from(mouse.row.saturating_sub(track.y));
                    self.view.jump(from_top, usize::from(track.height));
                }
            }
            (MouseKind::Down(Button::Left), Some(Target::PaneTab(tab))) => {
                self.pane.view = [PaneView::Affected, PaneView::Others, PaneView::All]
                    .get(tab)
                    .copied()
                    .unwrap_or(self.pane.view);
            }
            (MouseKind::Down(Button::Left), Some(Target::PanePaths)) => {
                self.pane.paths = self.pane.paths.toggled();
            }
            (MouseKind::Down(Button::Left), Some(Target::PaneFilter)) => {
                self.pane.filtering = true;
            }
            (MouseKind::ScrollUp, Some(Target::Pane)) => {
                self.pane.scroll = self.pane.scroll.saturating_sub(3);
            }
            (MouseKind::ScrollDown, Some(Target::Pane)) => {
                self.pane.scroll = self.pane.scroll.saturating_add(3);
            }
            (MouseKind::ScrollUp, _) if over_list => self.view.scroll_by(-3),
            (MouseKind::ScrollDown, _) if over_list => self.view.scroll_by(3),
            _ => {}
        }
        Action::None
    }

    fn confirm_mouse(&mut self, mouse: Mouse, target: Option<Target>) -> Action {
        if !matches!(mouse.kind, MouseKind::Down(Button::Left)) {
            return Action::None;
        }
        match target {
            Some(Target::Button(0)) => self.confirmed(),
            Some(Target::Button(_) | Target::Dismiss) => {
                self.phase = Phase::Select;
                Action::None
            }
            _ => Action::None,
        }
    }
}
