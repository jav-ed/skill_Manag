//! The words of a line that changed: what the line has that its partner lacks, and the other way round.

use std::ops::Range;

use similar::{Algorithm, DiffOp, capture_diff_slices};

/// Both lines together longer than this are shown as changed as a whole: marking the words of a paragraph
/// the size of a page costs more than it tells.
const MAX_BYTES: usize = 8 * 1024;
/// More words and gaps than this (both lines together) are shown as changed as a whole.
const MAX_TOKENS: usize = 2048;
/// When more than this share of a line (in percent of its visible characters) is marked on both sides, the
/// line was replaced, not edited, and the tint of the whole line says so better than a mark everywhere.
const REPLACED_PERCENT: usize = 70;

/// The byte ranges that differ in `old` and in `new`, each list sorted and apart. Both are empty when the
/// lines are too long, or too different, for marks to help.
pub(super) fn marks(old: &str, new: &str) -> (Vec<Range<usize>>, Vec<Range<usize>>) {
    let none = (Vec::new(), Vec::new());
    if old.len() + new.len() > MAX_BYTES {
        return none;
    }
    let (old_tokens, new_tokens) = (tokens(old), tokens(new));
    if old_tokens.len() + new_tokens.len() > MAX_TOKENS {
        return none;
    }
    let ops = capture_diff_slices(Algorithm::Myers, &texts(&old_tokens), &texts(&new_tokens));
    let (mut left, mut right) = (Vec::new(), Vec::new());
    for op in ops {
        match op {
            DiffOp::Equal { .. } => {}
            DiffOp::Delete {
                old_index, old_len, ..
            } => left.extend(span(&old_tokens, old_index, old_len)),
            DiffOp::Insert {
                new_index, new_len, ..
            } => right.extend(span(&new_tokens, new_index, new_len)),
            DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                left.extend(span(&old_tokens, old_index, old_len));
                right.extend(span(&new_tokens, new_index, new_len));
            }
        }
    }
    let (left, right) = (join(left, old), join(right, new));
    if replaced(&left, old) && replaced(&right, new) {
        return none;
    }
    (left, right)
}

fn texts<'a>(tokens: &[Token<'a>]) -> Vec<&'a str> {
    tokens.iter().map(|token| token.text).collect()
}

/// A run of letters and digits, a run of blanks, or one other character. Marking `(` and `)` apart from the
/// word beside them keeps a changed bracket from marking the whole word.
struct Token<'a> {
    text: &'a str,
    range: Range<usize>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Class {
    Word,
    Blank,
    Other,
}

fn class(c: char) -> Class {
    if c.is_alphanumeric() || c == '_' {
        Class::Word
    } else if c.is_whitespace() {
        Class::Blank
    } else {
        Class::Other
    }
}

/// The text of a range of it. The ranges here come from `char_indices` of this very text, so they are
/// always on character boundaries and the empty text is never what is returned.
fn cut(text: &str, range: Range<usize>) -> &str {
    text.get(range).unwrap_or_default()
}

fn tokens(text: &str) -> Vec<Token<'_>> {
    let mut out = Vec::new();
    let mut chars = text.char_indices().peekable();
    while let Some((start, first)) = chars.next() {
        let kind = class(first);
        let mut end = start + first.len_utf8();
        if kind != Class::Other {
            while let Some(&(next, c)) = chars.peek() {
                if class(c) != kind {
                    break;
                }
                end = next + c.len_utf8();
                chars.next();
            }
        }
        out.push(Token {
            text: cut(text, start..end),
            range: start..end,
        });
    }
    out
}

/// The text the `len` tokens from `index` cover, if there are any.
fn span(tokens: &[Token], index: usize, len: usize) -> Option<Range<usize>> {
    let mut run = tokens.iter().skip(index).take(len);
    let first = run.next()?;
    let last = run.next_back().unwrap_or(first);
    Some(first.range.start..last.range.end)
}

/// Changed words with nothing but blanks between them read as one phrase: one mark.
fn join(ranges: Vec<Range<usize>>, text: &str) -> Vec<Range<usize>> {
    let mut out: Vec<Range<usize>> = Vec::new();
    for range in ranges {
        match out.last_mut() {
            Some(last)
                if cut(text, last.end..range.start)
                    .chars()
                    .all(char::is_whitespace) =>
            {
                last.end = range.end;
            }
            _ => out.push(range),
        }
    }
    out
}

/// Whether the marks cover most of what can be seen of the line.
fn replaced(marks: &[Range<usize>], text: &str) -> bool {
    let visible = |part: &str| part.chars().filter(|c| !c.is_whitespace()).count();
    let total = visible(text);
    let marked: usize = marks
        .iter()
        .map(|range| visible(cut(text, range.clone())))
        .sum();
    total > 0 && marked * 100 > total * REPLACED_PERCENT
}
