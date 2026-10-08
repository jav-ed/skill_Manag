//! What a run produced, shaped for printing. Human and JSON output render the same rows.

use std::collections::BTreeSet;
use std::path::PathBuf;

use serde::Serialize;
use skillmirror_core::Hint;
use skillmirror_core::apply::{Applied, Failure, Outcome};
use skillmirror_core::plan::{ChangeKind, PlanEntry, PlanKind, SkillPlan};

/// Whether rows describe what would happen or what happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tense {
    Plan,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Kind {
    New,
    Update,
    Unchanged,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ChangeRow {
    pub(crate) path: String,
    pub(crate) kind: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ErrorRow {
    pub(crate) message: String,
    pub(crate) hint: Option<String>,
}

/// One skill in one project.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct Row {
    pub(crate) project: PathBuf,
    pub(crate) skill: String,
    pub(crate) status: Kind,
    pub(crate) files: usize,
    pub(crate) changes: Vec<ChangeRow>,
    pub(crate) removed: Vec<String>,
    pub(crate) error: Option<ErrorRow>,
}

impl Row {
    pub(crate) fn from_plan(entry: &PlanEntry) -> Self {
        match &entry.result {
            Ok(plan) => Self::with_plan(plan, kind_of_plan(plan.kind), None),
            Err(e) => Self::failed(
                entry.target.project.clone(),
                entry.target.skill.clone(),
                e.to_string(),
                e.hint(),
            ),
        }
    }

    pub(crate) fn from_applied(applied: &Applied) -> Self {
        let kind = match &applied.outcome {
            Outcome::Unchanged => Kind::Unchanged,
            Outcome::Created => Kind::New,
            Outcome::Updated => Kind::Update,
            Outcome::Failed(_) => Kind::Failed,
        };
        let error = match &applied.outcome {
            Outcome::Failed(Failure::Plan(e)) => Some(ErrorRow {
                message: e.to_string(),
                hint: e.hint(),
            }),
            Outcome::Failed(Failure::Apply(e)) => Some(ErrorRow {
                message: e.to_string(),
                hint: e.hint(),
            }),
            _ => None,
        };
        if let Some(plan) = &applied.plan {
            return Self::with_plan(plan, kind, error);
        }
        let ErrorRow { message, hint } = error.unwrap_or(ErrorRow {
            message: "failed".to_string(),
            hint: None,
        });
        Self::failed(
            applied.target.project.clone(),
            applied.target.skill.clone(),
            message,
            hint,
        )
    }

    fn with_plan(plan: &SkillPlan, status: Kind, error: Option<ErrorRow>) -> Self {
        Self {
            project: plan.target.project.clone(),
            skill: plan.target.skill.clone(),
            status,
            files: plan.file_count(),
            changes: plan
                .changes
                .iter()
                .map(|c| ChangeRow {
                    path: c.rel.to_string_lossy().into_owned(),
                    kind: change_name(c.kind),
                })
                .collect(),
            removed: plan
                .removed
                .iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect(),
            error,
        }
    }

    fn failed(project: PathBuf, skill: String, message: String, hint: Option<String>) -> Self {
        Self {
            project,
            skill,
            status: Kind::Failed,
            files: 0,
            changes: Vec::new(),
            removed: Vec::new(),
            error: Some(ErrorRow { message, hint }),
        }
    }
}

fn kind_of_plan(kind: PlanKind) -> Kind {
    match kind {
        PlanKind::Create => Kind::New,
        PlanKind::Update => Kind::Update,
        PlanKind::Unchanged => Kind::Unchanged,
    }
}

fn change_name(kind: ChangeKind) -> &'static str {
    match kind {
        ChangeKind::Added => "added",
        ChangeKind::Modified => "modified",
        ChangeKind::ModeChanged => "mode_changed",
    }
}

/// Totals over a list of rows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub(crate) struct Summary {
    pub(crate) skills: usize,
    pub(crate) projects: usize,
    pub(crate) new: usize,
    pub(crate) update: usize,
    pub(crate) unchanged: usize,
    pub(crate) failed: usize,
}

impl Summary {
    pub(crate) fn of(rows: &[Row]) -> Self {
        let mut summary = Self {
            skills: rows.len(),
            projects: rows
                .iter()
                .map(|r| &r.project)
                .collect::<BTreeSet<_>>()
                .len(),
            ..Self::default()
        };
        for row in rows {
            match row.status {
                Kind::New => summary.new += 1,
                Kind::Update => summary.update += 1,
                Kind::Unchanged => summary.unchanged += 1,
                Kind::Failed => summary.failed += 1,
            }
        }
        summary
    }

    /// Targets that a write would change.
    pub(crate) fn changes(&self) -> usize {
        self.new + self.update
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DeleteStatus {
    WouldDelete,
    Deleted,
    Failed { message: String },
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DeleteRow {
    pub(crate) project: PathBuf,
    pub(crate) skill: String,
    pub(crate) status: DeleteStatus,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct InstalledRow {
    pub(crate) project: PathBuf,
    pub(crate) skill: String,
    pub(crate) in_vault: Option<bool>,
}

/// One skill of the vault.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct SkillRow {
    pub(crate) name: String,
    pub(crate) group: Vec<String>,
    pub(crate) mandatory: bool,
}
