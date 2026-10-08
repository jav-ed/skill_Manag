//! The views of skillmirror that open in a browser: the static report, and (with the `server` feature)
//! the local server that shows the same data live and can run sync, push and undo.

mod report;
#[cfg(feature = "server")]
mod server;

pub use report::{MARKER, render_report};
#[cfg(feature = "server")]
pub use server::{Config, ServeConfig, ServeError, serve};
