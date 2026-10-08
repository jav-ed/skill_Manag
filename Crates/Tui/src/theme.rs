//! Colours and text styles, taken from the Go tool so the two look alike.

use ratatui::style::{Color, Modifier, Style};

pub(crate) const SUCCESS: Color = Color::Rgb(0x00, 0xFF, 0x87);
pub(crate) const WARNING: Color = Color::Rgb(0xFF, 0xDB, 0x58);
pub(crate) const ERROR: Color = Color::Rgb(0xFF, 0x5F, 0x5F);
pub(crate) const MUTED: Color = Color::Rgb(0x62, 0x62, 0x62);
pub(crate) const ACCENT: Color = Color::Rgb(0x87, 0xCE, 0xEB);

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
