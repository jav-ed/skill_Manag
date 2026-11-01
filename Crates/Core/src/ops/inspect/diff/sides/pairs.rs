//! Which old line goes next to which new line, where a stretch of lines was replaced.
//!
//! The line diff says "these old lines became those new lines" and nothing about which is which. Pairing them
//! in order is wrong as soon as a line was added in front: every line would sit beside the wrong partner and
//! every word would be marked. So lines that read alike are paired first, in order, and the lines between
//! two such pairs are paired in order.

use std::ops::Range;

/// More lines than this on a side are paired in order: the search costs lines times lines.
const MAX_LINES: usize = 256;
/// More text than this (both sides together) is paired in order.
const MAX_BYTES: usize = 64 * 1024;
/// How alike two lines must read to be partners, out of 1000.
const MIN_SIMILARITY: u32 = 500;

/// Pairs the old lines with the new lines by index. Every old and every new line appears exactly once, in
/// order on both sides; a line with no partner has `None` beside it.
pub(super) fn pair(old: &[&str], new: &[&str]) -> Vec<(Option<usize>, Option<usize>)> {
    let bytes: usize = old.iter().chain(new).map(|line| line.len()).sum();
    let anchors = if old.len() > MAX_LINES || new.len() > MAX_LINES || bytes > MAX_BYTES {
        Vec::new()
    } else {
        anchors(old, new)
    };
    let mut out = Vec::new();
    let (mut old_from, mut new_from) = (0, 0);
    // The end of both sides closes the last gap; it is no pair of its own.
    for (i, j) in anchors.into_iter().chain([(old.len(), new.len())]) {
        in_order(&mut out, old_from..i, new_from..j);
        if i < old.len() {
            out.push((Some(i), Some(j)));
        }
        (old_from, new_from) = (i + 1, j + 1);
    }
    out
}

/// Lines that read alike, as `(old, new)` indexes that grow on both sides. Each old line takes the most
/// alike new line after the previous partner (the first one, when two are as alike).
fn anchors(old: &[&str], new: &[&str]) -> Vec<(usize, usize)> {
    let new_grams: Vec<Vec<(char, char)>> = new.iter().map(|line| bigrams(line)).collect();
    let mut out = Vec::new();
    let mut from = 0;
    for (i, line) in old.iter().enumerate() {
        let grams = bigrams(line);
        let best = new_grams
            .iter()
            .enumerate()
            .skip(from)
            .map(|(j, other)| (similarity(&grams, other), j))
            .filter(|(similarity, _)| *similarity >= MIN_SIMILARITY)
            .max_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)));
        if let Some((_, j)) = best {
            out.push((i, j));
            from = j + 1;
        }
    }
    out
}

/// The lines of both ranges paired one to one in order; what one side has more stays without a partner.
/// Similarity finds partners, it does not prove that the lines left over are unrelated.
fn in_order(out: &mut Vec<(Option<usize>, Option<usize>)>, old: Range<usize>, new: Range<usize>) {
    for k in 0..old.len().max(new.len()) {
        out.push((
            (k < old.len()).then_some(old.start + k),
            (k < new.len()).then_some(new.start + k),
        ));
    }
}

/// The pairs of neighbouring characters of the trimmed line, sorted, a repeated pair repeated.
fn bigrams(line: &str) -> Vec<(char, char)> {
    let mut before = '\0';
    let mut grams: Vec<(char, char)> = line
        .trim()
        .chars()
        .map(|c| {
            let gram = (before, c);
            before = c;
            gram
        })
        .collect();
    grams.sort_unstable();
    grams
}

/// The Dice coefficient of two sorted bigram lists, out of 1000: twice the pairs they share over all pairs.
/// Two empty lines are the same line.
fn similarity(a: &[(char, char)], b: &[(char, char)]) -> u32 {
    if a.is_empty() && b.is_empty() {
        return 1000;
    }
    let (mut left, mut right) = (a.iter().peekable(), b.iter().peekable());
    let mut shared = 0usize;
    while let (Some(x), Some(y)) = (left.peek(), right.peek()) {
        match x.cmp(y) {
            std::cmp::Ordering::Less => {
                left.next();
            }
            std::cmp::Ordering::Greater => {
                right.next();
            }
            std::cmp::Ordering::Equal => {
                shared += 1;
                left.next();
                right.next();
            }
        }
    }
    let total = a.len() + b.len();
    // At most 1000: shared pairs cannot be more than half of all pairs.
    u32::try_from(2000 * shared / total).unwrap_or(1000)
}
