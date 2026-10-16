//! The vault itself: its mandatory list and new vaults.

pub(in crate::ops) mod mandatory;
pub(in crate::ops) mod vault_init;

#[cfg(test)]
mod mandatory_tests;
#[cfg(test)]
mod vault_init_tests;
