//! Writing a plan to disk: stage beside the target, then swap atomically.

mod error;
mod place;
mod run;
mod stage;
mod swap;

pub use error::ApplyError;
pub(crate) use place::{Keep, Replaces, discard, place};
pub use run::{Applied, ApplyOptions, ApplyReport, Failure, Leftover, Outcome, apply};
pub(crate) use swap::{exchange, place_new};

#[cfg(test)]
mod guard_tests;
#[cfg(test)]
mod new_project_tests;
#[cfg(test)]
mod stale_tests;
#[cfg(test)]
mod tests;
