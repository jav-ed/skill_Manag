//! The text of a pane: notes wrapped to its width and cut to its height.

use ratatui::text::{Line, Span};

use crate::items::{Note, Tone};
use crate::theme;

/// A row of a pane after wrapping.
pub(in crate::ui) struct Row {
    text: String,
    tone: Tone,
}

/// The rows from `scroll` on that fit in `height` rows. When some do not, the last row says how many are
/// left out, so a long pane never looks complete when it is not. `scroll` is held to what the rows allow;
/// the scroll that was used comes back.
pub(in crate::ui) fn fitted(
    rows: &[Row],
    height: usize,
    scroll: usize,
) -> (Vec<Line<'static>>, usize) {
    let scroll = scroll.min(rows.len().saturating_sub(height));
    let visible = rows.get(scroll..).unwrap_or_default();
    if visible.len() <= height {
        return (visible.iter().map(line).collect(), scroll);
    }
    // One row stays free for the note about what is left out.
    let room = height.saturating_sub(1);
    let mut out: Vec<Line<'static>> = visible.iter().take(room).map(line).collect();
    out.push(Line::from(Span::styled(
        format!("… and {} more lines", visible.len() - room),
        theme::muted(),
    )));
    (out, scroll)
}

/// `note` cut into rows of at most `width` characters. A row breaks after a space or a `/`, so a path
/// falls apart at its folders and not in the middle of a name; a name longer than the row is cut. The
/// rows after the first are indented by two, except for headings and plain text, so a wrapped entry
/// still reads as one.
pub(in crate::ui) fn wrap(note: &Note, width: usize) -> Vec<Row> {
    let width = width.max(1);
    let indent = match note.tone {
        Tone::Heading | Tone::Plain => 0,
        Tone::Change | Tone::Quiet | Tone::Problem => 2.min(width - 1),
    };
    let mut lines = vec![String::new()];
    for chunk in note.text.split_inclusive([' ', '/']) {
        let mut rest = chunk;
        loop {
            let (used, continued) = lines
                .last()
                .map_or((0, false), |l| (l.chars().count(), lines.len() > 1));
            let bare = used <= if continued { indent } else { 0 };
            // Spaces do not start a continuation row.
            if continued && bare && rest.trim().is_empty() {
                break;
            }
            if rest.chars().count() <= width - used {
                if let Some(last) = lines.last_mut() {
                    last.push_str(rest);
                }
                break;
            }
            if !bare {
                lines.push(" ".repeat(indent));
                continue;
            }
            // Nothing else is on the row and the chunk is still too long: cut it.
            let cut = rest
                .char_indices()
                .nth(width - used)
                .map_or(rest.len(), |(at, _)| at);
            let (head, tail) = rest.split_at(cut);
            if let Some(last) = lines.last_mut() {
                last.push_str(head);
            }
            lines.push(" ".repeat(indent));
            rest = tail;
        }
    }
    lines
        .into_iter()
        .map(|text| Row {
            text: text.trim_end().to_string(),
            tone: note.tone,
        })
        .collect()
}

fn line(row: &Row) -> Line<'static> {
    let style = match row.tone {
        Tone::Heading => theme::bold(),
        Tone::Plain => ratatui::style::Style::new(),
        Tone::Change => theme::accent(),
        Tone::Quiet => theme::muted(),
        Tone::Problem => theme::error(),
    };
    Line::from(Span::styled(row.text.clone(), style))
}
