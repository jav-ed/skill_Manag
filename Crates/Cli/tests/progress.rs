#![allow(clippy::unwrap_used, clippy::expect_used)]
//! The live line while the root is scanned: on a terminal only, never with `--json`, always cleared.
mod common;

use common::pty::Terminal;
use common::skillmirror;
use skillmirror_testkit::World;

/// The standard world with 600 more folders, so the scan is long enough to report progress.
fn wide_world() -> World {
    let world = World::standard();
    for n in 0..600 {
        world.project_file(&format!("many/d{n}/keep"), "x");
    }
    world
}

fn status_in_terminal(world: &World, extra: &[&str], env: &[(&str, &str)]) -> Terminal {
    let vault = world.vault();
    let root = world.root();
    let mut args = vec![
        "--vault",
        vault.to_str().unwrap(),
        "--root",
        root.to_str().unwrap(),
        "status",
    ];
    args.extend_from_slice(extra);
    Terminal::spawn_with(world, &args, 24, 140, env)
}

#[test]
fn a_terminal_sees_the_count_while_the_scan_runs_and_then_it_is_gone() {
    let world = wide_world();
    let mut term = status_in_terminal(&world, &[], &[]);

    term.wait_for("3 projects:");
    assert_eq!(term.exit_code(), 1);

    let raw = term.raw_text();
    assert!(
        raw.contains("scanning 256 folders"),
        "the line was drawn: {raw:?}"
    );
    assert!(
        !term.text().contains("scanning"),
        "the line is cleared before the report: {}",
        term.text()
    );
}

#[test]
fn json_and_a_dumb_terminal_get_no_line() {
    let world = wide_world();

    let mut json = status_in_terminal(&world, &["--json"], &[]);
    json.wait_for("\"summary\"");
    json.exit_code();
    assert!(
        !json.raw_text().contains("scanning"),
        "{:?}",
        json.raw_text()
    );

    let mut dumb = status_in_terminal(&world, &[], &[("TERM", "dumb")]);
    dumb.wait_for("3 projects:");
    dumb.exit_code();
    assert!(
        !dumb.raw_text().contains("scanning"),
        "{:?}",
        dumb.raw_text()
    );
}

#[test]
fn a_pipe_gets_nothing_on_stderr() {
    let world = wide_world();

    let out = skillmirror(&world).arg("status").output().unwrap();

    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "",
        "no line and no escape codes without a terminal"
    );
}

#[test]
fn list_and_delete_show_the_line_too() {
    let world = wide_world();
    let vault = world.vault();
    let root = world.root();
    for command in [&["list"][..], &["delete", "coding", "--dry-run"][..]] {
        let mut args = vec![
            "--vault",
            vault.to_str().unwrap(),
            "--root",
            root.to_str().unwrap(),
        ];
        args.extend_from_slice(command);
        let mut term = Terminal::spawn(&world, &args, 24, 140);

        term.exit_code();

        let raw = term.raw_text();
        assert!(raw.contains("scanning 256 folders"), "{command:?}: {raw:?}");
        assert!(!term.text().contains("scanning"), "{command:?}");
    }
}
