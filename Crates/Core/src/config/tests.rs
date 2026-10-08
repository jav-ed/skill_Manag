use std::ffi::OsString;
use std::path::{Path, PathBuf};

use super::*;
use crate::testutil::TempTree;

fn pairs(items: &[(&str, &str)]) -> Vec<(OsString, OsString)> {
    items
        .iter()
        .map(|(k, v)| (OsString::from(k), OsString::from(v)))
        .collect()
}

fn dirs_and_vault(tree: &TempTree) -> (Dirs, PathBuf) {
    let vault = tree.path().join("vault");
    std::fs::create_dir_all(&vault).unwrap();
    (Dirs::under(&tree.path().join("home")), vault)
}

#[test]
fn env_reads_only_the_two_known_variables() {
    let env = EnvOverrides::from_pairs(pairs(&[
        ("SKILLMIRROR_VAULT", "/v"),
        ("SKILLMIRROR_ROOT", "/r"),
        ("HOME", "/h"),
    ]))
    .unwrap();
    assert_eq!(env.vault, Some(PathBuf::from("/v")));
    assert_eq!(env.root, Some(PathBuf::from("/r")));
}

#[test]
fn empty_env_variable_counts_as_unset() {
    let env = EnvOverrides::from_pairs(pairs(&[("SKILLMIRROR_VAULT", "")])).unwrap();
    assert_eq!(env, EnvOverrides::default());
}

#[test]
fn legacy_env_variable_is_a_hard_error_with_a_replacement_hint() {
    let err = EnvOverrides::from_pairs(pairs(&[
        ("SKILL_MANAG_VAULT", "/v"),
        ("SKILL_MANAG_EXCLUDE_DIRS", "x"),
    ]))
    .unwrap_err();
    let hint = crate::Hint::hint(&err).unwrap();
    assert!(
        hint.contains("SKILL_MANAG_VAULT -> SKILLMIRROR_VAULT"),
        "{hint}"
    );
    assert!(
        hint.contains("SKILL_MANAG_EXCLUDE_DIRS has no replacement"),
        "{hint}"
    );
}

#[test]
fn flag_beats_env_beats_pointer_file() {
    let tree = TempTree::new();
    let (dirs, _) = dirs_and_vault(&tree);
    write_pointer(&dirs, Path::new("/from/pointer")).unwrap();
    let env = EnvOverrides {
        vault: Some("/from/env".into()),
        root: None,
    };

    let s = Settings::load(&Flags::default(), &EnvOverrides::default(), &dirs).unwrap();
    assert_eq!(s.vault().unwrap().source, Source::PointerFile);

    let s = Settings::load(&Flags::default(), &env, &dirs).unwrap();
    assert_eq!(
        (
            s.vault().unwrap().value.as_path(),
            s.vault().unwrap().source
        ),
        (Path::new("/from/env"), Source::Env)
    );

    let flags = Flags {
        vault: Some("/from/flag".into()),
        root: None,
    };
    let s = Settings::load(&flags, &env, &dirs).unwrap();
    assert_eq!(
        (
            s.vault().unwrap().value.as_path(),
            s.vault().unwrap().source
        ),
        (Path::new("/from/flag"), Source::Flag)
    );
}

#[test]
fn root_comes_from_the_vault_config_unless_overridden() {
    let tree = TempTree::new();
    let (dirs, vault) = dirs_and_vault(&tree);
    std::fs::write(
        vault.join("config.yaml"),
        "# comment\nroot: /projects\nmandatory: [coding]\nexclude_dirs: [testdata]\n",
    )
    .unwrap();
    let flags = Flags {
        vault: Some(vault),
        root: None,
    };

    let s = Settings::load(&flags, &EnvOverrides::default(), &dirs).unwrap();
    assert_eq!(s.root().unwrap().value, Path::new("/projects"));
    assert_eq!(s.root().unwrap().source, Source::VaultConfig);
    assert_eq!(s.mandatory(), ["coding"]);
    assert_eq!(s.scan_options().exclude_dirs, ["testdata"]);

    let flags = Flags {
        root: Some("/other".into()),
        ..flags
    };
    let s = Settings::load(&flags, &EnvOverrides::default(), &dirs).unwrap();
    assert_eq!(s.root().unwrap().source, Source::Flag);
}

#[test]
fn missing_vault_config_is_empty_but_malformed_is_a_hard_error() {
    let tree = TempTree::new();
    let (dirs, vault) = dirs_and_vault(&tree);
    let flags = Flags {
        vault: Some(vault.clone()),
        root: None,
    };

    let s = Settings::load(&flags, &EnvOverrides::default(), &dirs).unwrap();
    assert!(s.root().is_err());

    for bad in [
        "root: [unclosed",
        "rot: /typo\n",
        "mandatory: 5\n",
        "mandatory: [a, a]\n",
        "exclude_dirs: [a/b]\n",
    ] {
        std::fs::write(vault.join("config.yaml"), bad).unwrap();
        let err = Settings::load(&flags, &EnvOverrides::default(), &dirs).unwrap_err();
        assert!(
            matches!(err, ConfigError::InvalidVaultConfig { .. }),
            "{bad}: {err}"
        );
    }
}

