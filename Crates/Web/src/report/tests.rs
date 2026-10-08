//! The report is made of values; these tests build the data by hand and read the HTML back.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use skillmirror_core::ops::{Cell, DiffKind, FileDiff, ReportData, SkillInfo};
use skillmirror_core::scan::ScanIssue;

use super::{MARKER, render_report};

fn skill(name: &str, group: &[&str], description: Option<&str>, mandatory: bool) -> SkillInfo {
    SkillInfo {
        name: name.to_string(),
        group: group.iter().map(ToString::to_string).collect(),
        description: description.map(ToString::to_string),
        header_problem: None,
        mandatory,
        files: vec!["SKILL.md".to_string()],
    }
}

fn changed(text: &str) -> FileDiff {
    FileDiff {
        path: PathBuf::from("SKILL.md"),
        kind: DiffKind::Modified,
        added_lines: 1,
        removed_lines: 1,
        modes: None,
        skipped: None,
        text: text.to_string(),
    }
}

/// Two projects, a top-level skill, a skill in a group, and a folder the vault lacks.
fn data() -> ReportData {
    let mut cells = BTreeMap::new();
    cells.insert(("coding".to_string(), 0), Cell::Current);
    cells.insert(
        ("astro".to_string(), 1),
        Cell::Outdated(vec![changed(
            "--- a/SKILL.md\n+++ b/SKILL.md\n@@ -1 +1 @@\n-old\n+new\n",
        )]),
    );
    cells.insert(("tmux".to_string(), 1), Cell::MissingMandatory);
    cells.insert(("mine".to_string(), 1), Cell::NotInVault);
    ReportData {
        vault: PathBuf::from("/home/me/vault"),
        root: PathBuf::from("/home/me/projects"),
        skills: vec![
            skill("coding", &[], Some("Write code"), false),
            skill("tmux", &[], Some("Terminals"), true),
            skill("astro", &["web"], Some("Astro sites"), false),
        ],
        extra: vec!["mine".to_string()],
        projects: vec![
            PathBuf::from("/home/me/projects/one"),
            PathBuf::from("/home/me/projects/two"),
        ],
        cells,
        project_problems: Vec::new(),
        missing_bridges: Vec::new(),
        issues: Vec::new(),
    }
}

fn page(data: &ReportData) -> String {
    render_report(data, "2026-10-08 12:00 UTC")
}

#[test]
fn the_page_is_marked_self_contained_and_forbids_the_network() {
    let html = page(&data());

    assert!(html.starts_with("<!DOCTYPE html>"), "{}", &html[..40]);
    assert!(
        html[..80].contains(MARKER),
        "the marker follows the doctype"
    );
    assert!(html.contains("Content-Security-Policy"));
    assert!(html.contains("default-src 'none'"));
    for outside in ["http://", "https://", "<link", "src=", "@import", "url("] {
        assert!(!html.contains(outside), "{outside} would load something");
    }
    assert!(html.contains("2026-10-08 12:00 UTC"));
    assert!(html.contains("/home/me/vault"));
}

#[test]
fn every_value_is_escaped() {
    let mut data = data();
    let evil = "<script>alert(1)</script>";
    data.skills[0].name = format!("a{evil}");
    data.skills[0].description = Some(format!("\"{evil}\" & <img src=x onerror=alert(2)>"));
    data.skills[0].files = vec![format!("<b>{evil}</b>")];
    data.extra = vec![format!("x{evil}")];
    data.vault = PathBuf::from(format!("/v/{evil}"));
    data.issues = vec![ScanIssue {
        path: PathBuf::from(format!("/p/{evil}")),
        message: format!("<i>{evil}</i>"),
    }];
    data.projects[1] = PathBuf::from(format!("/p/{evil}/x"));
    data.cells.insert(
        ("tmux".to_string(), 0),
        Cell::Problem {
            message: format!("<u>{evil}</u>"),
            hint: Some(evil.to_string()),
        },
    );
    data.cells.insert(
        ("astro".to_string(), 0),
        Cell::Outdated(vec![changed(&format!("--\n--\n+{evil}\n-<img src=x>\n"))]),
    );

    let html = page(&data);

    assert_eq!(
        html.matches("<script").count(),
        1,
        "only the page's own script"
    );
    assert!(!html.contains("<img"), "no tag from a value");
    assert!(!html.contains("<b>") && !html.contains("<i>") && !html.contains("<u>"));
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
}

#[test]
fn the_matrix_has_a_cell_for_every_skill_and_project() {
    let html = page(&data());

    let matrix = &html[html.find("<table class=\"matrix\"").unwrap()..];
    let matrix = &matrix[..matrix.find("</table>").unwrap()];
    // Three skills and one folder the vault lacks, two projects each.
    assert_eq!(matrix.matches("<td").count(), 4 * 2, "{matrix}");
    for class in ["current", "outdated", "missing", "notinvault", "absent"] {
        assert!(matrix.contains(&format!("class=\"{class}\"")), "{class}");
    }
    assert_eq!(
        matrix.matches("<th title=").count(),
        2,
        "a column per project"
    );
}

