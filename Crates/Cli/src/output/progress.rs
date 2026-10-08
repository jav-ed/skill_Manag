//! The live line on stderr while the root is scanned.

use std::time::Duration;

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use skillmirror_core::events::Event;

/// A spinner with the number of folders looked at and projects found. It draws only where a person can see
/// it: indicatif hides a bar when stderr is not a terminal or `TERM` is `dumb`, and a command that prints
/// JSON asks for no line at all. A hidden bar prints nothing.
pub(crate) struct ScanLine {
    bar: ProgressBar,
}

impl ScanLine {
    pub(crate) fn start(quiet: bool) -> Self {
        let target = if quiet {
            ProgressDrawTarget::hidden()
        } else {
            ProgressDrawTarget::stderr_with_hz(10)
        };
        let bar = ProgressBar::with_draw_target(None, target);
        if !bar.is_hidden() {
            bar.set_style(
                ProgressStyle::with_template("{spinner} scanning {msg}")
                    .unwrap_or_else(|_| ProgressStyle::default_spinner()),
            );
            // A slow scan shows the spinner after a moment even before the first count has come.
            bar.enable_steady_tick(Duration::from_millis(150));
        }
        Self { bar }
    }

    /// Listens to the events of a scan.
    pub(crate) fn hear(&self, event: &Event) {
        if let Event::ScanProgress {
            directories,
            projects,
        } = event
        {
            self.bar.set_message(format!(
                "{directories} folders, {projects} with skills so far"
            ));
        }
    }

    /// Takes the line away, so what is printed next starts on a clean row.
    pub(crate) fn clear(self) {
        self.bar.finish_and_clear();
    }
}
