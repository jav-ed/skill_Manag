//! Process exit codes, defined once.

use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Exit {
    /// Everything done, or nothing differs.
    Clean,
    /// `--check` found targets that differ from the vault.
    Drift,
    /// The command line was valid but cannot be carried out as given (clap itself exits 2 on a syntax error).
    Usage,
    /// The command could not run: bad configuration, missing vault, unreadable root.
    HardError,
    /// The command ran, but some targets failed.
    Partial,
}

impl Exit {
    pub(crate) fn code(self) -> ExitCode {
        ExitCode::from(match self {
            Self::Clean => 0,
            Self::Drift => 1,
            Self::Usage => 2,
            Self::HardError => 3,
            Self::Partial => 4,
        })
    }
}
