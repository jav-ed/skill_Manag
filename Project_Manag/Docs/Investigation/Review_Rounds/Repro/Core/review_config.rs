//! Review round 2, series S (`save_config` text rewrite) and P (profiles). They print what they observe.
//!
//! Findings: M4 (S1, S2), L3 (P1), L4 (P2). See `Project_Manag/Docs/Investigation/Review_Rounds/round_2_Full.md`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::print_stdout)]

mod review_common;

use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::PathBuf;
use std::time::Instant;

use review_common::{commit_all, write};
use skillmirror_core::config::{ConfigUpdate, VaultConfig, save_config};
use skillmirror_core::ops::{self, Selection};
use skillmirror_core::vault::discover;

fn show(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}

fn try_save(label: &str, original: &[u8], update: &ConfigUpdate<'_>) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.yaml");
    std::fs::write(&path, original).unwrap();
    let result = save_config(dir.path(), update);
    let after = std::fs::read(&path).unwrap();
    match result {
        Ok(()) => println!("S {label:34} OK  -> {}", show(&after)),
        Err(e) => {
            let text = e.to_string();
            let first = text.lines().next().unwrap_or("");
            println!(
                "S {label:34} ERR {first} | file untouched: {}",
                after == original
            );
        }
    }
}

#[test]
fn s1_save_config_edge_cases() {
    let names = |list: &[&str]| list.iter().map(|s| (*s).to_string()).collect::<Vec<_>>();
    let new = names(&["c"]);
    let upd = ConfigUpdate {
        root: None,
        mandatory: Some(&new),
    };
    try_save(
        "blank line inside the list",
        b"mandatory:\n  - a\n\n  - b\nexclude_dirs:\n  - x\n",
        &upd,
    );
    try_save(
        "col-0 comment inside the list",
        b"mandatory:\n  - a\n# note\n  - b\n",
        &upd,
    );
    try_save(
        "indented comment inside the list",
        b"mandatory:\n  - a\n  # old: b\n  - b\nroot: /x\n",
        &upd,
    );
    try_save(
        "CRLF file",
        b"root: /a\r\nmandatory:\r\n  - a\r\nexclude_dirs:\r\n  - x\r\n",
        &upd,
    );
    try_save(
        "flow list over two lines",
        b"mandatory: [a,\n  b]\nroot: /x\n",
        &upd,
    );
    try_save(
        "col-0 items, next key follows",
        b"mandatory:\n- a\n- b\nroot: /x\n",
        &upd,
    );
    try_save(
        "trailing comment on the key",
        b"mandatory: # my list\n  - a\nroot: /x\n",
        &upd,
    );
    try_save(
        "document start marker",
        b"---\nroot: /x\nmandatory: [a]\n",
        &upd,
    );
    try_save("no trailing newline", b"root: /x\nmandatory: [a]", &upd);
    try_save("key missing: appended", b"root: /x\n# end\n", &upd);
    try_save("only comments", b"# nothing here\n", &upd);
    try_save("quoted key", b"\"mandatory\": [a]\nroot: /x\n", &upd);
    try_save(
        "BOM at start, key on line 1",
        "\u{feff}mandatory: [a]\nroot: /x\n".as_bytes(),
        &upd,
    );
    try_save(
        "profiles with a nested `root:` key",
        b"profiles:\n  root:\n    skills: [a]\nmandatory: [a]\n",
        &upd,
    );
    try_save("unparsable file", b"mandatory: [a\n", &upd);

    let root = PathBuf::from("/tmp/a \"b\" \\c #d: e");
    let upd = ConfigUpdate {
        root: Some(&root),
        mandatory: None,
    };
    try_save(
        "root with quotes, backslash, #",
        b"root: /x\nmandatory: [a]\n",
        &upd,
    );
    let weird = names(&[
        "1password",
        "null",
        "y",
        "yes",
        "on",
        "a b",
        "é",
        "-x",
        "a:b",
        "x#y",
        " lead",
        "tail ",
        "~",
        "0x1f",
        "1e3",
    ]);
    let upd = ConfigUpdate {
        root: None,
        mandatory: Some(&weird),
    };
    try_save("awkward mandatory names", b"root: /x\n", &upd);
}

