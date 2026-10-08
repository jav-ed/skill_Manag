//! Error plumbing shared by the engine: a "what to do next" hint and the top-level error.

use crate::apply::ApplyError;
use crate::backup::BackupError;
use crate::config::ConfigError;
use crate::ops::{
    AuthorError, DeleteError, MandatoryError, ProjectError, ScopeError, SelectError, VaultInitError,
};
use crate::plan::PlanError;
use crate::scan::ScanError;
use crate::vault::VaultError;

/// An optional second line telling the user what to do about an error. Front ends print it under the message.
pub trait Hint {
    fn hint(&self) -> Option<String>;
}

/// Every failure the engine can report. Each variant keeps its module's own error type.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Vault(#[from] VaultError),
    #[error(transparent)]
    Scan(#[from] ScanError),
    #[error(transparent)]
    Plan(#[from] PlanError),
    #[error(transparent)]
    Apply(#[from] ApplyError),
    #[error(transparent)]
    Backup(#[from] BackupError),
    #[error(transparent)]
    Delete(#[from] DeleteError),
    #[error(transparent)]
    Project(#[from] ProjectError),
    #[error(transparent)]
    Select(#[from] SelectError),
    #[error(transparent)]
    Scope(#[from] ScopeError),
    #[error(transparent)]
    Author(#[from] AuthorError),
    #[error(transparent)]
    VaultInit(#[from] VaultInitError),
    #[error(transparent)]
    Mandatory(#[from] MandatoryError),
}

impl Hint for Error {
    fn hint(&self) -> Option<String> {
        match self {
            Self::Config(e) => e.hint(),
            Self::Vault(e) => e.hint(),
            Self::Scan(e) => e.hint(),
            Self::Plan(e) => e.hint(),
            Self::Apply(e) => e.hint(),
            Self::Backup(e) => e.hint(),
            Self::Delete(_) => None,
            Self::Project(e) => e.hint(),
            Self::Select(e) => e.hint(),
            Self::Scope(e) => e.hint(),
            Self::Author(e) => e.hint(),
            Self::VaultInit(e) => e.hint(),
            Self::Mandatory(e) => e.hint(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