#[test]
fn comment_only_config_is_empty() {
    let tree = TempTree::new();
    let (dirs, vault) = dirs_and_vault(&tree);
    std::fs::write(vault.join("config.yaml"), "# nothing here\n").unwrap();
    let flags = Flags {
        vault: Some(vault),
        root: None,
    };
    assert!(Settings::load(&flags, &EnvOverrides::default(), &dirs).is_ok());
}

#[test]
fn old_pointer_without_new_one_asks_for_migrate() {
    let tree = TempTree::new();
    let (dirs, _) = dirs_and_vault(&tree);
    std::fs::create_dir_all(dirs.legacy_pointer_file().parent().unwrap()).unwrap();
    std::fs::write(dirs.legacy_pointer_file(), "/old/vault\n").unwrap();
    let err = Settings::load(&Flags::default(), &EnvOverrides::default(), &dirs).unwrap_err();
    assert!(matches!(err, ConfigError::LegacyConfig { .. }));
    assert!(crate::Hint::hint(&err).unwrap().contains("migrate"));
}

#[test]
fn pointer_file_must_be_absolute_and_non_empty() {
    let tree = TempTree::new();
    let (dirs, _) = dirs_and_vault(&tree);
    std::fs::create_dir_all(dirs.config()).unwrap();
    std::fs::write(dirs.pointer_file(), "relative/vault\n").unwrap();
    assert!(matches!(
        read_pointer(&dirs),
        Err(ConfigError::NotAbsolute { .. })
    ));
    std::fs::write(dirs.pointer_file(), "  \n").unwrap();
    assert!(matches!(
        read_pointer(&dirs),
        Err(ConfigError::PointerEmpty { .. })
    ));
    assert!(write_pointer(&dirs, Path::new("rel")).is_err());
}

#[test]
fn pointer_round_trips() {
    let tree = TempTree::new();
    let (dirs, _) = dirs_and_vault(&tree);
    assert_eq!(read_pointer(&dirs).unwrap(), None);
    write_pointer(&dirs, Path::new("/some/vault")).unwrap();
    assert_eq!(
        read_pointer(&dirs).unwrap(),
        Some(PathBuf::from("/some/vault"))
    );
}

#[test]
fn profiles_parse_and_are_checked_for_unknown_parents_and_loops() {
    let ok = "profiles:\n  base:\n    skills: [coding]\n  web:\n    description: A site\n    extends: [base]\n    groups: [web]\n    exclude: [vite]\n";
    let config = VaultConfig::parse(ok, Path::new("config.yaml")).unwrap();
    assert_eq!(config.profiles["web"].extends, ["base"]);
    assert_eq!(
        config.profiles["web"].description.as_deref(),
        Some("A site")
    );

    for bad in [
        "profiles:\n  a:\n    extends: [ghost]\n",
        "profiles:\n  a:\n    extends: [b]\n  b:\n    extends: [a]\n",
        "profiles:\n  a:\n    extends: [a]\n",
        "profiles:\n  \"x/y\":\n    skills: [coding]\n",
        "profiles:\n  a:\n    skils: [coding]\n",
    ] {
        let err = VaultConfig::parse(bad, Path::new("config.yaml")).unwrap_err();
        assert!(
            matches!(err, ConfigError::InvalidVaultConfig { .. }),
            "{bad}: {err}"
        );
    }
}

#[test]
fn targets_name_known_agent_folders_and_nothing_else() {
    let ok = VaultConfig::parse("targets: [claude]\n", Path::new("config.yaml")).unwrap();
    assert_eq!(ok.targets, ["claude"]);
    assert!(
        VaultConfig::default().targets.is_empty(),
        "no targets unless the config names them"
    );

    for (bad, wanted) in [
        ("targets: [cursor]\n", "cursor"),
        ("targets: [claude, claude]\n", "twice"),
        ("targets: [\"\"]\n", "\"\""),
    ] {
        let err = VaultConfig::parse(bad, Path::new("config.yaml")).unwrap_err();
        assert!(
            matches!(&err, ConfigError::InvalidVaultConfig { message, .. } if message.contains(wanted)),
            "{bad}: {err}"
        );
    }
    let err = VaultConfig::parse("targets: [cursor]\n", Path::new("config.yaml")).unwrap_err();
    assert!(
        err.to_string().contains("claude"),
        "the message lists the known names: {err}"
    );
}
