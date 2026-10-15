//! A build without the web view says so, and how to build it in, instead of failing in some other way.
#![cfg(not(feature = "web"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod common;

use common::skillmirror;
use predicates::str::contains;
use skillmirror_testkit::World;

#[test]
fn the_web_command_of_a_build_without_the_web_view_says_how_to_build_it_in() {
    let world = World::standard();

    skillmirror(&world)
        .arg("web")
        .assert()
        .code(2)
        .stderr(contains("no web view"))
        .stderr(contains("--features web"))
        .stderr(contains("just install-web"));
}
