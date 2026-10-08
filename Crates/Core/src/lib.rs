//! Engine of skillmirror. Synchronous, terminal-free and independent of every front end.
#![cfg_attr(
    test,
    allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)
)]

#[cfg(not(target_os = "linux"))]
compile_error!("skillmirror supports Linux only");

pub mod apply;
pub mod backup;
pub mod brand;
pub mod config;
pub mod error;
pub mod events;
mod git;
pub mod ops;
pub mod plan;
mod runid;
pub mod scan;
pub mod vault;

#[cfg(test)]
mod testutil;

pub use error::{Error, Hint, Result, describe};
