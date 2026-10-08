//! A name that is unique to one run of the tool, for staging and trash folders.

use std::time::{SystemTime, UNIX_EPOCH};

/// Process id plus start time, so two runs, even of one process, never share a temporary folder name.
pub(crate) fn run_id() -> std::io::Result<String> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(std::io::Error::other)?
        .as_nanos();
    Ok(format!("{}-{nanos:x}", std::process::id()))
}
