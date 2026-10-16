//! Operations that only read: how projects stand against the vault, what a sync would change, what is
//! installed, everything about one skill, the HTML report, and the doctor.

pub(in crate::ops) mod diff;
pub(in crate::ops) mod doctor;
pub(in crate::ops) mod info;
pub(in crate::ops) mod list;
pub(in crate::ops) mod report;
pub(in crate::ops) mod status;

#[cfg(test)]
mod diff_tests;
#[cfg(test)]
mod info_tests;
#[cfg(test)]
mod report_tests;
// The world these tests build is the one the tests of history and scope start from too.
#[cfg(test)]
pub(in crate::ops) mod status_tests;
