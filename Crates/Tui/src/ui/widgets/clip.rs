//! Cutting a path to a width from the front: the folders next to the name say the most, so they stay.

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// `path` as it is when it fits `width` columns, else `…/` and the last folders that fit; when one folder
/// name alone is too long, `…` and the end of it.
pub(in crate::ui) fn front(path: &str, width: usize) -> String {
    if path.width() <= width {
        return path.to_string();
    }
    let room = width.saturating_sub(2);
    // The longest tail that starts after a `/` and fits next to `…/`.
    let tail = path
        .match_indices('/')
        .filter_map(|(at, _)| path.get(at + 1..))
        .find(|tail| tail.width() <= room);
    if let Some(tail) = tail {
        return format!("…/{tail}");
    }
    let mut kept: Vec<char> = Vec::new();
    let mut used = 0;
    for c in path.chars().rev() {
        let w = c.width().unwrap_or(0);
        if used + w > width.saturating_sub(1) {
            break;
        }
        used += w;
        kept.push(c);
    }
    kept.reverse();
    format!("…{}", kept.into_iter().collect::<String>())
}

#[cfg(test)]
mod tests {
    use super::front;

    #[test]
    fn a_path_that_fits_stays_as_it_is() {
        assert_eq!(front("web/seo", 10), "web/seo");
        assert_eq!(front("web/seo", 7), "web/seo");
    }

    #[test]
    fn a_long_path_keeps_its_last_folders_whole() {
        assert_eq!(front("Web_Skills/seo/schema/checks", 18), "…/schema/checks");
        assert_eq!(front("Web_Skills/seo/schema/checks", 13), "…/checks");
        assert_eq!(
            front("Web_Skills/seo/schema/checks", 19),
            "…/seo/schema/checks"
        );
    }

    #[test]
    fn one_folder_name_that_is_too_long_is_cut_at_the_front() {
        assert_eq!(front("a/averyveryverylongfoldername", 8), "…dername");
        assert_eq!(front("averyveryverylongfoldername", 6), "…rname");
    }

    #[test]
    fn the_result_is_never_wider_than_the_width() {
        for width in 3..30 {
            let cut = front("Default_Skills/web/seo/schema/audits/weekly/checks", width);
            assert!(cut.chars().count() <= width, "{width}: {cut}");
        }
    }
}
