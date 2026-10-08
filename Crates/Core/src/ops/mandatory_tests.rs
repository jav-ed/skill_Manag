use super::mandatory::Change;
use super::*;
use crate::testutil::Fixture;

fn vault() -> Fixture {
    Fixture::new(&[
        ("coding/SKILL.md", "c"),
        ("tmux/SKILL.md", "t"),
        ("web/astro/SKILL.md", "a"),
    ])
}

fn list(names: &[&str]) -> Vec<String> {
    names.iter().map(|n| (*n).to_string()).collect()
}

#[test]
fn adding_appends_in_order_and_skips_what_is_listed() {
    let world = vault();
    let now = list(&["coding"]);

    let after = change_mandatory(
        &world.vault,
        &now,
        Change::Add(&list(&["tmux", "coding", "astro"])),
    )
    .unwrap();

    assert_eq!(after, ["coding", "tmux", "astro"]);
}

#[test]
fn adding_an_unknown_skill_changes_nothing() {
    let world = vault();
    let error = change_mandatory(
        &world.vault,
        &list(&["coding"]),
        Change::Add(&list(&["tmux", "ghost"])),
    )
    .unwrap_err();
    assert!(
        matches!(
            error,
            MandatoryError::Select(SelectError::UnknownSkill { .. })
        ),
        "{error:?}"
    );
}

#[test]
fn removing_drops_the_names_and_keeps_the_order_of_the_rest() {
    let world = vault();
    let after = change_mandatory(
        &world.vault,
        &list(&["coding", "tmux", "astro"]),
        Change::Remove(&list(&["tmux"])),
    )
    .unwrap();
    assert_eq!(after, ["coding", "astro"]);
}

#[test]
fn a_name_the_vault_lost_can_still_be_removed() {
    let world = vault();
    let after = change_mandatory(
        &world.vault,
        &list(&["coding", "gone"]),
        Change::Remove(&list(&["gone"])),
    )
    .unwrap();
    assert_eq!(after, ["coding"]);
}

#[test]
fn removing_a_name_that_is_not_listed_is_an_error() {
    let world = vault();
    let error = change_mandatory(
        &world.vault,
        &list(&["coding"]),
        Change::Remove(&list(&["tmux"])),
    )
    .unwrap_err();
    assert!(
        matches!(error, MandatoryError::NotListed { .. }),
        "{error:?}"
    );
}
