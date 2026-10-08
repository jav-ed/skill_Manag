//! Push: install the mandatory skills into every project that has a skills directory, bypassing the opt-in rule.

use super::Workspace;
use crate::plan::{Plan, PlanError};
use crate::scan::{ScanReport, push_targets};

/// Plans every mandatory skill for every project. A mandatory name the vault does not have is a hard error.
pub fn plan(workspace: &Workspace, report: &ScanReport) -> Result<Plan, PlanError> {
    let mandatory = workspace.settings.mandatory();
    if let Some(missing) = mandatory
        .iter()
        .find(|name| !workspace.vault.skills.contains_key(*name))
    {
        return Err(PlanError::MandatoryNotInVault {
            skill: missing.clone(),
        });
    }
    let targets = push_targets(&report.skills_dirs, mandatory);
    Ok(Plan::for_targets(
        &workspace.vault,
        &workspace.files,
        targets,
    ))
}
