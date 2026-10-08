//! The views of skillmirror that open in a browser. Today that is the static report; a local server
//! would join it here later.

mod report;

pub use report::{MARKER, render_report};
