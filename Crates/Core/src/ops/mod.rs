//! The operations front ends call: open a workspace, scan, then plan and apply sync, push, list and delete.

mod adopt;
mod author;
mod bridge;
mod created;
mod delete;
mod diff;
mod doctor;
mod info;
mod list;
mod project;
mod push;
mod report;
mod scope;
mod select;
mod status;
mod sync;
mod vault_init;
mod workspace;

pub use adopt::{AdoptPlan, adopt, plan_adopt};
pub use author::{AuthorError, Authored, check_new_name, new_skill, new_skill_paths};
pub use bridge::{
    Bridge, BridgeError, BridgeState, create as create_bridge, plan as plan_bridges,
    plan_project as plan_project_bridges,
};
pub use delete::{
    DeleteError, DeleteReport, Deleted, delete, target_in_project, targets_named, validate_name,
};
pub use diff::{
    DiffFilter, DiffKind, DiffReport, FileDiff, SkillDiff, Skipped, diff, diff_of_plan,
};
pub use doctor::{Finding, Report as DoctorReport, Severity, doctor};
pub use info::{ProjectState, SkillDetail, SkillInfo, SkillState, skill_detail, skill_info};
pub use list::{Installed, InstalledSet, installed};
pub use project::{NewProject, ProjectError, check_existing, check_new, create_new, plan_install};
pub use push::{plan as plan_push, plan_scoped as plan_push_scoped};
pub use report::{Cell, ReportData, report_data};
pub use scope::{Scope, ScopeError};
pub use select::{SelectError, Selection, resolve, resolve_names};
pub use status::{Outdated, Problem, ProjectStatus, StatusReport, status, status_scoped};
pub use sync::{plan as plan_sync, plan_scoped as plan_sync_scoped};
pub use vault_init::{NewVault, VaultInitError, check_new_vault, init_vault};
pub use workspace::Workspace;

#[cfg(test)]
mod adopt_tests;
#[cfg(test)]
mod author_tests;
#[cfg(test)]
mod bridge_tests;
#[cfg(test)]
mod diff_tests;
#[cfg(test)]
mod doctor_machine_tests;
#[cfg(test)]
mod doctor_tests;
#[cfg(test)]
mod info_tests;
#[cfg(test)]
mod profile_tests;
#[cfg(test)]
mod project_tests;
#[cfg(test)]
mod report_tests;
#[cfg(test)]
mod safety_tests;
#[cfg(test)]
mod scope_tests;
#[cfg(test)]
mod status_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod vault_init_tests;
#[cfg(test)]
mod workspace_tests;
