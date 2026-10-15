//! A build without the web view says so, and how to build it in, instead of failing in some other way.
#![cfg(not(feature = "web"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod common;

use common::skillmirror;
use predicates::prelude::*;
use predicates::str::contains;
use skillmirror_testkit::World;

#[test]
fn the_web_command_of_a_build_without_the_web_view_says_what_to_turn_on_and_how_to_build_it_in() {
    let world = World::standard();

    skillmirror(&world)
        .arg("web")
        .assert()
        .code(2)
        .stderr(contains("switched off in the config"))
        .stderr(contains("no web view"))
        .stderr(contains("config web on"))
        .stderr(contains("--features web"))
        .stderr(contains("just install-web"));
}

#[test]
fn with_the_switch_on_the_build_is_the_only_thing_left_to_name() {
    let world = World::standard();
    skillmirror(&world)
        .args(["config", "web", "on"])
        .assert()
        .success();

    skillmirror(&world)
        .arg("web")
        .assert()
        .code(2)
        .stderr(contains("no web view"))
        .stderr(contains("--features web"))
        .stderr(contains("switched off").not());
}
