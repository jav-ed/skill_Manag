use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

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

#[test]
fn a_missing_file_is_created_with_both_keys() {
    let tree = TempTree::new();
    let vault = vault(&tree);
    let mandatory = names(&["coding", "tmux"]);
    save_config(
        &vault,
        &ConfigUpdate {
            root: Some(std::path::Path::new("/work")),
            mandatory: Some(&mandatory),
        },
    )
    .unwrap();
    let text = std::fs::read_to_string(vault.join("config.yaml")).unwrap();
    assert_eq!(text, "root: \"/work\"\nmandatory:\n  - coding\n  - tmux\n");
    let mode = std::fs::metadata(vault.join("config.yaml"))
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o644);
    let loaded = VaultConfig::load(&vault).unwrap();
    assert_eq!(loaded.mandatory, mandatory);
}

#[test]
fn comments_other_keys_and_their_order_survive_a_rewrite() {
    let tree = TempTree::new();
    let vault = vault(&tree);
    std::fs::write(
        vault.join("config.yaml"),
        "# my vault\nroot: /old\n\n# always installed\nmandatory:\n  - coding\n  # tmux is optional now\n  - tmux\nexclude_dirs: [node_modules]\n",
    )
    .unwrap();
    save_config(
        &vault,
        &ConfigUpdate {
            root: Some(std::path::Path::new("/new")),
            mandatory: Some(&names(&["astro"])),
        },
    )
    .unwrap();
    let text = std::fs::read_to_string(vault.join("config.yaml")).unwrap();
    assert_eq!(
        text,
        "# my vault\nroot: \"/new\"\n\n# always installed\nmandatory:\n  - astro\nexclude_dirs: [node_modules]\n"
    );
}

#[test]
fn only_the_given_keys_change_and_a_missing_key_is_appended() {
    let tree = TempTree::new();
    let vault = vault(&tree);
    std::fs::write(vault.join("config.yaml"), "exclude_dirs: [a]").unwrap();
    save_config(
        &vault,
        &ConfigUpdate {
            root: None,
            mandatory: Some(&[]),
        },
    )
    .unwrap();
    let text = std::fs::read_to_string(vault.join("config.yaml")).unwrap();
    assert_eq!(text, "exclude_dirs: [a]\nmandatory: []\n");
}

#[test]
fn names_that_yaml_would_misread_are_quoted() {
    let tree = TempTree::new();
    let vault = vault(&tree);
    let tricky = names(&["y", "123", "a: b", "plain-name_1.0", "say \"hi\""]);
    save_config(
        &vault,
        &ConfigUpdate {
            root: None,
            mandatory: Some(&tricky),
        },
    )
    .unwrap();
    assert_eq!(VaultConfig::load(&vault).unwrap().mandatory, tricky);
}

#[test]
fn a_malformed_file_is_an_error_and_stays_as_it_was() {
    let tree = TempTree::new();
    let vault = vault(&tree);
    std::fs::write(vault.join("config.yaml"), "mandatory: [a\nroot: {").unwrap();
    let err = save_config(
        &vault,
        &ConfigUpdate {
            root: Some(std::path::Path::new("/w")),
            mandatory: None,
        },
    )
    .unwrap_err();
    assert!(matches!(err, ConfigError::InvalidVaultConfig { .. }));
    assert_eq!(
        std::fs::read_to_string(vault.join("config.yaml")).unwrap(),
        "mandatory: [a\nroot: {"
    );
}

#[test]
fn a_relative_root_is_refused() {
    let tree = TempTree::new();
    let vault = vault(&tree);
    let err = save_config(
        &vault,
        &ConfigUpdate {
            root: Some(std::path::Path::new("relative/dir")),
            mandatory: None,
        },
    )
    .unwrap_err();
    assert!(matches!(err, ConfigError::NotAbsolute { .. }));
    assert!(!vault.join("config.yaml").exists());
}

#[test]
fn the_permissions_of_an_existing_file_are_kept() {
    let tree = TempTree::new();
    let vault = vault(&tree);
    let file = vault.join("config.yaml");
    std::fs::write(&file, "root: /a\n").unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
    save_config(
        &vault,
        &ConfigUpdate {
            root: Some(std::path::Path::new("/b")),
            mandatory: None,
        },
    )
    .unwrap();
    assert_eq!(
        std::fs::metadata(&file).unwrap().permissions().mode() & 0o777,
        0o600
    );
}
