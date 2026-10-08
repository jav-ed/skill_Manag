//! Scope: narrowing sync, push and status to some skills, one project, or both.

use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::Hint;
use crate::scan::{ScanReport, same_folder};

#[derive(Debug, thiserror::Error)]
pub enum ScopeError {
    #[error("{path} is not a project with a skills folder under the scan root")]
    ProjectNotFound { path: PathBuf },
    #[error("skill {skill:?} is not a mandatory skill")]
    NotMandatory { skill: String },
}

impl Hint for ScopeError {
    fn hint(&self) -> Option<String> {
        match self {
            Self::ProjectNotFound { .. } => Some(
                "`skillmirror status --all` lists the projects; `add` installs into one that has no skills folder yet"
                    .to_string(),
            ),
            Self::NotMandatory { .. } => Some(
                "`push` installs the mandatory skills; `add` installs any other skill into one project"
                    .to_string(),
            ),
        }
    }
}

/// Which part of the scan a command works on. The default is everything.
#[derive(Debug, Default, Clone)]
pub struct Scope {
    /// Only these skill names; `None` takes every skill.
    pub skills: Option<BTreeSet<String>>,
    /// Only this project; `None` takes every project.
    pub project: Option<PathBuf>,
}

impl Scope {
    pub fn is_everything(&self) -> bool {
        self.skills.is_none() && self.project.is_none()
    }

    /// Whether a skill name is inside the scope.
    pub fn keeps(&self, skill: &str) -> bool {
        self.skills.as_ref().is_none_or(|set| set.contains(skill))
    }

    /// The scan without the projects outside the scope. A project that the scan did not find is an error,
    /// so a typo never looks like "nothing to do".
    pub fn narrow(&self, report: &ScanReport) -> Result<ScanReport, ScopeError> {
        let Some(project) = &self.project else {
            return Ok(ScanReport {
                skills_dirs: report.skills_dirs.clone(),
                issues: report.issues.clone(),
            });
        };
        let skills_dirs: Vec<_> = report
            .skills_dirs
            .iter()
            .filter(|dir| same_folder(project, &dir.project))
            .cloned()
            .collect();
        if skills_dirs.is_empty() {
            return Err(ScopeError::ProjectNotFound {
                path: project.clone(),
            });
        }
        let issues = report
            .issues
            .iter()
            .filter(|issue| {
                skills_dirs
                    .iter()
                    .any(|d| issue.path.starts_with(&d.project))
            })
            .cloned()
            .collect();
        Ok(ScanReport {
            skills_dirs,
            issues,
        })
    }
}