#[test]
fn an_outdated_cell_links_to_its_diff_and_the_diff_is_coloured() {
    let html = page(&data());

    // astro is the third skill (index 2) and project two is index 1.
    assert!(
        html.contains("href=\"#d-2-1\""),
        "the cell links to the diff"
    );
    assert!(html.contains("id=\"d-2-1\""), "the diff has that id");
    assert!(html.contains("<span class=\"del\">-old\n</span>"), "{html}");
    assert!(html.contains("<span class=\"add\">+new\n</span>"));
    assert!(html.contains("<span class=\"hunk\">@@ -1 +1 @@\n</span>"));
}

#[test]
fn a_long_diff_is_cut_and_says_so() {
    let mut data = data();
    let numbered: Vec<String> = (0..500).map(|n| format!("+line {n}")).collect();
    let lines = numbered.join("\n") + "\n";
    data.cells.insert(
        ("astro".to_string(), 1),
        Cell::Outdated(vec![changed(&format!("--\n--\n{lines}"))]),
    );

    let html = page(&data);

    assert!(html.contains("+line 399"));
    assert!(!html.contains("+line 400"));
    assert!(html.contains("100 more lines"));
}

#[test]
fn the_vault_is_a_tree_of_groups() {
    let html = page(&data());

    let tree = &html[html.find("class=\"tree\"").unwrap()..];
    let tree = &tree[..tree.find("<h2").unwrap()];
    assert!(tree.contains("<summary>web/</summary>"), "{tree}");
    let group = tree.find("<summary>web/</summary>").unwrap();
    assert!(
        tree.find(">astro</a>").unwrap() > group,
        "astro sits inside the group"
    );
    assert!(
        tree.find(">coding</a>").unwrap() < group,
        "top-level skills come first"
    );
    assert!(tree.contains("mandatory"));
}

#[test]
fn the_filter_has_something_to_match_on_every_row() {
    let html = page(&data());

    assert!(
        html.contains("data-find=\"coding  write code\""),
        "name, group, description"
    );
    assert!(html.contains("data-find=\"astro web astro sites\""));
    assert!(html.contains("id=\"filter\""));
    assert!(html.contains("id=\"nomatch\" hidden"));
}

#[test]
fn problems_appear_only_when_there_are_some() {
    let calm = page(&data());
    assert!(!calm.contains("Needs a look"));

    let mut data = data();
    data.issues.push(ScanIssue {
        path: PathBuf::from("/p/.agents/.stage-1-2-3"),
        message: "left over".to_string(),
    });
    data.missing_bridges.push((0, "claude".to_string()));
    data.project_problems
        .push((1, "a link is in the way".to_string()));
    let html = page(&data);

    assert!(html.contains("Needs a look"));
    assert!(html.contains("left over"));
    assert!(html.contains("the claude link to .agents/skills is not made"));
    assert!(html.contains("a link is in the way"));
}

#[test]
fn the_same_data_gives_the_same_page_and_an_empty_vault_renders() {
    assert_eq!(page(&data()), page(&data()));

    let empty = ReportData {
        vault: PathBuf::from("/v"),
        root: PathBuf::from("/r"),
        skills: Vec::new(),
        extra: Vec::new(),
        projects: Vec::new(),
        cells: BTreeMap::new(),
        project_problems: Vec::new(),
        missing_bridges: Vec::new(),
        issues: Vec::new(),
    };
    let html = page(&empty);
    assert!(html.contains("0 skills in the vault"));
    assert!(html.contains("</html>"));
}

#[test]
fn projects_are_named_below_the_root_and_by_their_last_parts_elsewhere() {
    let mut data = data();
    data.projects[1] = PathBuf::from("/elsewhere/deep/folder/two");

    let html = page(&data);

    assert!(
        html.contains("<span>one</span>"),
        "below the root: just the rest"
    );
    assert!(
        html.contains("<span>folder/two</span>"),
        "outside the root: the last two parts"
    );
    assert!(
        html.contains("title=\"/home/me/projects/one\""),
        "the full path stays in the tooltip"
    );
}

#[test]
fn one_differing_file_is_singular() {
    let html = page(&data());

    assert!(html.contains("outdated, 1 file differs"), "{html}");
    let mut data = data();
    data.cells.insert(
        ("astro".to_string(), 1),
        Cell::Outdated(vec![changed("--\n--\n+a\n"), changed("--\n--\n+b\n")]),
    );
    assert!(page(&data).contains("outdated, 2 files differ"));
}

#[test]
fn nested_groups_nest() {
    let mut data = data();
    data.skills
        .push(skill("meta", &["web", "seo"], Some("Tags"), false));

    let html = page(&data);

    let tree = &html[html.find("class=\"tree\"").unwrap()..];
    let tree = &tree[..tree.find("<h2").unwrap()];
    let web = tree.find("<summary>web/</summary>").unwrap();
    let seo = tree.find("<summary>seo/</summary>").unwrap();
    let meta = tree.find(">meta</a>").unwrap();
    assert!(
        web < seo && seo < meta,
        "a skill sits in its deepest group: {tree}"
    );
}
