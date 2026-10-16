//! Operations that grow the vault. `skill` works on single skills (new, adopted), `vault` on the vault
//! itself (the mandatory list, new vaults).

pub(in crate::ops) mod skill;
pub(in crate::ops) mod vault;
