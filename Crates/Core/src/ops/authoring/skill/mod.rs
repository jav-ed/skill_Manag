//! Skills in the vault: new ones, adopted ones, and ones replaced by a project's copy (promote).

pub(in crate::ops) mod adopt;
pub(in crate::ops) mod author;
mod created;
pub(in crate::ops) mod promote;

#[cfg(test)]
mod adopt_tests;
#[cfg(test)]
mod author_tests;
#[cfg(test)]
mod promote_tests;
