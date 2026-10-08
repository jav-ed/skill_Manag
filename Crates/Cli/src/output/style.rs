//! Colours of the terminal output.

use anstyle::{AnsiColor, Style};

pub(super) const SUCCESS: Style = Style::new()
    .bold()
    .fg_color(Some(anstyle::Color::Ansi(AnsiColor::Green)));
pub(super) const WARNING: Style =
    Style::new().fg_color(Some(anstyle::Color::Ansi(AnsiColor::Yellow)));
pub(super) const ERROR: Style = Style::new()
    .bold()
    .fg_color(Some(anstyle::Color::Ansi(AnsiColor::Red)));
pub(super) const MUTED: Style = Style::new().dimmed();
pub(super) const HEADER: Style = Style::new().bold();
pub(super) const NAME: Style = Style::new().fg_color(Some(anstyle::Color::Ansi(AnsiColor::Cyan)));
pub(super) const ADDED: Style = Style::new().fg_color(Some(anstyle::Color::Ansi(AnsiColor::Green)));
pub(super) const REMOVED: Style = Style::new().fg_color(Some(anstyle::Color::Ansi(AnsiColor::Red)));
pub(super) const HUNK: Style = Style::new().fg_color(Some(anstyle::Color::Ansi(AnsiColor::Cyan)));
