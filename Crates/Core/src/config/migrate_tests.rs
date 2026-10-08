use std::path::Path;

use super::*;
use crate::testutil::TempTree;

fn legacy(dirs: &Dirs, content: &str) {
    std::fs::create_dir_all(dirs.legacy_pointer_file().parent().unwrap()).unwrap();
    std::fs::write(dirs.legacy_pointer_file(), content).unwrap();
}

#[test]
fn copies_the_pointer_and_keeps_the_old_one_by_default() {
    let tree = TempTree::new();
    let dirs = Dirs::under(tree.path());
    legacy(&dirs, "/my/vault\n");
    let report = migrate(&dirs, false).unwrap();
    assert_eq!(report.vault, Path::new("/my/vault"));
    assert!(!report.already_done && !report.retired);
    assert_eq!(
        read_pointer(&dirs).unwrap().unwrap(),
        Path::new("/my/vault")
    );
    assert!(dirs.legacy_pointer_file().exists());
}

#[test]
fn is_idempotent_and_retire_removes_the_old_pointer() {
    let tree = TempTree::new();
    let dirs = Dirs::under(tree.path());
    legacy(&dirs, "/my/vault\n");
    migrate(&dirs, false).unwrap();
    let again = migrate(&dirs, true).unwrap();
    assert!(again.already_done && again.retired);
    assert!(!dirs.legacy_pointer_file().exists());
    assert!(!dirs.legacy_pointer_file().parent().unwrap().exists());
}

#[test]
fn a_different_new_pointer_is_never_overwritten() {
    let tree = TempTree::new();
    let dirs = Dirs::under(tree.path());
    legacy(&dirs, "/old/vault\n");
    write_pointer(&dirs, Path::new("/new/vault")).unwrap();
    assert!(matches!(
        migrate(&dirs, false),
        Err(ConfigError::MigrateConflict { .. })
    ));
    assert_eq!(
        read_pointer(&dirs).unwrap().unwrap(),
        Path::new("/new/vault")
    );
}

#[test]
fn nothing_to_migrate_and_bad_pointers_are_hard_errors() {
    let tree = TempTree::new();
    let dirs = Dirs::under(tree.path());
    assert!(matches!(
        migrate(&dirs, false),
        Err(ConfigError::NothingToMigrate { .. })
    ));
    legacy(&dirs, "relative/vault\n");
    assert!(matches!(
        migrate(&dirs, false),
        Err(ConfigError::NotAbsolute { .. })
    ));
    legacy(&dirs, "  \n");
    assert!(matches!(
        migrate(&dirs, false),
        Err(ConfigError::PointerEmpty { .. })
    ));
}
