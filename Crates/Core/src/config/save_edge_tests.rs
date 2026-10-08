//! `save_config` on files people really have: links, CRLF, a BOM, quoted keys, blank lines in lists.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use super::*;
use crate::testutil::TempTree;

fn names(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}

fn vault(tree: &TempTree) -> PathBuf {
    let vault = tree.path().join("vault");
    std::fs::create_dir_all(&vault).unwrap();
    vault
}

fn set_mandatory(vault: &Path, items: &[&str]) -> Result<(), ConfigError> {
    save_config(
        vault,
        &ConfigUpdate {
            root: None,
            mandatory: Some(&names(items)),
        },
    )
}

fn saved(content: &str, items: &[&str]) -> String {
    let tree = TempTree::new();
    let vault = vault(&tree);
    std::fs::write(vault.join("config.yaml"), content).unwrap();
    set_mandatory(&vault, items).unwrap();
    std::fs::read_to_string(vault.join("config.yaml")).unwrap()
}

#[test]
fn a_symlinked_config_stays_a_link_and_the_target_gets_the_change() {
    let tree = TempTree::new();
    let vault = vault(&tree);
    let dotfiles = tree.path().join("dotfiles");
    std::fs::create_dir_all(&dotfiles).unwrap();
    std::fs::write(dotfiles.join("config.yaml"), "root: /x\nmandatory: [a]\n").unwrap();
    std::fs::set_permissions(
        dotfiles.join("config.yaml"),
        std::fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    std::os::unix::fs::symlink(dotfiles.join("config.yaml"), vault.join("config.yaml")).unwrap();

    set_mandatory(&vault, &["b"]).unwrap();

    let link = std::fs::symlink_metadata(vault.join("config.yaml")).unwrap();
    assert!(link.file_type().is_symlink(), "the link was replaced");
    assert_eq!(
        std::fs::read_to_string(dotfiles.join("config.yaml")).unwrap(),
        "root: /x\nmandatory:\n  - b\n"
    );
    let mode = std::fs::metadata(dotfiles.join("config.yaml"))
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600);
}

#[test]
fn a_dangling_link_is_an_error_and_stays_a_link() {
    let tree = TempTree::new();
    let vault = vault(&tree);
    std::os::unix::fs::symlink(tree.path().join("gone.yaml"), vault.join("config.yaml")).unwrap();

    let err = set_mandatory(&vault, &["b"]).unwrap_err();

    assert!(matches!(err, ConfigError::Io(_)), "{err}");
    let link = std::fs::symlink_metadata(vault.join("config.yaml")).unwrap();
    assert!(link.file_type().is_symlink());
    assert!(!tree.path().join("gone.yaml").exists());
}

#[test]
fn a_blank_line_inside_the_list_does_not_leave_old_items_behind() {
    let text = saved("mandatory:\n  - a\n\n  - b\nexclude_dirs: [x]\n", &["c"]);
    assert_eq!(text, "mandatory:\n  - c\nexclude_dirs: [x]\n");
}

#[test]
fn a_column_zero_comment_inside_the_list_does_not_break_the_save() {
    let text = saved("mandatory:\n  - a\n# note\n  - b\nroot: /r\n", &["c"]);
    assert_eq!(text, "mandatory:\n  - c\nroot: /r\n");
}

#[test]
fn blank_lines_and_comments_after_the_list_belong_to_what_follows() {
    let text = saved(
        "mandatory:\n  - a\n\n# about the next key\nexclude_dirs: [x]\n",
        &["c"],
    );
    assert_eq!(
        text,
        "mandatory:\n  - c\n\n# about the next key\nexclude_dirs: [x]\n"
    );
}

#[test]
fn quoted_keys_are_found_and_replaced() {
    assert_eq!(
        saved("\"mandatory\":\n  - a\nroot: /r\n", &["c"]),
        "mandatory:\n  - c\nroot: /r\n"
    );
    assert_eq!(saved("'mandatory' : [a]\n", &["c"]), "mandatory:\n  - c\n");
}

#[test]
fn a_byte_order_mark_stays_and_the_key_after_it_is_found() {
    let text = saved("\u{feff}mandatory: [a]\nroot: /r\n", &["c"]);
    assert_eq!(text, "\u{feff}mandatory:\n  - c\nroot: /r\n");
}

#[test]
fn crlf_line_endings_are_kept() {
    let text = saved(
        "root: /a\r\nmandatory:\r\n  - b\r\nexclude_dirs:\r\n  - x\r\n",
        &["c", "d"],
    );
    assert_eq!(
        text,
        "root: /a\r\nmandatory:\r\n  - c\r\n  - d\r\nexclude_dirs:\r\n  - x\r\n"
    );
    let appended = saved("root: /a\r\n", &["c"]);
    assert_eq!(appended, "root: /a\r\nmandatory:\r\n  - c\r\n");
}

#[test]
fn a_comment_that_mentions_the_key_is_not_the_key() {
    let text = saved("# mandatory: old idea\nmandatory: [a]\n", &["c"]);
    assert_eq!(text, "# mandatory: old idea\nmandatory:\n  - c\n");
}
