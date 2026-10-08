//! Status: how every project stands against the vault. Reads, never writes.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::{BridgeState, Workspace, installed, plan_bridges, plan_push, plan_sync};
use crate::Hint;
use crate::plan::{ChangeKind, PlanError, PlanKind, SkillPlan};
use crate::scan::ScanReport;

/// A skill the project has, whose folder differs from the vault. The counts are files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outdated {
    pub skill: String,
    pub added: usize,
    pub changed: usize,
    pub removed: usize,
}

/// A skill that could not be compared, with the reason and the hint that goes with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub skill: String,
    pub message: String,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectStatus {
    pub project: PathBuf,
    pub up_to_date: Vec<String>,
    pub outdated: Vec<Outdated>,
    /// Mandatory skills the project does not have; `push` would install them.
    pub missing_mandatory: Vec<String>,
    /// Installed folders the vault has no skill for; `sync` leaves them alone.
    pub not_in_vault: Vec<String>,
    /// Targets from the vault config whose link to `.agents/skills` does not exist yet; `bridge` makes it.
    pub missing_bridges: Vec<String>,
    pub failed: Vec<Problem>,
}

impl ProjectStatus {
    fn new(project: PathBuf) -> Self {
        Self {
            project,
            up_to_date: Vec::new(),
            outdated: Vec::new(),
            missing_mandatory: Vec::new(),
            not_in_vault: Vec::new(),
            missing_bridges: Vec::new(),
            failed: Vec::new(),
        }
    }

    /// Whether `sync` or `push` would change something here. A skill the vault lacks is no drift: nothing
    /// the tool does could fix it.
    pub fn drifts(&self) -> bool {
        !self.outdated.is_empty()
            || !self.missing_mandatory.is_empty()
            || !self.missing_bridges.is_empty()
    }
}

#[derive(Debug, Default)]
pub struct StatusReport {
    pub projects: Vec<ProjectStatus>,
}

impl StatusReport {
    pub fn drifting(&self) -> usize {
        self.projects.iter().filter(|p| p.drifts()).count()
    }

    pub fn failed(&self) -> usize {
        self.projects.iter().map(|p| p.failed.len()).sum()
    }
}

/// Compares every project with the vault. A mandatory name the vault does not have is a hard error, as
/// it is for `push`.
pub fn status(workspace: &Workspace, report: &ScanReport) -> Result<StatusReport, PlanError> {
    let mut projects: Vec<ProjectStatus> = report
        .skills_dirs
        .iter()
        .map(|dir| ProjectStatus::new(dir.project.clone()))
        .collect();
    let position: HashMap<PathBuf, usize> = projects
        .iter()
        .enumerate()
        .map(|(i, p)| (p.project.clone(), i))
        .collect();

    for entry in plan_sync(workspace, report).entries {
        let Some(status) = at(&position, &mut projects, &entry.target.project) else {
            continue;
        };
        let skill = entry.target.skill;
        match entry.result {
            Ok(plan) if plan.kind == PlanKind::Unchanged => status.up_to_date.push(skill),
            Ok(plan) => status.outdated.push(outdated(skill, &plan)),
            Err(error) => status.failed.push(problem(skill, &error)),
        }
    }
    for entry in plan_push(workspace, report)?.entries {
        let Some(status) = at(&position, &mut projects, &entry.target.project) else {
            continue;
        };
        let skill = entry.target.skill;
        let known = |status: &ProjectStatus| {
            status.up_to_date.contains(&skill)
                || status.outdated.iter().any(|o| o.skill == skill)
                || status.failed.iter().any(|f| f.skill == skill)
        };
        match entry.result {
            Ok(plan) if plan.kind == PlanKind::Create => status.missing_mandatory.push(skill),
            // An installed mandatory skill was compared by the sync plan already.
            Err(error) if !known(status) => status.failed.push(problem(skill, &error)),
            Ok(_) | Err(_) => {}
        }
    }
    for row in installed(report, Some(&workspace.vault)).rows {
        if row.in_vault == Some(false)
            && let Some(status) = at(&position, &mut projects, &row.target.project)
        {
            status.not_in_vault.push(row.target.skill);
        }
    }
    let targets = &workspace.settings.config().targets;
    for bridge in plan_bridges(targets, &report.skills_dirs) {
        let Some(status) = at(&position, &mut projects, &bridge.project) else {
            continue;
        };
        if bridge.state == BridgeState::Missing {
            status.missing_bridges.push(bridge.name.clone());
        } else if let Some((message, hint)) = bridge.problem() {
            status.failed.push(Problem {
                skill: format!("bridge {}", bridge.name),
                message,
                hint: Some(hint),
            });
        }
    }
    for status in &mut projects {
        status.missing_bridges.sort();
        status.up_to_date.sort();
        status.outdated.sort_by(|a, b| a.skill.cmp(&b.skill));
        status.missing_mandatory.sort();
        status.not_in_vault.sort();
        status.failed.sort_by(|a, b| a.skill.cmp(&b.skill));
    }
    Ok(StatusReport { projects })
}

fn at<'a>(
    position: &HashMap<PathBuf, usize>,
    projects: &'a mut [ProjectStatus],
    project: &Path,
) -> Option<&'a mut ProjectStatus> {
    position.get(project).and_then(|i| projects.get_mut(*i))
}

fn outdated(skill: String, plan: &SkillPlan) -> Outdated {
    let count =
        |wanted: fn(ChangeKind) -> bool| plan.changes.iter().filter(|c| wanted(c.kind)).count();
    // A folder that goes is counted by the files below it, not once more for itself.
    let removed = plan
        .removed
        .iter()
        .filter(|path| {
            !plan
                .removed
                .iter()
                .any(|other| other != *path && other.starts_with(path))
        })
        .count();
    Outdated {
        skill,
        added: count(|kind| kind == ChangeKind::Added),
        changed: count(|kind| kind != ChangeKind::Added),
        removed,
    }
}

fn problem(skill: String, error: &PlanError) -> Problem {
    Problem {
        skill,
        message: error.to_string(),
        hint: error.hint(),
    }
}