#[test]
fn s2_save_config_replaces_a_symlinked_config_and_read_only_mode() {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().join("vault");
    let dotfiles = dir.path().join("dotfiles");
    std::fs::create_dir_all(&vault).unwrap();
    write(&dotfiles.join("config.yaml"), "root: /x\nmandatory: [a]\n");
    symlink(dotfiles.join("config.yaml"), vault.join("config.yaml")).unwrap();
    let new = vec!["b".to_string()];
    save_config(
        &vault,
        &ConfigUpdate {
            root: None,
            mandatory: Some(&new),
        },
    )
    .unwrap();
    let meta = std::fs::symlink_metadata(vault.join("config.yaml")).unwrap();
    println!(
        "S2 config.yaml is still a symlink: {}",
        meta.file_type().is_symlink()
    );
    println!(
        "S2 dotfiles copy now: {:?}",
        std::fs::read_to_string(dotfiles.join("config.yaml")).unwrap()
    );
    println!(
        "S2 vault copy now:    {:?}",
        std::fs::read_to_string(vault.join("config.yaml")).unwrap()
    );

    let ro = dir.path().join("ro");
    write(&ro.join("config.yaml"), "root: /x\nmandatory: [a]\n");
    std::fs::set_permissions(
        ro.join("config.yaml"),
        std::fs::Permissions::from_mode(0o444),
    )
    .unwrap();
    let saved = save_config(
        &ro,
        &ConfigUpdate {
            root: None,
            mandatory: Some(&new),
        },
    );
    let now = std::fs::read_to_string(ro.join("config.yaml")).unwrap();
    println!(
        "S2 read-only (0444) config: saved={}; content now {now:?}",
        saved.is_ok()
    );
}

/// Each level extends the two profiles of the level below, so the number of paths doubles per level.
fn ladder(levels: usize) -> String {
    let mut text = String::from("profiles:\n  a0: {}\n  b0: {}\n");
    for i in 1..levels {
        let p = i - 1;
        text.push_str(&format!(
            "  a{i}:\n    extends: [a{p}, b{p}]\n  b{i}:\n    extends: [a{p}, b{p}]\n"
        ));
    }
    text
}

#[test]
fn p1_profile_diamonds_are_exponential() {
    for levels in [8, 12, 16, 18, 20, 22] {
        let dir = tempfile::tempdir().unwrap();
        write(&dir.path().join("config.yaml"), &ladder(levels));
        let started = Instant::now();
        let loaded = VaultConfig::load(dir.path());
        let state = if loaded.is_ok() { "ok" } else { "ERR" };
        println!(
            "P1 {levels:2} levels: load {state} in {:?}",
            started.elapsed()
        );
        if started.elapsed().as_secs() > 5 {
            println!("P1 stopping: too slow");
            break;
        }
    }
}

#[test]
fn p2_profile_exclude_depends_on_the_order_of_extends() {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().join("vault");
    for skill in ["x", "y"] {
        write(&vault.join(skill).join("SKILL.md"), "s");
    }
    write(
        &vault.join("config.yaml"),
        "profiles:\n  a: {skills: [x, y]}\n  c: {exclude: [y]}\n  b1: {extends: [a, c]}\n  b2: {extends: [c, a]}\n",
    );
    commit_all(&vault);
    let found = discover(&vault).unwrap();
    let config = VaultConfig::load(&vault).unwrap();
    for name in ["b1", "b2"] {
        let selection = Selection {
            profiles: vec![name.to_string()],
            ..Selection::default()
        };
        println!(
            "P2 profile {name}: {:?}",
            ops::resolve(&found, &config, &selection)
        );
    }
}
