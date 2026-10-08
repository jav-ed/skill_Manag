//! JSON paths. A project folder whose name is not valid UTF-8 must not make `--json` fail after the
//! files were written, so such a path is shown with replacement characters.

use std::path::Path;

use serde::Serializer;

pub(super) fn path<S: Serializer>(path: &Path, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&path.to_string_lossy())
}
