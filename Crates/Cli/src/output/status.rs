//! Rows, the JSON document and the text of `status`.

use std::path::PathBuf;

use serde::Serialize;
use skillmirror_core::ops::{Outdated, ProjectStatus, StatusReport};

use super::human::{pad, plural};
use super::style::{ERROR, HEADER, MUTED, NAME, SUCCESS, WARNING};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OutdatedRow {
    skill: String,
    files_added: usize,
    files_changed: usize,
    files_removed: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProblemRow {
    skill: String,
    message: String,
    hint: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProjectRow {
    #[serde(serialize_with = "super::lossy::path")]
    project: PathBuf,
    /// `in_sync`, `drift` (sync or push would change something) or `problem` (only comparisons failed).
    status: &'static str,
    up_to_date: Vec<String>,
    outdated: Vec<OutdatedRow>,
    missing_mandatory: Vec<String>,
    /// Targets of the vault config whose link to `.agents/skills` is not made yet.
    missing_bridges: Vec<String>,
    not_in_vault: Vec<String>,
    failed: Vec<ProblemRow>,
}

impl ProjectRow {
    pub(crate) fn of(project: &ProjectStatus) -> Self {
        let status = if project.drifts() {
            "drift"
        } else if project.failed.is_empty() {
            "in_sync"
        } else {
            "problem"
        };
        Self {
            project: project.project.clone(),
            status,
            up_to_date: project.up_to_date.clone(),
            outdated: project.outdated.iter().map(outdated_row).collect(),
            missing_mandatory: project.missing_mandatory.clone(),
            missing_bridges: project.missing_bridges.clone(),
            not_in_vault: project.not_in_vault.clone(),
            failed: project
                .failed
                .iter()
                .map(|p| ProblemRow {
                    skill: p.skill.clone(),
                    message: p.message.clone(),
                    hint: p.hint.clone(),
                })
                .collect(),
        }
    }
}

fn outdated_row(o: &Outdated) -> OutdatedRow {
    OutdatedRow {
        skill: o.skill.clone(),
        files_added: o.added,
        files_changed: o.changed,
        files_removed: o.removed,
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize)]
pub(crate) struct StatusSummary {
    projects: usize,
    in_sync: usize,
    drift: usize,
    outdated_skills: usize,
    missing_mandatory: usize,
    missing_bridges: usize,
    not_in_vault: usize,
    failed: usize,
}

impl StatusSummary {
    pub(crate) fn of(report: &StatusReport) -> Self {
        let sum = |count: fn(&ProjectStatus) -> usize| report.projects.iter().map(count).sum();
        Self {
            projects: report.projects.len(),
            in_sync: report
                .projects
                .iter()
                .filter(|p| !p.drifts() && p.failed.is_empty())
                .count(),
            drift: report.drifting(),
            outdated_skills: sum(|p| p.outdated.len()),
            missing_mandatory: sum(|p| p.missing_mandatory.len()),
            missing_bridges: sum(|p| p.missing_bridges.len()),
            not_in_vault: sum(|p| p.not_in_vault.len()),
            failed: report.failed(),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct StatusJson<'a> {
    projects: &'a [ProjectRow],
    summary: StatusSummary,
}

impl<'a> StatusJson<'a> {
    pub(crate) fn new(projects: &'a [ProjectRow], summary: StatusSummary) -> Self {
        Self { projects, summary }
    }

    pub(crate) fn render(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Projects that need a word are listed with their lines; the calm ones only with `show_all`.
pub(crate) fn render_status(report: &StatusReport, show_all: bool) -> String {
    let mut out = String::new();
    for project in &report.projects {
        let calm =
            !project.drifts() && project.failed.is_empty() && project.not_in_vault.is_empty();
        if calm && !show_all {
            continue;
        }
        putln!(out, "\n{HEADER}● {}{HEADER:#}", project.project.display());
        for o in &project.outdated {
            putln!(
                out,
                "  {WARNING}~{WARNING:#} {NAME}{}{NAME:#} {MUTED}outdated ({}){MUTED:#}",
                pad(&o.skill),
                file_counts(o)
            );
        }
        for skill in &project.missing_mandatory {
            putln!(
                out,
                "  {WARNING}+{WARNING:#} {NAME}{}{NAME:#} {MUTED}mandatory, not installed{MUTED:#}",
                pad(skill)
            );
        }
        for name in &project.missing_bridges {
            putln!(
                out,
                "  {WARNING}+{WARNING:#} {NAME}{}{NAME:#} {MUTED}bridge to .agents/skills not made (`skillmirror bridge`){MUTED:#}",
                pad(name)
            );
        }
        for problem in &project.failed {
            putln!(
                out,
                "  {ERROR}✗{ERROR:#} {NAME}{}{NAME:#} {}",
                pad(&problem.skill),
                problem.message
            );
            if let Some(hint) = &problem.hint {
                putln!(out, "      {MUTED}hint: {hint}{MUTED:#}");
            }
        }
        for skill in &project.not_in_vault {
            putln!(
                out,
                "  {MUTED}? {} not in the vault, sync leaves it alone{MUTED:#}",
                pad(skill)
            );
        }
        if show_all {
            for skill in &project.up_to_date {
                putln!(
                    out,
                    "  {SUCCESS}✓{SUCCESS:#} {NAME}{}{NAME:#} {MUTED}up to date{MUTED:#}",
                    pad(skill)
                );
            }
        }
    }
    let summary = StatusSummary::of(report);
    let style = if summary.drift > 0 || summary.failed > 0 {
        WARNING
    } else {
        SUCCESS
    };
    let mut parts = vec![format!("{} in sync", summary.in_sync)];
    if summary.drift > 0 {
        parts.push(format!(
            "{} differ ({} outdated, {} mandatory missing{})",
            summary.drift,
            plural(summary.outdated_skills, "skill folder"),
            summary.missing_mandatory,
            if summary.missing_bridges > 0 {
                format!(", {} missing", plural(summary.missing_bridges, "bridge"))
            } else {
                String::new()
            }
        ));
    }
    if summary.failed > 0 {
        parts.push(format!("{} with a problem", summary.failed));
    }
    put!(
        out,
        "\n{style}{}: {}{style:#}\n",
        plural(summary.projects, "project"),
        parts.join(", ")
    );
    out
}

fn file_counts(o: &Outdated) -> String {
    let mut parts = Vec::new();
    for (count, what) in [
        (o.added, "added"),
        (o.changed, "changed"),
        (o.removed, "removed"),
    ] {
        if count > 0 {
            parts.push(format!("{} {what}", plural(count, "file")));
        }
    }
    if parts.is_empty() {
        "differs".to_string()
    } else {
        parts.join(", ")
    }
}
