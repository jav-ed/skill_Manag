//! The changes a plan would make, as the lines the diff page shows.

use skillmirror_core::agents::AgentsPlan;
use skillmirror_core::ops::{DiffKind, FileDiff, SkillDiff, Skipped};

use crate::results::short_path;

/// More lines than this are cut; the command line `diff` shows everything.
const MAX_LINES: usize = 5000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tone {
    /// A skill in a project.
    Skill,
    /// A file of it.
    File,
    Added,
    Removed,
    Hunk,
    Plain,
    /// Why no lines are shown.
    Note,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiffLine {
    pub(crate) tone: Tone,
    pub(crate) text: String,
}

fn line(tone: Tone, text: impl Into<String>) -> DiffLine {
    DiffLine {
        tone,
        text: text.into(),
    }
}

/// The lines of the AGENTS.md files a plan would write.
pub(crate) fn lines_of_agents(plan: &AgentsPlan) -> Vec<DiffLine> {
    let mut out = Vec::new();
    for entry in plan.entries.iter().filter(|e| e.writes()) {
        if !out.is_empty() {
            out.push(line(Tone::Plain, ""));
        }
        out.push(line(
            Tone::Skill,
            format!(
                "AGENTS.md in {}",
                short_path(&entry.project.to_string_lossy())
            ),
        ));
        // The first two lines are the `---` and `+++` headers; the line above names the file.
        for text in entry.diff().lines().skip(2) {
            out.push(text_line(text));
        }
    }
    cut(out)
}

fn text_line(text: &str) -> DiffLine {
    let tone = if text.starts_with("@@") {
        Tone::Hunk
    } else if text.starts_with('+') {
        Tone::Added
    } else if text.starts_with('-') {
        Tone::Removed
    } else {
        Tone::Plain
    };
    line(tone, format!("    {text}"))
}

fn cut(mut out: Vec<DiffLine>) -> Vec<DiffLine> {
    if out.len() > MAX_LINES {
        let more = out.len() - MAX_LINES;
        out.truncate(MAX_LINES);
        out.push(line(
            Tone::Note,
            format!("… {more} more lines are not shown; `skillmirror diff` shows them all"),
        ));
    }
    out
}

pub(crate) fn lines_of(skills: &[SkillDiff]) -> Vec<DiffLine> {
    let mut out = Vec::new();
    for skill in skills {
        if !out.is_empty() {
            out.push(line(Tone::Plain, ""));
        }
        out.push(line(
            Tone::Skill,
            format!(
                "{} in {}",
                skill.target.skill,
                short_path(&skill.target.project.to_string_lossy())
            ),
        ));
        for file in &skill.files {
            file_lines(file, &mut out);
        }
    }
    cut(out)
}

fn file_lines(file: &FileDiff, out: &mut Vec<DiffLine>) {
    let what = match file.kind {
        DiffKind::Added => "new file",
        DiffKind::Modified => "changed",
        DiffKind::ModeChanged => "permissions",
        DiffKind::Removed => "removed",
    };
    out.push(line(
        Tone::File,
        format!(
            "  {} ({what}, +{} -{})",
            file.path.display(),
            file.added_lines,
            file.removed_lines
        ),
    ));
    if let Some((old, new)) = file.modes {
        out.push(line(Tone::Note, format!("    mode {old:o} → {new:o}")));
    }
    match &file.skipped {
        Some(Skipped::Binary) => out.push(line(Tone::Note, "    binary file, lines not shown")),
        Some(Skipped::TooLarge) => out.push(line(Tone::Note, "    too large, lines not shown")),
        Some(Skipped::Unreadable(reason)) => {
            out.push(line(Tone::Note, format!("    cannot be read: {reason}")));
        }
        None => {}
    }
    // The first two lines of the text are the `---` and `+++` headers; the page names the file itself.
    for text in file.text.lines().skip(2) {
        out.push(text_line(text));
    }
}
