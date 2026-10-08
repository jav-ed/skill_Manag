//! The text and the JSON document of `diff`.

use std::path::PathBuf;

use serde::Serialize;
use skillmirror_core::ops::{DiffKind, DiffReport, FileDiff, SkillDiff, Skipped};

use super::human::plural;
use super::style::{ADDED, HEADER, HUNK, MUTED, NAME, REMOVED};

fn kind_name(kind: DiffKind) -> &'static str {
    match kind {
        DiffKind::Added => "added",
        DiffKind::Modified => "modified",
        DiffKind::ModeChanged => "mode_changed",
        DiffKind::Removed => "removed",
    }
}

fn skipped_name(skipped: &Skipped) -> String {
    match skipped {
        Skipped::Binary => "binary".to_string(),
        Skipped::TooLarge => "too_large".to_string(),
        Skipped::Unreadable(why) => format!("unreadable: {why}"),
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct FileRow {
    path: String,
    change: &'static str,
    lines_added: usize,
    lines_removed: usize,
    /// Why the lines are not in `diff`: `binary`, `too_large` or `unreadable: ...`.
    not_shown: Option<String>,
    mode_before: Option<String>,
    mode_after: Option<String>,
    /// A unified diff, empty when the lines are not shown.
    diff: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct SkillRow {
    #[serde(serialize_with = "super::lossy::path")]
    project: PathBuf,
    skill: String,
    files: Vec<FileRow>,
}

#[derive(Debug, Serialize)]
pub(crate) struct FailedRow {
    #[serde(serialize_with = "super::lossy::path")]
    project: PathBuf,
    skill: String,
    message: String,
}

#[derive(Serialize)]
pub(crate) struct DiffJson {
    skills: Vec<SkillRow>,
    failed: Vec<FailedRow>,
}

impl DiffJson {
    pub(crate) fn of(report: &DiffReport) -> Self {
        Self {
            skills: report
                .skills
                .iter()
                .map(|s| SkillRow {
                    project: s.target.project.clone(),
                    skill: s.target.skill.clone(),
                    files: s.files.iter().map(file_row).collect(),
                })
                .collect(),
            failed: report
                .failed
                .iter()
                .map(|(target, error)| FailedRow {
                    project: target.project.clone(),
                    skill: target.skill.clone(),
                    message: error.to_string(),
                })
                .collect(),
        }
    }

    pub(crate) fn render(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

fn file_row(file: &FileDiff) -> FileRow {
    FileRow {
        path: file.path.to_string_lossy().into_owned(),
        change: kind_name(file.kind),
        lines_added: file.added_lines,
        lines_removed: file.removed_lines,
        not_shown: file.skipped.as_ref().map(skipped_name),
        mode_before: file.modes.map(|(before, _)| format!("{before:o}")),
        mode_after: file.modes.map(|(_, after)| format!("{after:o}")),
        diff: file.text.clone(),
    }
}

/// Each skill folder with its files as unified diffs, or with `stat` only the line counts per file.
pub(crate) fn render_diff(report: &DiffReport, stat: bool) -> String {
    let mut out = String::new();
    for skill in &report.skills {
        putln!(
            out,
            "\n{HEADER}● {}{HEADER:#}  {NAME}{}{NAME:#}",
            skill.target.project.display(),
            skill.target.skill
        );
        if stat {
            out.push_str(&stat_lines(skill));
        } else {
            for file in &skill.files {
                out.push_str(&file_text(file));
            }
        }
    }
    let skills = plural(report.skills.len(), "skill folder");
    let files = plural(report.files(), "file");
    let differ = if report.skills.len() == 1 {
        "differs"
    } else {
        "differ"
    };
    put!(out, "\n{skills} {differ}, {files} would change\n");
    out
}

fn stat_lines(skill: &SkillDiff) -> String {
    let mut out = String::new();
    let width = skill
        .files
        .iter()
        .map(|f| f.path.to_string_lossy().chars().count())
        .max()
        .unwrap_or(0);
    let (mut added, mut removed) = (0, 0);
    for file in &skill.files {
        added += file.added_lines;
        removed += file.removed_lines;
        let path = file.path.to_string_lossy();
        let what = match (&file.skipped, file.kind) {
            (Some(skipped), _) => skipped_name(skipped),
            (None, DiffKind::ModeChanged) => file
                .modes
                .map_or_else(String::new, |(a, b)| format!("mode {a:o} → {b:o}")),
            (None, _) => format!(
                "{ADDED}+{}{ADDED:#} {REMOVED}-{}{REMOVED:#}",
                file.added_lines, file.removed_lines
            ),
        };
        putln!(out, "  {path:<width$} | {:<8} {what}", kind_name(file.kind));
    }
    putln!(
        out,
        "  {MUTED}{} changed, {ADDED}+{added}{ADDED:#} {REMOVED}-{removed}{REMOVED:#}{MUTED:#}",
        plural(skill.files.len(), "file")
    );
    out
}

fn file_text(file: &FileDiff) -> String {
    let mut out = String::new();
    let path = file.path.display();
    match (&file.skipped, file.kind) {
        (Some(skipped), _) => {
            putln!(
                out,
                "{HEADER}{} {path}{HEADER:#} {MUTED}(lines not shown: {}){MUTED:#}",
                kind_name(file.kind),
                skipped_name(skipped)
            );
        }
        (None, DiffKind::ModeChanged) => {
            let (before, after) = file.modes.unwrap_or((0, 0));
            putln!(
                out,
                "{HEADER}mode {path}{HEADER:#} {MUTED}{before:o} → {after:o}{MUTED:#}"
            );
        }
        (None, _) => {
            for line in file.text.lines() {
                let style = if line.starts_with("+++") || line.starts_with("---") {
                    HEADER
                } else if line.starts_with("@@") {
                    HUNK
                } else if line.starts_with('+') {
                    ADDED
                } else if line.starts_with('-') {
                    REMOVED
                } else {
                    MUTED
                };
                putln!(out, "{style}{line}{style:#}");
            }
        }
    }
    out
}
