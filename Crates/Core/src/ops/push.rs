//! Push: install the mandatory skills into every project that has a skills directory, bypassing the opt-in rule.

use super::{Scope, Workspace};
use crate::plan::{Plan, PlanError};
use crate::scan::{ScanReport, push_targets};

/// Plans every mandatory skill for every project. A mandatory name the vault does not have is a hard error.
pub fn plan(workspace: &Workspace, report: &ScanReport) -> Result<Plan, PlanError> {
    plan_for(workspace, report, workspace.settings.mandatory())
}

/// The same, limited to a scope: only the named mandatory skills, only one project. A named skill that is
/// not mandatory is an error, because push would otherwise install anything anywhere.
pub fn plan_scoped(
    workspace: &Workspace,
    report: &ScanReport,
    scope: &Scope,
) -> Result<Plan, crate::Error> {
    let mandatory = workspace.settings.mandatory();
    if let Some(skills) = &scope.skills
        && let Some(other) = skills.iter().find(|s| !mandatory.contains(s))
    {
        return Err(super::ScopeError::NotMandatory {
            skill: other.clone(),
        }
        .into());
    }
    let narrowed = scope.narrow(report)?;
    let wanted: Vec<String> = mandatory
        .iter()
        .filter(|name| scope.keeps(name))
        .cloned()
        .collect();
    Ok(plan_for(workspace, &narrowed, &wanted)?)
}

fn plan_for(
    workspace: &Workspace,
    report: &ScanReport,
    names: &[String],
) -> Result<Plan, PlanError> {
    if let Some(missing) = names
        .iter()
        .find(|name| !workspace.vault.skills.contains_key(*name))
    {
        return Err(PlanError::MandatoryNotInVault {
            skill: missing.clone(),
        });
    }
    let targets = push_targets(&report.skills_dirs, names);
    Ok(Plan::for_targets(
        &workspace.vault,
        &workspace.files,
        targets,
    ))
}
