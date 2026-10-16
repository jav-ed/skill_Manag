//! The commands that only read: `list`, `skills`, `info`, `status`, `diff`, `changes`, `doctor` and `report`.

pub(in crate::commands) mod changes;
pub(in crate::commands) mod diff;
pub(in crate::commands) mod doctor;
pub(in crate::commands) mod info;
pub(in crate::commands) mod list;
pub(in crate::commands) mod report;
pub(in crate::commands) mod skills;
pub(in crate::commands) mod status;
