//! Info: everything known about one skill: what the vault holds, which profiles name it, and what it is in
//! each project. Reads, never writes.

use std::path::PathBuf;

use super::{Outdated, Problem, Scope, SelectError, Selection, Workspace, resolve, status_scoped};
use crate::scan::ScanReport;
use crate::vault::{Skill, read_header};

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
    /// The files git tracks in the skill folder, relative to it, sorted. These are the files that are copied.
    pub files: Vec<String>,
    /// Files in the skill folder that git does not track and does not ignore, sorted. They are never copied.
    pub untracked: Vec<String>,
}

/// Reads the header and the file lists of one skill.
pub fn skill_info(workspace: &Workspace, skill: &Skill) -> SkillInfo {
    let (description, header_problem) = match read_header(skill) {
        Ok(header) => (header.description.filter(|d| !d.trim().is_empty()), None),
        Err(problem) => (None, Some(problem)),
    };
    let git = workspace.files.get(&skill.name);
    let names = |paths: Vec<&std::path::Path>| {
        let mut out: Vec<String> = paths.iter().map(|p| p.display().to_string()).collect();
        out.sort();
        out
    };
    SkillInfo {
        name: skill.name.clone(),
        group: skill.group.clone(),
        description,
        header_problem,
        mandatory: workspace.settings.mandatory().contains(&skill.name),
        files: names(
            git.map(|f| f.tracked.iter().map(|t| t.rel.as_path()).collect())
                .unwrap_or_default(),
        ),
        untracked: names(
            git.map(|f| f.untracked.iter().map(PathBuf::as_path).collect())
                .unwrap_or_default(),
        ),
    }
}

/// What a skill is in one project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillState {
    Current,
    Outdated(Outdated),
    /// Mandatory and not installed.
    MissingMandatory,
    /// Installed, but the vault has no such skill.
    NotInVault,
    Problem(Problem),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectState {
    pub project: PathBuf,
    pub state: SkillState,
}

#[derive(Debug)]
pub struct SkillDetail {
    pub name: String,
    /// `None` when the vault has no such skill and only installed folders carry the name.
    pub info: Option<SkillInfo>,
    /// The profiles of the vault config that select the skill, sorted.
    pub profiles: Vec<String>,
    /// The projects where the skill is installed or should be (mandatory), in scan order.
    pub projects: Vec<ProjectState>,
    /// Every project with a skills folder, so that "in 2 of 7" can be said.
    pub total_projects: usize,
}

impl SkillDetail {
    /// Whether `sync` or `push` would change the skill somewhere.
    pub fn drifts(&self) -> bool {
        self.projects.iter().any(|p| {
            matches!(
                p.state,
                SkillState::Outdated(_) | SkillState::MissingMandatory
            )
        })
    }

    pub fn failed(&self) -> usize {
        self.projects
            .iter()
            .filter(|p| matches!(p.state, SkillState::Problem(_)))
            .count()
    }
}

/// Everything about one skill. A name that neither the vault nor any project has is an error.
pub fn skill_detail(
    workspace: &Workspace,
    report: &ScanReport,
    name: &str,
) -> Result<SkillDetail, crate::Error> {
    let scope = Scope {
        skills: Some(std::iter::once(name.to_string()).collect()),
        project: None,
    };
    let status = status_scoped(workspace, report, &scope)?;
    let mut projects = Vec::new();
    for project in status.projects {
        let at = |state| ProjectState {
            project: project.project.clone(),
            state,
        };
        projects.extend(project.up_to_date.iter().map(|_| at(SkillState::Current)));
        projects.extend(
            project
                .outdated
                .iter()
                .map(|o| at(SkillState::Outdated(o.clone()))),
        );
        projects.extend(
            project
                .missing_mandatory
                .iter()
                .map(|_| at(SkillState::MissingMandatory)),
        );
        projects.extend(
            project
                .not_in_vault
                .iter()
                .map(|_| at(SkillState::NotInVault)),
        );
        projects.extend(
            project
                .failed
                .iter()
                .map(|p| at(SkillState::Problem(p.clone()))),
        );
    }
    let info = workspace
        .vault
        .skills
        .get(name)
        .map(|skill| skill_info(workspace, skill));
    if info.is_none() && projects.is_empty() {
        return Err(SelectError::UnknownSkill {
            name: name.to_string(),
        }
        .into());
    }
    Ok(SkillDetail {
        name: name.to_string(),
        info,
        profiles: profiles_naming(workspace, name),
        projects,
        total_projects: report.skills_dirs.len(),
    })
}

fn profiles_naming(workspace: &Workspace, skill: &str) -> Vec<String> {
    let config = workspace.settings.config();
    config
        .profiles
        .keys()
        .filter(|profile| {
            let selection = Selection {
                profiles: vec![(*profile).clone()],
                ..Selection::default()
            };
            resolve(&workspace.vault, config, &selection)
                .is_ok_and(|members| members.contains(skill))
        })
        .cloned()
        .collect()
}
