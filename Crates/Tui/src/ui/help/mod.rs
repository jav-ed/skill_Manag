//! The help overlay (`?`): the keys of the screen in topics, each with what it is good for. The short line
//! at the bottom only names keys; this is where they are explained.

mod screens;
mod work;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph};

use crate::app::Screen;
use crate::hit::{HitMap, Target};
use crate::num::to_u16;
use crate::theme;

/// The widest the box gets, however wide the terminal is.
const WIDTH_MAX: u16 = 84;
/// The column the explanations start at, counted from the left edge of the text.
const LABEL_WIDTH: usize = 18;

/// A heading and the keys under it, each with what it does.
pub(in crate::ui) struct Topic {
    pub(in crate::ui) title: &'static str,
    pub(in crate::ui) keys: Vec<(&'static str, &'static str)>,
}

pub(in crate::ui) fn topic(title: &'static str, keys: Vec<(&'static str, &'static str)>) -> Topic {
    Topic { title, keys }
}

/// The topics of the screen as it is now.
fn topics(screen: &Screen) -> Vec<Topic> {
    match screen {
        Screen::Menu(_) => screens::menu_topics(),
        Screen::Setup(setup) => screens::setup_topics(setup),
        Screen::History(history) => screens::history_topics(history),
        Screen::Place(place) => screens::place_topics(place),
        Screen::Work(work) => work::work_topics(work),
    }
}

/// The topic every screen ends with.
pub(in crate::ui) fn general(first: &[(&'static str, &'static str)]) -> Topic {
    let mut keys = first.to_vec();
    keys.push(("?", "show or hide this help."));
    keys.push((
        "ctrl+c",
        "quit. While something is being written it asks twice.",
    ));
    topic("General", keys)
}

/// `text` cut at spaces into lines of at most `width` characters; a word longer than that is kept whole.
fn words(text: &str, width: usize) -> Vec<String> {
    let mut lines = vec![String::new()];
    for word in text.split_whitespace() {
        let last = lines.last_mut().map_or(0, |l| l.chars().count());
        if last > 0 && last + 1 + word.chars().count() > width {
            lines.push(String::new());
        }
        if let Some(line) = lines.last_mut() {
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
    }
    lines
}

/// The lines of the box for `text_width` columns.
fn lines(topics: &[Topic], text_width: usize) -> Vec<Line<'static>> {
    let mut out = Vec::new();
    for (n, topic) in topics.iter().enumerate() {
        if n > 0 {
            out.push(Line::raw(""));
        }
        out.push(Line::from(Span::styled(topic.title, theme::bold())));
        for (label, text) in &topic.keys {
            let wrapped = words(text, text_width.saturating_sub(LABEL_WIDTH).max(10));
            for (i, part) in wrapped.into_iter().enumerate() {
                let first = if i == 0 {
                    Span::styled(
                        format!("  {label:<width$}", width = LABEL_WIDTH - 2),
                        theme::accent(),
                    )
                } else {
                    Span::raw(" ".repeat(LABEL_WIDTH))
                };
                out.push(Line::from(vec![first, Span::raw(part)]));
            }
        }
    }
    out
}

/// Draws the help over the screen. `scroll` is how far down it is read; it comes back held to what fits.
pub(in crate::ui) fn overlay(
    frame: &mut Frame,
    hits: &mut HitMap,
    screen: &Screen,
    scroll: &mut usize,
) {
    let full_area = frame.area();
    hits.push(full_area, Target::Dismiss);
    let width = WIDTH_MAX
        .min(full_area.width.saturating_sub(2))
        .max(full_area.width.min(4));
    // Two columns for the frame and two for the padding.
    let text_width = usize::from(width.saturating_sub(4));
    let lines = lines(&topics(screen), text_width);
    let wanted = to_u16(lines.len() + 2);
    let height = wanted
        .min(full_area.height.saturating_sub(2))
        .max(full_area.height.min(3));
    let rect = Rect::new(
        full_area.x + (full_area.width - width) / 2,
        full_area.y + (full_area.height - height) / 2,
        width,
        height,
    );
    hits.push(rect, Target::Overlay);
    frame.render_widget(Clear, rect);
    let inside = usize::from(height.saturating_sub(2));
    let most = lines.len().saturating_sub(inside);
    *scroll = (*scroll).min(most);
    let mut block = Block::default()
        .borders(Borders::ALL)
        .title(" keys ")
        .padding(Padding::horizontal(1));
    if most > 0 {
        block = block.title_bottom(" ↑/↓ scroll  any other key closes ");
    }
    let shown: Vec<Line<'static>> = lines.into_iter().skip(*scroll).take(inside).collect();
    frame.render_widget(Paragraph::new(shown).block(block), rect);
}
