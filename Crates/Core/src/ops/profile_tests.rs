//! Profiles that extend each other: deep diamonds must stay fast, and the result must not depend on order.

use std::collections::BTreeSet;
use std::path::Path;
use std::time::{Duration, Instant};

use super::*;
use crate::config::VaultConfig;
use crate::testutil::Fixture;

fn vault() -> Fixture {
    Fixture::new(&[
        ("x/SKILL.md", "x"),
        ("y/SKILL.md", "y"),
        ("z/SKILL.md", "z"),
    ])
}

fn pick(f: &Fixture, config: &VaultConfig, profile: &str) -> BTreeSet<String> {
    let selection = Selection {
        profiles: vec![profile.to_string()],
        ..Selection::default()
    };
    resolve(&f.vault, config, &selection).unwrap()
}

fn set(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(ToString::to_string).collect()
}

#[test]
fn a_deep_diamond_of_profiles_loads_and_resolves_in_milliseconds() {
    let f = vault();
    // Every level has two profiles that both extend both profiles of the level below: 2^22 paths.
    let mut lines = vec![
        "profiles:".to_string(),
        "  l0_a:\n    skills: [x]".to_string(),
        "  l0_b:\n    skills: [y]".to_string(),
    ];
    for level in 1..=22 {
        for side in ["a", "b"] {
            let below = level - 1;
            lines.push(format!(
                "  l{level}_{side}:\n    extends: [l{below}_a, l{below}_b]"
            ));
        }
    }
    let yaml = lines.join("\n") + "\n";
    let started = Instant::now();

    let config = VaultConfig::parse(&yaml, Path::new("config.yaml")).unwrap();
    let chosen = pick(&f, &config, "l22_a");

    assert_eq!(chosen, set(&["x", "y"]));
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "took {:?}",
        started.elapsed()
    );
}

const ORDER: &str = "\
profiles:
  a:
    skills: [x, y]
  c:
    exclude: [y]
  b1:
    extends: [a, c]
  b2:
    extends: [c, a]
  d:
    extends: [a]
    exclude: [y]
  e:
    extends: [d]
    skills: [y]
";

#[test]
fn the_order_of_extends_never_changes_the_result() {
    let f = vault();
    let config = VaultConfig::parse(ORDER, Path::new("config.yaml")).unwrap();

    assert_eq!(pick(&f, &config, "b1"), pick(&f, &config, "b2"));
    assert_eq!(pick(&f, &config, "b1"), set(&["x", "y"]));
}

#[test]
fn exclude_removes_what_the_profile_itself_selects_and_a_child_can_add_it_back() {
    let f = vault();
    let config = VaultConfig::parse(ORDER, Path::new("config.yaml")).unwrap();

    let only_exclude = Selection {
        profiles: vec!["c".to_string()],
        ..Selection::default()
    };
    assert!(
        matches!(
            resolve(&f.vault, &config, &only_exclude),
            Err(SelectError::Empty)
        ),
        "an exclude alone selects nothing"
    );
    assert_eq!(pick(&f, &config, "d"), set(&["x"]));
    assert_eq!(pick(&f, &config, "e"), set(&["x", "y"]));
}

#[test]
fn an_exclude_of_an_unknown_skill_is_still_a_hard_error() {
    let f = vault();
    let yaml = "profiles:\n  p:\n    skills: [x]\n    exclude: [ghost]\n";
    let config = VaultConfig::parse(yaml, Path::new("config.yaml")).unwrap();
    let selection = Selection {
        profiles: vec!["p".to_string()],
        ..Selection::default()
    };

    assert!(matches!(
        resolve(&f.vault, &config, &selection),
        Err(SelectError::UnknownSkill { .. })
    ));
}
