//! Fuzzy filter over item names, with the matched character positions for highlighting.

use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

/// One match: the index of the item and the matched character positions of its name.
pub(crate) type Hit = (usize, Vec<u32>);

pub(crate) struct Fuzzy {
    matcher: Matcher,
}

impl Default for Fuzzy {
    fn default() -> Self {
        Self {
            matcher: Matcher::new(Config::DEFAULT),
        }
    }
}

impl Fuzzy {
    /// Matches in score order, best first; equal scores keep the original order. An empty query matches everything.
    pub(crate) fn run<'a>(
        &mut self,
        query: &str,
        names: impl Iterator<Item = &'a str>,
    ) -> Vec<Hit> {
        if query.is_empty() {
            return names.enumerate().map(|(i, _)| (i, Vec::new())).collect();
        }
        let pattern = Pattern::parse(query, CaseMatching::Smart, Normalization::Smart);
        let mut buf = Vec::new();
        let mut scored: Vec<(u32, usize, Vec<u32>)> = Vec::new();
        for (index, name) in names.enumerate() {
            let hay = Utf32Str::new(name, &mut buf);
            let mut positions = Vec::new();
            if let Some(score) = pattern.indices(hay, &mut self.matcher, &mut positions) {
                positions.sort_unstable();
                positions.dedup();
                scored.push((score, index, positions));
            }
        }
        scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        scored.into_iter().map(|(_, i, p)| (i, p)).collect()
    }
}
