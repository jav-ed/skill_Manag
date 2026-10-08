//! Sync: refresh the skills a project already has. The opt-in rule: nothing is ever added.

use super::Workspace;
use crate::plan::Plan;
use crate::scan::{ScanReport, sync_targets};

/// Plans every installed skill folder whose name exists in the vault.
pub fn plan(workspace: &Workspace, report: &ScanReport) -> Plan {
    let set = sync_targets(&report.skills_dirs, |name| {
        workspace.vault.skills.contains_key(name)
    });
    Plan::for_targets(&workspace.vault, &workspace.files, set.targets)
}
