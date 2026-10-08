//! Shared helpers for the end-to-end tests.
#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)]

pub(crate) mod pty;

use assert_cmd::Command;
use skillmirror_testkit::World;

/// The binary with an isolated environment and the world's vault and root given as flags.
pub(crate) fn skillmirror(world: &World) -> Command {
    let mut cmd = bare(world);
    cmd.arg("--vault")
        .arg(world.vault())
        .arg("--root")
        .arg(world.root());
    cmd
}

/// The binary with an isolated environment and no flags.
pub(crate) fn bare(world: &World) -> Command {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("skillmirror");
    cmd.env_clear().envs(world.env());
    cmd
}

/// Runs a command and parses its stdout as one JSON document.
pub(crate) fn json_of(cmd: &mut Command) -> serde_json::Value {
    let out = cmd.output().unwrap();
    assert!(
        out.status.success() || out.status.code() == Some(1),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|e| panic!("not JSON ({e}): {}", String::from_utf8_lossy(&out.stdout)))
}
