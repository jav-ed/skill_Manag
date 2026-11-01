//! Breaking text into rows of a width. The layout that counts the rows and the page that draws them use
//! this one function, so that a row the layout reserved is a row the page fills.

use std::ops::Range;

use unicode_width::UnicodeWidthChar;

/// The byte ranges of `text` that make its rows of at most `width` cells: a row breaks after its last blank
/// when it has one, so that words stay whole, and in the middle of a word when it has none. Always at least
/// one range: an empty text is one empty row.
pub(crate) fn wrap(text: &str, width: usize) -> Vec<Range<usize>> {
    let width = width.max(1);
    let mut rows = Vec::new();
    let mut start = 0;
    let mut cells = 0;
    // Where the row can break: just after its last blank, and the cells the row has up to there.
    let mut blank: Option<(usize, usize)> = None;
    for (at, c) in text.char_indices() {
        let w = c.width().unwrap_or(0);
        // A loop: the part carried over from the last blank can still be too wide for a wide character.
        while cells > 0 && cells + w > width {
            let (cut, kept) = match blank {
                Some((after, kept)) if after > start => (after, kept),
                _ => (at, cells),
            };
            rows.push(start..cut);
            start = cut;
            cells -= kept;
            blank = None;
        }
        cells += w;
        if c == ' ' {
            blank = Some((at + 1, cells));
        }
    }
    rows.push(start..text.len());
    rows
}

/// `text` as rows of at most `width` cells: every line of it, wrapped. An empty text has no rows.
pub(crate) fn rows(text: &str, width: usize) -> Vec<String> {
    text.lines()
        .flat_map(|line| {
            wrap(line, width)
                .into_iter()
                .map(move |range| cut(line, range).to_string())
        })
        .collect()
}

/// The text of a range of it. The ranges come from `wrap` or from the marks of the same text, so they are on
/// character boundaries; the empty text is never what is returned.
pub(crate) fn cut(text: &str, range: Range<usize>) -> &str {
    text.get(range).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use unicode_width::UnicodeWidthStr;

    use super::*;

    // Rule: the rows a text is broken into hold all of it, none is wider than the width, and words stay whole
    // when they can. The layout reserves rows with this and the page fills them, so a miscount shifts every
    // row below it. 8/10.
    #[test]
    fn rows_hold_all_the_text_within_the_width_and_keep_words_whole() {
        let text = "alpha beta gamma delta 日本語のテキスト supercalifragilistic";
        for width in [1, 5, 10, 17, 80] {
            let ranges = wrap(text, width);

            let joined: String = ranges.iter().map(|r| cut(text, r.clone())).collect();
            assert_eq!(joined, text, "width {width}");
            for range in &ranges {
                let row = cut(text, range.clone());
                // A wide character wider than the width stands alone in its row.
                assert!(
                    row.width() <= width || row.chars().count() == 1,
                    "{row:?} at {width}"
                );
            }
        }
        let rows = rows("alpha beta gamma", 11);
        assert_eq!(rows, ["alpha beta ", "gamma"], "a row breaks after a blank");
        assert!(
            matches!(wrap("", 10).as_slice(), [only] if only.is_empty()),
            "an empty text is one empty row"
        );
    }
}
