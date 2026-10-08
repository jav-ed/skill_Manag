//! Configuration edge cases: odd paths and empty values.

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use super::*;
use crate::testutil::TempTree;

#[test]
fn a_vault_path_that_is_not_utf8_survives_the_pointer_file() {
    let tree = TempTree::new();
    let dirs = Dirs::under(tree.path());
    let odd = Path::new(OsStr::from_bytes(b"/tmp/v\xffault"));
    write_pointer(&dirs, odd).unwrap();
    assert_eq!(read_pointer(&dirs).unwrap().unwrap(), odd);
}

#[test]
fn an_empty_root_or_exclude_path_names_the_key() {
    let path = Path::new("config.yaml");
    for (text, needle) in [
        ("root: ''\n", "root"),
        ("exclude_paths: ['']\n", "exclude_paths"),
    ] {
        let err = VaultConfig::parse(text, path).unwrap_err();
        assert!(err.to_string().contains(needle), "{err}");
    }
}

#[test]
fn retiring_the_old_pointer_keeps_a_config_directory_that_holds_other_files() {
    let tree = TempTree::new();
    let dirs = Dirs::under(tree.path());
    std::fs::create_dir_all(dirs.legacy_pointer_file().parent().unwrap()).unwrap();
    std::fs::write(dirs.legacy_pointer_file(), "/my/vault\n").unwrap();
    std::fs::write(
        dirs.legacy_pointer_file().with_file_name("other.yaml"),
        "keep",
    )
    .unwrap();
    migrate(&dirs, true).unwrap();
    assert!(!dirs.legacy_pointer_file().exists());
    assert!(
        dirs.legacy_pointer_file()
            .with_file_name("other.yaml")
            .exists()
    );
}

#[test]
fn a_vault_path_that_is_a_file_is_named_as_such() {
    let tree = TempTree::new();
    let file = tree.write("a-file", "x");
    let flags = Flags {
        vault: Some(file.clone()),
        root: None,
    };
    let dirs = Dirs::under(&tree.path().join("home"));
    let err = Settings::load(&flags, &EnvOverrides::default(), &dirs).unwrap_err();
    assert!(
        matches!(err, ConfigError::VaultNotADirectory { .. }),
        "{err}"
    );
    assert!(err.to_string().contains("not a directory"));
}
