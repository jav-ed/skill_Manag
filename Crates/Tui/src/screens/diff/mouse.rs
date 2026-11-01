//! The mouse of the changes page: the wheel, the ruler at the right edge, the folds and the buttons.

use ratatui::layout::Position;

use super::DiffView;
use crate::hit::{HitMap, Target};
use crate::input::{Button, Mouse, MouseKind};

/// Rows one notch of the wheel scrolls by.
const WHEEL: isize = 3;

impl DiffView {
    pub(crate) fn on_mouse(&mut self, mouse: Mouse, hits: &HitMap) {
        let target = hits.at(Position::new(mouse.column, mouse.row));
        match (mouse.kind, target) {
            (MouseKind::ScrollUp, _) => self.scroll_by(-WHEEL),
            (MouseKind::ScrollDown, _) => self.scroll_by(WHEEL),
            (
                MouseKind::Down(Button::Left) | MouseKind::Drag(Button::Left),
                Some(Target::DiffRuler),
            ) => self.on_ruler(mouse.row, hits),
            (MouseKind::Down(Button::Left), Some(Target::DiffFold(item))) => self.open_fold(item),
            (MouseKind::Down(Button::Left), Some(Target::DiffButton(button))) => {
                self.press(button);
            }
            _ => {}
        }
    }

    /// The ruler is the whole page squeezed into its height: a click on it puts that part in the middle.
    fn on_ruler(&mut self, row: u16, hits: &HitMap) {
        let (Some(ruler), Some(layout)) = (hits.rect_of(Target::DiffRuler), &self.layout) else {
            return;
        };
        let from_top = usize::from(row.saturating_sub(ruler.y));
        let target = from_top * layout.total / usize::from(ruler.height).max(1);
        self.scroll_to(target.saturating_sub(self.viewport / 2));
    }

    fn open_fold(&mut self, item: usize) {
        let Some(layout) = &self.layout else {
            return;
        };
        if let Some(super::Item::Fold { id, .. }) = layout.items.get(item).map(|p| &p.item) {
            self.opened.insert(*id);
        }
    }
}
