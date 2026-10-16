//! The operations front ends call: open a workspace, scan, then plan and apply sync, push, list and delete.
//!
//! The code is grouped by what it works on: `authoring` grows the vault, `projects` writes skills into
//! projects, `inspect` only reads, `selection` says which skills and projects a command touches. Front
//! ends use the names re-exported here, not the folders.

mod authoring;
mod history;
mod inspect;
mod projects;
mod run;
mod selection;
mod workspace;

pub use authoring::skill::adopt::{AdoptPlan, adopt, plan_adopt};
pub use authoring::skill::author::{
    AuthorError, Authored, check_new_name, new_skill, new_skill_paths,
};
pub use authoring::vault::mandatory::{
    Change as MandatoryChange, MandatoryError, change_mandatory,
};
pub use authoring::vault::vault_init::{NewVault, VaultInitError, check_new_vault, init_vault};
pub use history::{RunRow, Step, UndoLine, UndoView, do_undo, list_runs, plan_undo};
pub use inspect::diff::{
    DiffFilter, DiffKind, DiffReport, FileDiff, SkillDiff, Skipped, diff, diff_of_plan,
};
pub use inspect::doctor::{Finding, Report as DoctorReport, Severity, doctor};
pub use inspect::info::{
    ProjectState, SkillDetail, SkillInfo, SkillState, profiles_of, skill_detail, skill_info,
};
pub use inspect::list::{Installed, InstalledSet, installed};
pub use inspect::local::{
    Edit, EditKind, Found, LocalChanges, LocalError, LocalSkill, LocalState, Located,
    local_changes, locate,
};
pub use inspect::report::{Cell, ReportData, report_data};
pub use inspect::status::{Outdated, Problem, ProjectStatus, StatusReport, status, status_scoped};
pub use projects::bridge::{
    Bridge, BridgeError, BridgeState, create as create_bridge, plan as plan_bridges,
    plan_project as plan_project_bridges,
};
pub use projects::delete::{
    DeleteError, DeleteReport, Deleted, delete, target_in_project, targets_named, validate_name,
};
pub use projects::project::{
    BRANCH, NewProject, ProjectError, Repo, check_existing, check_new, create_new, plan_install,
    repo_for,
};
pub use projects::push::{plan as plan_push, plan_scoped as plan_push_scoped};
pub use projects::scaffold::{require_scaffold, run_scaffold};
pub use projects::sync::{plan as plan_sync, plan_scoped as plan_sync_scoped};
pub use run::{RunOutcome, run_plan};
pub use selection::scope::{Scope, ScopeError};
pub use selection::select::{SelectError, Selection, resolve, resolve_names};
pub use workspace::Workspace;

#[cfg(test)]
mod history_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod workspace_tests;
