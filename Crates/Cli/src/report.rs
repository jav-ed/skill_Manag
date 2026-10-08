//! Errors as the user sees them: one message line and an optional hint line.

use skillmirror_core::Hint;

use crate::exit::Exit;
use crate::output;

#[derive(Debug, thiserror::Error)]
pub(crate) enum CliError {
    #[error(transparent)]
    Core(#[from] skillmirror_core::Error),
    #[error("{0}")]
    Usage(String, Option<String>),
    /// The tool declined to do something that would have been legal but unsafe, such as replacing a file
    /// that is not its own.
    #[error("{0}")]
    Refused(String, Option<String>),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Tui(#[from] skillmirror_tui::TuiError),
}

impl CliError {
    pub(crate) fn usage(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::Usage(message.into(), Some(hint.into()))
    }

    pub(crate) fn refused(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::Refused(message.into(), Some(hint.into()))
    }

    pub(crate) fn exit(&self) -> Exit {
        match self {
            Self::Usage(..)
            | Self::Tui(skillmirror_tui::TuiError::NoTerminal)
            | Self::Core(
                skillmirror_core::Error::Delete(skillmirror_core::ops::DeleteError::InvalidName {
                    ..
                })
                | skillmirror_core::Error::Backup(skillmirror_core::backup::BackupError::NoSuchRun {
                    ..
                }),
            ) => Exit::Usage,
            _ => Exit::HardError,
        }
    }

    fn hint(&self) -> Option<String> {
        match self {
            Self::Core(e) => e.hint(),
            Self::Usage(_, hint) | Self::Refused(_, hint) => hint.clone(),
            Self::Tui(e) => e.hint(),
            _ => None,
        }
    }
}

/// Errors of the engine's modules reach the user through the engine's own error type.
macro_rules! from_core_error {
    ($($error:ty),+ $(,)?) => {
        $(impl From<$error> for CliError {
            fn from(error: $error) -> Self {
                Self::Core(error.into())
            }
        })+
    };
}

from_core_error!(
    skillmirror_core::backup::BackupError,
    skillmirror_core::config::ConfigError,
    skillmirror_core::ops::DeleteError,
    skillmirror_core::ops::ProjectError,
    skillmirror_core::ops::SelectError,
    skillmirror_core::apply::ApplyError,
    skillmirror_core::plan::PlanError,
    skillmirror_core::scan::ScanError,
    skillmirror_core::vault::VaultError,
);

pub(crate) fn print_error(error: &CliError) {
    output::error_line(&error.to_string());
    if let Some(hint) = error.hint() {
        output::hint_line(&hint);
    }
}
