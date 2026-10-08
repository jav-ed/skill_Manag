//! The web interface itself: the static files that `Ui/` builds, held in the binary.

use axum::http::{HeaderValue, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};

include!(concat!(env!("OUT_DIR"), "/ui_files.rs"));

/// The file for an address: the address as it is, or the folder's `index.html`. Only names in the table
/// are served, so no address can reach anything else.
fn find(path: &str) -> Option<&'static (&'static str, &'static str, &'static [u8])> {
    let name = path.trim_matches('/');
    let direct = if name.is_empty() {
        "index.html".to_string()
    } else {
        name.to_string()
    };
    let folder = format!("{name}/index.html");
    FILES
        .iter()
        .find(|(known, _, _)| *known == direct)
        .or_else(|| FILES.iter().find(|(known, _, _)| *known == folder))
}

/// Everything that is not an API route or the report.
pub(crate) async fn serve(uri: Uri) -> Response {
    // A file name has no `..`, no empty part and no backslash; anything else is not ours.
    let path = uri.path();
    if path.split('/').any(|part| part == ".." || part == ".") || path.contains('\\') {
        return StatusCode::NOT_FOUND.into_response();
    }
    match find(path) {
        Some((_, kind, bytes)) => (
            [(header::CONTENT_TYPE, HeaderValue::from_static(kind))],
            *bytes,
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            "Not found.\n",
        )
            .into_response(),
    }
}

#[cfg(test)]
pub(crate) fn all() -> &'static [(&'static str, &'static str, &'static [u8])] {
    FILES
}
