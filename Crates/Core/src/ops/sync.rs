//! Sync: refresh the skills a project already has. The opt-in rule: nothing is ever added.

use super::{Scope, ScopeError, Workspace};
use crate::plan::Plan;
use crate::scan::{ScanReport, sync_targets};

/// Plans every installed skill folder whose name exists in the vault.
pub fn plan(workspace: &Workspace, report: &ScanReport) -> Plan {
    let set = sync_targets(&report.skills_dirs, |name| {
        workspace.vault.skills.contains_key(name)
    });
    Plan::for_targets(&workspace.vault, &workspace.files, set.targets)
}

/// The same, limited to a scope. A skill in the scope that a project does not have stays out: the
/// opt-in rule holds for a named skill too.
pub fn plan_scoped(
    workspace: &Workspace,
    report: &ScanReport,
    scope: &Scope,
) -> Result<Plan, ScopeError> {
    let narrowed = scope.narrow(report)?;
    let set = sync_targets(&narrowed.skills_dirs, |name| {
        scope.keeps(name) && workspace.vault.skills.contains_key(name)
    });
    Ok(Plan::for_targets(
        &workspace.vault,
        &workspace.files,
        set.targets,
    ))
}
