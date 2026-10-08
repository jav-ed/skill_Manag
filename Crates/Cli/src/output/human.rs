//! Plain-text rendering. Strings are built here and printed by the caller, so tests need no terminal.

use super::style::{ERROR, HEADER, MUTED, NAME, SUCCESS, WARNING};
use super::view::{DeleteRow, DeleteStatus, InstalledRow, Kind, Row, SkillRow, Summary, Tense};

/// Width of the skill name column.
const NAME_WIDTH: usize = 30;

pub(super) fn plural(count: usize, word: &str) -> String {
    if count == 1 {
        format!("{count} {word}")
    } else {
        format!("{count} {word}s")
    }
}

pub(super) fn pad(name: &str) -> String {
    let missing = NAME_WIDTH.saturating_sub(name.chars().count());
    format!("{name}{}", " ".repeat(missing))
}

/// Rows grouped by project. Unchanged rows appear only with `show_all`.
pub(crate) fn render_rows(rows: &[Row], tense: Tense, show_all: bool) -> String {
    let mut out = String::new();
    let mut current: Option<&std::path::Path> = None;
    for row in rows
        .iter()
        .filter(|r| show_all || r.status != Kind::Unchanged)
    {
        if current != Some(row.project.as_path()) {
            current = Some(row.project.as_path());
            putln!(out, "\n{HEADER}● {}{HEADER:#}", row.project.display());
        }
        let name = pad(&row.skill);
        let (sym, style, text) = describe(row, tense);
        putln!(
            out,
            "  {style}{sym}{style:#} {NAME}{name}{NAME:#} {MUTED}{text}{MUTED:#}"
        );
        if let Some(hint) = row.error.as_ref().and_then(|e| e.hint.as_ref()) {
            putln!(out, "      {MUTED}hint: {hint}{MUTED:#}");
        }
    }
    out
}

fn describe(row: &Row, tense: Tense) -> (&'static str, anstyle::Style, String) {
    let done = tense == Tense::Done;
    match row.status {
        Kind::New => (
            "+",
            SUCCESS,
            format!(
                "{} ({})",
                if done { "added" } else { "new" },
                plural(row.files, "file")
            ),
        ),
        Kind::Update => {
            let mut detail = format!("{} changed", row.changes.len());
            if !row.removed.is_empty() {
                put!(detail, ", {} removed", row.removed.len());
            }
            (
                "~",
                WARNING,
                format!("{} ({detail})", if done { "updated" } else { "update" }),
            )
        }
        Kind::Unchanged => ("=", MUTED, "up to date".to_string()),
        Kind::Failed => (
            "✗",
            ERROR,
            row.error
                .as_ref()
                .map(|e| e.message.clone())
                .unwrap_or_default(),
        ),
    }
}

/// One closing line: totals by outcome, then the mode in parentheses when nothing was written.
pub(crate) fn render_summary(summary: &Summary, tense: Tense, note: &str) -> String {
    let (new, update, same) = match tense {
        Tense::Plan => ("to add", "to update", "up to date"),
        Tense::Done => ("added", "updated", "unchanged"),
    };
    let parts: Vec<String> = [
        (summary.new, new),
        (summary.update, update),
        (summary.unchanged, same),
        (summary.failed, "failed"),
    ]
    .into_iter()
    .filter(|(count, _)| *count > 0)
    .map(|(count, label)| format!("{count} {label}"))
    .collect();
    let detail = if parts.is_empty() {
        "nothing to do".to_string()
    } else {
        parts.join(", ")
    };
    let style = if summary.failed > 0 { WARNING } else { SUCCESS };
    let note = if note.is_empty() {
        String::new()
    } else {
        format!(" ({note})")
    };
    format!(
        "\n{style}{} in {}: {detail}{note}{style:#}\n",
        plural(summary.skills, "skill"),
        plural(summary.projects, "project")
    )
}

pub(crate) fn render_delete(rows: &[DeleteRow], dry_run: bool) -> String {
    let mut out = String::new();
    let (mut ok, mut failed) = (0, 0);
    for row in rows {
        let name = pad(&row.skill);
        match &row.status {
            DeleteStatus::WouldDelete | DeleteStatus::Deleted => {
                ok += 1;
                let verb = if dry_run {
                    "would delete from"
                } else {
                    "deleted from"
                };
                putln!(
                    out,
                    "  {WARNING}-{WARNING:#} {NAME}{name}{NAME:#} {MUTED}{verb} {}{MUTED:#}",
                    row.project.display()
                );
            }
            DeleteStatus::Failed { message } => {
                failed += 1;
                putln!(out, "  {ERROR}✗{ERROR:#} {NAME}{name}{NAME:#} {message}");
            }
        }
    }
    let verb = if dry_run {
        "would be removed"
    } else {
        "removed"
    };
    let note = if dry_run { " (dry run)" } else { "" };
    let failed_text = if failed > 0 {
        format!(", {failed} failed")
    } else {
        String::new()
    };
    put!(
        out,
        "\n{} {verb}{failed_text}{note}\n",
        plural(ok, "folder")
    );
    out
}

pub(crate) fn render_installed(rows: &[InstalledRow]) -> String {
    let mut out = String::new();
    for row in rows {
        let name = pad(&row.skill);
        let mark = if row.in_vault == Some(false) {
            format!("  {WARNING}(not in vault){WARNING:#}")
        } else {
            String::new()
        };
        putln!(out, "{NAME}{name}{NAME:#} {}{mark}", row.project.display());
    }
    let projects: std::collections::BTreeSet<_> = rows.iter().map(|r| &r.project).collect();
    put!(
        out,
        "\n{} installed in {}\n",
        plural(rows.len(), "skill"),
        plural(projects.len(), "project")
    );
    out
}

/// The vault as a folder tree: groups are headers, skills hang below them.
pub(crate) fn render_vault(rows: &[SkillRow]) -> String {
    let mut out = String::new();
    let mut shown: Vec<&String> = Vec::new();
    for row in rows {
        let common = shown
            .iter()
            .zip(&row.group)
            .take_while(|(a, b)| ***a == **b)
            .count();
        shown.truncate(common);
        for (depth, name) in row.group.iter().enumerate().skip(common) {
            putln!(out, "{}{HEADER}{name}/{HEADER:#}", "  ".repeat(depth));
            shown.push(name);
        }
        let mark = if row.mandatory {
            format!("  {MUTED}(mandatory){MUTED:#}")
        } else {
            String::new()
        };
        putln!(
            out,
            "{}{NAME}{}{NAME:#}{mark}",
            "  ".repeat(row.group.len()),
            row.name
        );
    }
    let groups: std::collections::BTreeSet<_> = rows
        .iter()
        .flat_map(|r| (1..=r.group.len()).map(|n| r.group.get(..n).unwrap_or_default().join("/")))
        .collect();
    put!(
        out,
        "\n{} in {}\n",
        plural(rows.len(), "skill"),
        plural(groups.len(), "group")
    );
    out
}
