//! Small number helpers, so the rest of the crate never needs a lossy `as` cast.

/// Terminal coordinates are `u16`; a larger count is clamped, which no screen ever reaches.
pub(crate) fn to_u16(n: usize) -> u16 {
    u16::try_from(n).unwrap_or(u16::MAX)
}

/// `done / total` for a progress gauge, in `0.0..=1.0`. An empty total counts as finished.
pub(crate) fn ratio(done: usize, total: usize) -> f64 {
    if total == 0 {
        return 1.0;
    }
    f64::from(to_u32(done.min(total))) / f64::from(to_u32(total))
}

fn to_u32(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// Moves `value` by `delta` and keeps it in `0..len`. An empty list gives 0.
pub(crate) fn step(value: usize, delta: isize, len: usize) -> usize {
    let moved = if delta.is_negative() {
        value.saturating_sub(delta.unsigned_abs())
    } else {
        value.saturating_add(delta.unsigned_abs())
    };
    moved.min(len.saturating_sub(1))
}

/// Singular for one, plural otherwise: `1 project`, `3 projects`.
pub(crate) fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        format!("{n} {word}")
    } else {
        format!("{n} {word}s")
    }
}
