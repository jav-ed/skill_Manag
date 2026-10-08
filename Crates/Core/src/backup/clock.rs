//! Run ids that sort by time and read as a date.

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SEQUENCE: AtomicU32 = AtomicU32::new(0);

/// `20261007-123456-042-<pid>-<n>`: UTC date and time, milliseconds, process and a per-process counter.
pub(super) fn new_id() -> std::io::Result<String> {
    let since = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(std::io::Error::other)?;
    let (year, month, day, hour, minute, second) = civil(since.as_secs());
    let n = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    Ok(format!(
        "{year:04}{month:02}{day:02}-{hour:02}{minute:02}{second:02}-{:03}-{}-{n}",
        since.subsec_millis(),
        std::process::id()
    ))
}

/// The id as `2026-10-07 12:34:56 UTC`, or the id itself when it has another shape.
pub fn describe(id: &str) -> String {
    let digits = |from: usize, to: usize| {
        id.get(from..to)
            .filter(|s| s.bytes().all(|b| b.is_ascii_digit()))
    };
    match (
        digits(0, 4),
        digits(4, 6),
        digits(6, 8),
        digits(9, 11),
        digits(11, 13),
        digits(13, 15),
    ) {
        (Some(y), Some(mo), Some(d), Some(h), Some(mi), Some(s)) if id.get(8..9) == Some("-") => {
            format!("{y}-{mo}-{d} {h}:{mi}:{s} UTC")
        }
        _ => id.to_string(),
    }
}

/// The time now as `2026-10-08 12:34 UTC`, for a header that says when a file was made.
pub fn now_utc() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    format_utc(secs)
}

fn format_utc(secs: u64) -> String {
    let (year, month, day, hour, minute, _) = civil(secs);
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02} UTC")
}

/// Civil date and time of a Unix timestamp (days-to-date after Howard Hinnant).
fn civil(secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    let days = secs / 86_400;
    let rest = secs % 86_400;
    let z = days + 719_468;
    let era = z / 146_097;
    let day_of_era = z % 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let mp = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = year_of_era + era * 400 + u64::from(month <= 2);
    (year, month, day, rest / 3_600, rest % 3_600 / 60, rest % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_matches_known_dates() {
        assert_eq!(civil(0), (1970, 1, 1, 0, 0, 0));
        assert_eq!(civil(951_782_400), (2000, 2, 29, 0, 0, 0));
        assert_eq!(civil(1_791_376_496), (2026, 10, 7, 12, 34, 56));
    }

    #[test]
    fn the_header_time_has_the_form_of_a_date_and_minutes() {
        assert_eq!(format_utc(1_791_376_496), "2026-10-07 12:34 UTC");
        assert_eq!(format_utc(0), "1970-01-01 00:00 UTC");
        assert!(now_utc().ends_with(" UTC"));
    }

    #[test]
    fn ids_are_unique_sortable_and_readable() {
        let a = new_id().unwrap();
        let b = new_id().unwrap();
        assert_ne!(a, b);
        assert!(describe(&a).ends_with(" UTC"), "{}", describe(&a));
        assert_eq!(
            describe("20261007-123456-042-1-0"),
            "2026-10-07 12:34:56 UTC"
        );
        assert_eq!(describe("odd"), "odd");
    }
}
