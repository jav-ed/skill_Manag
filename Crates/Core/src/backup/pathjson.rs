//! A path inside a JSON note: plain text when it is valid UTF-8 (the usual case, readable by a person),
//! its raw bytes otherwise, so no project folder is ever too oddly named to be backed up.

use std::ffi::OsString;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize)]
struct Raw {
    bytes: Vec<u8>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Stored {
    Text(String),
    Raw(Raw),
}

pub(super) fn serialize<S: Serializer>(path: &Path, serializer: S) -> Result<S::Ok, S::Error> {
    match path.to_str() {
        Some(text) => serializer.serialize_str(text),
        None => Raw {
            bytes: path.as_os_str().as_bytes().to_vec(),
        }
        .serialize(serializer),
    }
}

pub(super) fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<PathBuf, D::Error> {
    Ok(match Stored::deserialize(deserializer)? {
        Stored::Text(text) => PathBuf::from(text),
        Stored::Raw(raw) => PathBuf::from(OsString::from_vec(raw.bytes)),
    })
}
