//! What a finished run shows: one entry per skill, one line per project.

use std::collections::BTreeMap;

use skillmirror_core::apply::{ApplyReport, Failure, Outcome};
use skillmirror_core::backup::Finished;
use skillmirror_core::ops::DeleteReport;
use skillmirror_core::plan::SkillPlan;

use crate::session::describe;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Sync,
    Push,
    Delete,
}

impl Kind {
    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Sync => "Sync results",
            Self::Push => "Push results",
            Self::Delete => "Delete results",
        }
    }

    pub(crate) fn verb(self) -> &'static str {
        match self {
            Self::Sync => "synced to",
            Self::Push => "pushed to",
            Self::Delete => "deleted from",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Done {
    Same,
    Wrote,
    Failed,
}

/// One project's outcome for one skill.
#[derive(Debug, Clone)]
pub(crate) struct Line {
    pub(crate) project: String,
    pub(crate) ok: bool,
    pub(crate) text: String,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct SkillResult {
    pub(crate) name: String,
    /// Projects that were written (or, for a delete, removed from).
    pub(crate) ok: usize,
    /// Projects that already matched the vault.
    pub(crate) unchanged: usize,
    pub(crate) failed: usize,
    /// Files written into the projects that changed.
    pub(crate) files: usize,
    pub(crate) lines: Vec<Line>,
}

#[derive(Debug, Clone)]
pub(crate) struct Results {
    pub(crate) kind: Kind,
    pub(crate) skills: Vec<SkillResult>,
    /// Things that went well but need a hand, such as an old copy that could not be removed.
    pub(crate) warnings: Vec<String>,
    /// The backup run that holds what this job replaced or removed.
    pub(crate) backup: Option<String>,
}

impl Results {
    /// Notes where the backup is, and a failed clean-up of old backups.
    pub(crate) fn with_backup(mut self, finished: &Finished) -> Self {
        if let Some(error) = &finished.prune_error {
            self.warnings
                .push(format!("Old backups could not be removed: {error}"));
        }
        if finished.stored > 0 {
            self.backup = Some(finished.id.clone());
        }
        self
    }

    pub(crate) fn failed(&self) -> usize {
        self.skills.iter().map(|s| s.failed).sum()
    }

    pub(crate) fn from_applied(report: ApplyReport, kind: Kind) -> Self {
        let mut by_skill: BTreeMap<String, SkillResult> = BTreeMap::new();
        let mut warnings = Vec::new();
        for applied in report.applied {
            if let Some(left) = &applied.leftover {
                warnings.push(format!(
                    "The old copy {} could not be removed ({}); delete it by hand.",
                    left.path.display(),
                    left.reason
                ));
            }
            let skill = entry(&mut by_skill, &applied.target.skill);
            let project = applied.target.project.display().to_string();
            let files = applied.plan.as_ref().map_or(0, SkillPlan::file_count);
            let (done, text) = match applied.outcome {
                Outcome::Unchanged => (Done::Same, "up to date".to_string()),
                Outcome::Created => (Done::Wrote, format!("created ({files} files)")),
                Outcome::Updated => (Done::Wrote, format!("updated ({files} files)")),
                Outcome::Failed(Failure::Plan(e)) => (Done::Failed, describe(&e)),
                Outcome::Failed(Failure::Apply(e)) => (Done::Failed, describe(&e)),
            };
            let ok = done != Done::Failed;
            match done {
                Done::Same => skill.unchanged += 1,
                Done::Wrote => {
                    skill.ok += 1;
                    skill.files += files;
                }
                Done::Failed => skill.failed += 1,
            }
            skill.lines.push(Line { project, ok, text });
        }
        Self {
            kind,
            skills: by_skill.into_values().collect(),
            warnings,
            backup: None,
        }
    }

    pub(crate) fn from_deleted(report: DeleteReport) -> Self {
        let mut by_skill: BTreeMap<String, SkillResult> = BTreeMap::new();
        for deleted in report.deleted {
            let skill = entry(&mut by_skill, &deleted.target.skill);
            let project = deleted.target.project.display().to_string();
            let (ok, text) = match deleted.result {
                Ok(()) => (true, "deleted".to_string()),
                Err(e) => (false, e.to_string()),
            };
            if ok {
                skill.ok += 1;
            } else {
                skill.failed += 1;
            }
            skill.lines.push(Line { project, ok, text });
        }
        Self {
            kind: Kind::Delete,
            skills: by_skill.into_values().collect(),
            warnings: Vec::new(),
            backup: None,
        }
    }
}

fn entry<'a>(map: &'a mut BTreeMap<String, SkillResult>, name: &str) -> &'a mut SkillResult {
    map.entry(name.to_string()).or_insert_with(|| SkillResult {
        name: name.to_string(),
        ..SkillResult::default()
    })
}

/// The last two path components, like `parent/project`.
pub(crate) fn short_path(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    let from = parts.len().saturating_sub(2);
    parts.get(from..).unwrap_or_default().join("/")
}
