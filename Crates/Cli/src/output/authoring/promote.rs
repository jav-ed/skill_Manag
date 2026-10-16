//! What `promote` prints: the plan, and what was replaced.

use std::path::PathBuf;

use serde::Serialize;
use skillmirror_core::ops::{PromotePlan, PromoteReport, SkillPromotion};

use crate::output::human::plural;
use crate::output::inspect::changes::{edits_words, kind_word};
use crate::output::style::{ERROR, HEADER, MUTED, NAME, WARNING};

#[derive(Debug, Serialize)]
struct PromotedRow {
    name: String,
    #[serde(serialize_with = "crate::output::lossy::path")]
    dir: PathBuf,
    files: usize,
    edits: Vec<String>,
    /// `planned` on a dry run, else `replaced` or `failed`.
    result: &'static str,
    message: Option<String>,
    /// Files that were copied but that git did not take.
    left_out: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct PromoteJson {
    dry_run: bool,
    #[serde(serialize_with = "crate::output::lossy::path")]
    project: PathBuf,
    skills: Vec<PromotedRow>,
    /// The backup run that kept the vault's old folders.
    backup: Option<String>,
}

impl PromoteJson {
    pub(crate) fn planned(plan: &PromotePlan) -> Self {
        Self {
            dry_run: true,
            project: plan.project.clone(),
            skills: plan
                .skills
                .iter()
                .map(|s| row(s, "planned", None, Vec::new()))
                .collect(),
            backup: None,
        }
    }

    pub(crate) fn done(plan: &PromotePlan, report: &PromoteReport) -> Self {
        let skills = plan
            .skills
            .iter()
            .zip(&report.skills)
            .map(|(skill, outcome)| match &outcome.result {
                Ok(done) => row(
                    skill,
                    "replaced",
                    None,
                    done.left_out
                        .iter()
                        .map(|f| f.display().to_string())
                        .collect(),
                ),
                Err(e) => row(skill, "failed", Some(e.to_string()), Vec::new()),
            })
            .collect();
        Self {
            dry_run: false,
            project: plan.project.clone(),
            skills,
            backup: (report.backup.stored > 0).then(|| report.backup.id.clone()),
        }
    }

    pub(crate) fn render(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

fn row(
    skill: &SkillPromotion,
    result: &'static str,
    message: Option<String>,
    left_out: Vec<String>,
) -> PromotedRow {
    PromotedRow {
        name: skill.name.clone(),
        dir: skill.dest.clone(),
        files: skill.files.len(),
        edits: skill
            .edits
            .iter()
            .map(|e| format!("{} {}", e.rel.display(), kind_word(e.kind)))
            .collect(),
        result,
        message,
        left_out,
    }
}

/// What a promote would do, or is about to do.
pub(crate) fn render_promote_plan(plan: &PromotePlan, dry_run: bool) -> String {
    let mut out = String::new();
    putln!(
        out,
        "{HEADER}{}{HEADER:#} in the vault, with the copies from {}:",
        if dry_run { "Would replace" } else { "Replace" },
        plan.project.display()
    );
    for skill in &plan.skills {
        putln!(
            out,
            "  {NAME}{}{NAME:#} {MUTED}({}){MUTED:#} {}",
            skill.name,
            plural(skill.files.len(), "file"),
            edits_words(&skill.edits)
        );
    }
    out
}

pub(crate) fn render_promoted(report: &PromoteReport) -> String {
    let mut out = String::new();
    for outcome in &report.skills {
        match &outcome.result {
            Ok(done) => {
                putln!(
                    out,
                    "{HEADER}Replaced{HEADER:#} {NAME}{}{NAME:#} in the vault: {} ({})",
                    outcome.name,
                    done.dir.display(),
                    plural(done.files, "file")
                );
                if !done.left_out.is_empty() {
                    putln!(
                        out,
                        "{WARNING}{} not taken by git (the vault's .gitignore?): {}{WARNING:#}",
                        plural(done.left_out.len(), "file"),
                        done.left_out
                            .iter()
                            .map(|f| f.display().to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
            }
            Err(e) => putln!(
                out,
                "{ERROR}Not replaced{ERROR:#} {NAME}{}{NAME:#}: {e}",
                outcome.name
            ),
        }
    }
    if report.skills.iter().any(|o| o.result.is_ok()) {
        putln!(
            out,
            "{MUTED}Staged in git, not committed. Commit it in the vault when it is ready; sync and add copy what git tracks.{MUTED:#}"
        );
    }
    out
}
