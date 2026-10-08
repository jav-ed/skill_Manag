//! The operations front ends call: open a workspace, scan, then plan and apply sync, push, list and delete.

mod delete;
mod list;
mod project;
mod push;
mod select;
mod status;
mod sync;
mod workspace;

pub use delete::{
    DeleteError, DeleteReport, Deleted, delete, target_in_project, targets_named, validate_name,
};
pub use list::{Installed, InstalledSet, installed};
pub use project::{NewProject, ProjectError, check_existing, check_new, create_new, plan_install};
pub use push::plan as plan_push;
pub use select::{SelectError, Selection, resolve, resolve_names};
pub use status::{Outdated, Problem, ProjectStatus, StatusReport, status};
pub use sync::plan as plan_sync;
pub use workspace::Workspace;

#[cfg(test)]
mod profile_tests;
#[cfg(test)]
mod project_tests;
#[cfg(test)]
mod safety_tests;
#[cfg(test)]
mod status_tests;
#[cfg(test)]
mod tests;
