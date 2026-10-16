//! Which skills and which projects a command touches: names, groups and profiles, and the scope of a run.

pub(in crate::ops) mod scope;
pub(in crate::ops) mod select;

#[cfg(test)]
mod profile_tests;
#[cfg(test)]
mod scope_tests;
