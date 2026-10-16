//! Operations that grow the vault: new skills, adopted skills, the mandatory list and new vaults.

pub(in crate::ops) mod adopt;
pub(in crate::ops) mod author;
mod created;
pub(in crate::ops) mod mandatory;
pub(in crate::ops) mod vault_init;

#[cfg(test)]
mod adopt_tests;
#[cfg(test)]
mod author_tests;
#[cfg(test)]
mod mandatory_tests;
#[cfg(test)]
mod vault_init_tests;
