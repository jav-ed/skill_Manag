//! Everything the HTML report shows, gathered once: the vault's skills, the projects, and what each skill
//! is in each project. Reads, never writes.

use std::collections::BTreeMap;
use std::path::PathBuf;

use super::{DiffFilter, FileDiff, Workspace, diff, status};
use crate::plan::PlanError;
use crate::scan::{ScanIssue, ScanReport};
use crate::vault::read_header;

/// A skill of the vault.
#[derive(Debug, Clone)]
pub struct SkillInfo {
    pub name: String,
    /// The folders between the vault and the skill, outermost first.
    pub group: Vec<String>,
    /// From the header of `SKILL.md`; `None` when there is none or it cannot be read.
    pub description: Option<String>,
    /// Why the header cannot be read.
    pub header_problem: Option<String>,
    pub mandatory: bool,
    /// The files git tracks in the skill folder, relative to it, sorted.
    pub files: Vec<String>,
}

/// What a skill is in one project.
#[derive(Debug)]
pub enum Cell {
    /// Installed and equal to the vault.
    Current,
    /// Installed and different; the files that differ, as diffs.
    Outdated(Vec<FileDiff>),
    /// Mandatory and not installed.
    MissingMandatory,
    /// Installed, but the vault has no such skill.
    NotInVault,
    /// Could not be compared.
    Problem {
        message: String,
        hint: Option<String>,
    },
}

#[derive(Debug)]
pub struct ReportData {
    pub vault: PathBuf,
    pub root: PathBuf,
    /// The vault's skills: top-level ones first, then by group, each sorted by name.
    pub skills: Vec<SkillInfo>,
    /// Installed folder names the vault has no skill for, sorted.
    pub extra: Vec<String>,
    /// Projects with a skills directory, in scan order.
    pub projects: Vec<PathBuf>,
    /// By skill name and project index. A skill a project does not have and does not need has no cell.
    pub cells: BTreeMap<(String, usize), Cell>,
    /// Things wrong with a project as a whole, such as a link that is in the way, by project index.
    pub project_problems: Vec<(usize, String)>,
    /// Links the vault config asks for that are not made yet, by project index.
    pub missing_bridges: Vec<(usize, String)>,
    /// What the scan could not read.
    pub issues: Vec<ScanIssue>,
}

/// Compares every project with the vault and reads the skills' headers. A mandatory name the vault does
/// not have is a hard error, as it is for `push`.
pub fn report_data(workspace: &Workspace, scan: &ScanReport) -> Result<ReportData, PlanError> {
    let compared = status(workspace, scan)?;
    let mut diffs: BTreeMap<(String, PathBuf), Vec<FileDiff>> = BTreeMap::new();
    for skill in diff(workspace, scan, &DiffFilter::default()).skills {
        diffs.insert((skill.target.skill, skill.target.project), skill.files);
    }
    let projects: Vec<PathBuf> = compared
        .projects
        .iter()
        .map(|p| p.project.clone())
        .collect();
    let mut cells = BTreeMap::new();
    let mut project_problems = Vec::new();
    let mut missing_bridges = Vec::new();
    let mut extra = std::collections::BTreeSet::new();
    for (index, project) in compared.projects.into_iter().enumerate() {
        for skill in project.up_to_date {
            cells.insert((skill, index), Cell::Current);
        }
        for outdated in project.outdated {
            let files = diffs
                .remove(&(outdated.skill.clone(), project.project.clone()))
                .unwrap_or_default();
            cells.insert((outdated.skill, index), Cell::Outdated(files));
        }
        for skill in project.missing_mandatory {
            cells.insert((skill, index), Cell::MissingMandatory);
        }
        for skill in project.not_in_vault {
            extra.insert(skill.clone());
            cells.insert((skill, index), Cell::NotInVault);
        }
        for name in project.missing_bridges {
            missing_bridges.push((index, name));
        }
        for problem in project.failed {
            if problem.skill.starts_with("bridge ") {
                project_problems.push((index, problem.message));
            } else {
                cells.insert(
                    (problem.skill, index),
                    Cell::Problem {
                        message: problem.message,
                        hint: problem.hint,
                    },
                );
            }
        }
    }
    let mandatory = workspace.settings.mandatory();
    let mut skills: Vec<SkillInfo> = workspace
        .vault
        .skills
        .values()
        .map(|skill| {
            let (description, header_problem) = match read_header(skill) {
                Ok(header) => (header.description.filter(|d| !d.trim().is_empty()), None),
                Err(problem) => (None, Some(problem)),
            };
            let mut files: Vec<String> = workspace
                .files
                .get(&skill.name)
                .map(|f| {
                    f.tracked
                        .iter()
                        .map(|t| t.rel.display().to_string())
                        .collect()
                })
                .unwrap_or_default();
            files.sort();
            SkillInfo {
                name: skill.name.clone(),
                group: skill.group.clone(),
                description,
                header_problem,
                mandatory: mandatory.contains(&skill.name),
                files,
            }
        })
        .collect();
    skills.sort_by(|a, b| a.group.cmp(&b.group).then_with(|| a.name.cmp(&b.name)));
    Ok(ReportData {
        vault: workspace.vault.path.clone(),
        root: workspace
            .settings
            .root()
            .map(|r| r.value.clone())
            .unwrap_or_default(),
        skills,
        extra: extra.into_iter().collect(),
        projects,
        cells,
        project_problems,
        missing_bridges,
        issues: scan.issues.clone(),
    })
}
