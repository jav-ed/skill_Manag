//! Colours and text styles, taken from the Go tool so the two look alike.

use ratatui::style::{Color, Modifier, Style};

pub(crate) const SUCCESS: Color = Color::Rgb(0x00, 0xFF, 0x87);
pub(crate) const WARNING: Color = Color::Rgb(0xFF, 0xDB, 0x58);
pub(crate) const ERROR: Color = Color::Rgb(0xFF, 0x5F, 0x5F);
pub(crate) const MUTED: Color = Color::Rgb(0x62, 0x62, 0x62);
pub(crate) const ACCENT: Color = Color::Rgb(0x87, 0xCE, 0xEB);

// The changes page: a removed line, the words changed in it, an added line and its words, the hatching where a
// side has no line, the line that stands for folded lines, and the ruler at the right edge with its window.
pub(crate) const REMOVED_LINE: Color = Color::Rgb(0x3b, 0x1f, 0x24);
pub(crate) const REMOVED_WORD: Color = Color::Rgb(0x7d, 0x2a, 0x33);
pub(crate) const ADDED_LINE: Color = Color::Rgb(0x1c, 0x37, 0x27);
pub(crate) const ADDED_WORD: Color = Color::Rgb(0x2a, 0x6b, 0x40);
pub(crate) const HATCH: Color = Color::Rgb(0x3a, 0x3a, 0x3a);
pub(crate) const FOLD_LINE: Color = Color::Rgb(0x1f, 0x26, 0x33);
pub(crate) const RULER_TRACK: Color = Color::Rgb(0x1a, 0x1a, 0x1a);
pub(crate) const RULER_WINDOW: Color = Color::Rgb(0x3a, 0x3a, 0x4a);

pub(crate) const fn muted() -> Style {
    Style::new().fg(MUTED)
}

pub(crate) const fn accent() -> Style {
    Style::new().fg(ACCENT)
}

pub(crate) const fn warning() -> Style {
    Style::new().fg(WARNING)
}

pub(crate) const fn error() -> Style {
    Style::new().fg(ERROR).add_modifier(Modifier::BOLD)
}

pub(crate) const fn bold() -> Style {
    Style::new().add_modifier(Modifier::BOLD)
}
