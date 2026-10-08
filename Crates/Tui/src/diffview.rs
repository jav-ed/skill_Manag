//! The changes a plan would make, as the lines the diff page shows.

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
        let tone = if text.starts_with("@@") {
            Tone::Hunk
        } else if text.starts_with('+') {
            Tone::Added
        } else if text.starts_with('-') {
            Tone::Removed
        } else {
            Tone::Plain
        };
        out.push(line(tone, format!("    {text}")));
    }
}
