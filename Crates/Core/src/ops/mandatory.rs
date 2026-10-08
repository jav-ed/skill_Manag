//! Editing the `mandatory` list of the vault config: which skills `push` installs everywhere.

use crate::Hint;
use crate::vault::Vault;

use super::{SelectError, resolve_names};

#[derive(Debug, thiserror::Error)]
pub enum MandatoryError {
    #[error(transparent)]
    Select(#[from] SelectError),
    #[error("{name:?} is not in the mandatory list")]
    NotListed { name: String },
}

impl Hint for MandatoryError {
    fn hint(&self) -> Option<String> {
        match self {
            Self::Select(e) => e.hint(),
            Self::NotListed { .. } => {
                Some("`skillmirror mandatory list` shows the list".to_string())
            }
        }
    }
}

/// What to do with the list.
#[derive(Debug, Clone, Copy)]
pub enum Change<'a> {
    Add(&'a [String]),
    Remove(&'a [String]),
}

/// The list after the change. Adding keeps the order and appends names that are not there yet; a name
/// that is already listed is not an error, so the command can be run twice. Every added name must be a
/// skill of the vault. Removing needs no vault: a list with a name the vault lost must stay fixable.
pub fn change_mandatory(
    vault: &Vault,
    current: &[String],
    change: Change<'_>,
) -> Result<Vec<String>, MandatoryError> {
    match change {
        Change::Add(names) => {
            resolve_names(vault, names)?;
            let mut list = current.to_vec();
            for name in names {
                if !list.contains(name) {
                    list.push(name.clone());
                }
            }
            Ok(list)
        }
        Change::Remove(names) => {
            if let Some(unlisted) = names.iter().find(|n| !current.contains(n)) {
                return Err(MandatoryError::NotListed {
                    name: unlisted.clone(),
                });
            }
            Ok(current
                .iter()
                .filter(|n| !names.contains(n))
                .cloned()
                .collect())
        }
    }
}
