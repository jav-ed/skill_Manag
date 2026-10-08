//! Errors of the terminal UI.

use skillmirror_core::Hint;

#[derive(Debug, thiserror::Error)]
pub enum TuiError {
    /// The terminal could not be set up or written to.
    #[error("terminal error: {0}")]
    Terminal(#[from] std::io::Error),
    /// stdin or stdout is not a terminal, so there is nothing to draw on.
    #[error("the interactive view needs a terminal")]
    NoTerminal,
    /// Reading the keyboard failed, so no key could ever arrive again.
    #[error("cannot read the keyboard: {0}")]
    Input(String),
}

impl Hint for TuiError {
    fn hint(&self) -> Option<String> {
        match self {
            Self::Terminal(_) | Self::Input(_) => None,
            Self::NoTerminal => Some(
                "run `skillmirror --help` to see the commands that work without a terminal"
                    .to_string(),
            ),
        }
    }
}
