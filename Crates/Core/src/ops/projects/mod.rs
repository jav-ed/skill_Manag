//! Operations that write skills into projects: add and init, sync, push, delete, and the bridge links.

pub(in crate::ops) mod bridge;
pub(in crate::ops) mod delete;
pub(in crate::ops) mod project;
pub(in crate::ops) mod push;
pub(in crate::ops) mod scaffold;
pub(in crate::ops) mod sync;

#[cfg(test)]
mod bridge_tests;
#[cfg(test)]
mod project_tests;
#[cfg(test)]
mod safety_tests;
#[cfg(test)]
mod scaffold_tests;
