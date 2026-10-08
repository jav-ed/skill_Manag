//! New skills: created from a template, staged, never committed, never over something that exists.

use std::path::PathBuf;

use super::*;
use crate::testutil::Fixture;
use crate::vault::read_header;

fn world() -> Fixture {
    Fixture::new(&[
        ("coding/SKILL.md", "coding"),
        ("web/astro/SKILL.md", "astro"),
        ("config.yaml", "mandatory: []\n"),
    ])
}

fn staged(fixture: &Fixture) -> String {
    fixture.tree.git("vault", &["status", "--porcelain"])
}

fn commits(fixture: &Fixture) -> usize {
    fixture
        .tree
        .git("vault", &["rev-list", "--count", "HEAD"])
        .trim()
        .parse()
        .unwrap()
}

#[test]
fn a_new_skill_gets_a_header_that_reads_back_and_is_staged_not_committed() {
    let world = world();
    let before = commits(&world);

    let made = new_skill(&world.vault, "tmux", "", Some("Drive tmux: panes, windows")).unwrap();

    assert_eq!(made.dir, world.vault.path.join("tmux"));
    assert_eq!(made.files, [PathBuf::from("SKILL.md")]);
    assert!(
        made.left_out.is_empty(),
        "nothing was left out: {:?}",
        made.left_out
    );
    assert_eq!(staged(&world), "A  tmux/SKILL.md\n");
    assert_eq!(commits(&world), before, "nothing is committed");
    let now = world.refresh();
    let skill = now
        .vault
        .skills
        .get("tmux")
        .expect("the new skill is found");
    let header = read_header(skill).unwrap();
    assert_eq!(header.name.as_deref(), Some("tmux"));
    assert_eq!(
        header.description.as_deref(),
        Some("Drive tmux: panes, windows")
    );
    let tracked = &now.files.get("tmux").unwrap().tracked;
    assert_eq!(tracked.len(), 1, "git knows the file, so sync can copy it");
}

#[test]
fn a_description_with_quotes_and_line_breaks_stays_one_valid_header() {
    let world = world();
    new_skill(
        &world.vault,
        "quoted",
        "",
        Some("say \"hi\"\\ now\nand then"),
    )
    .unwrap();
    let now = world.refresh();
    let header = read_header(now.vault.skills.get("quoted").unwrap()).unwrap();
    assert_eq!(
        header.description.as_deref(),
        Some("say \"hi\"\\ now and then")
    );
}

#[test]
fn without_a_description_the_header_says_what_to_fill_in() {
    let world = world();
    new_skill(&world.vault, "later", "", None).unwrap();
    let now = world.refresh();
    let header = read_header(now.vault.skills.get("later").unwrap()).unwrap();
    assert!(
        header.description.unwrap().starts_with("TODO"),
        "an empty description would hide the skill from agents without a word"
    );
}

#[test]
fn a_group_is_made_when_it_is_new_and_reused_when_it_exists() {
    let world = world();
    new_skill(&world.vault, "seo", "web/seo-tools", None).unwrap();
    new_skill(&world.vault, "next", "web", None).unwrap();
    let now = world.refresh();
    assert_eq!(now.vault.skills["seo"].group, ["web", "seo-tools"]);
    assert_eq!(now.vault.skills["next"].group, ["web"]);
    assert_eq!(
        now.vault.skills["astro"].group,
        ["web"],
        "the old one is untouched"
    );
}

#[test]
fn a_name_that_is_taken_or_is_a_group_is_refused_and_nothing_is_written() {
    let world = world();
    let status_before = staged(&world);

    let taken = new_skill(&world.vault, "astro", "", None).unwrap_err();
    assert!(matches!(taken, AuthorError::NameTaken { .. }), "{taken:?}");
    let group = new_skill(&world.vault, "web", "", None).unwrap_err();
    assert!(
        matches!(group, AuthorError::NameIsGroup { .. }),
        "{group:?}"
    );

    assert_eq!(staged(&world), status_before);
    assert!(!world.vault.path.join("web/SKILL.md").exists());
}

#[test]
fn names_and_groups_that_would_escape_or_confuse_are_refused() {
    let world = world();
    for name in ["", "Upper", "a/b", "..", ".hidden", "with space", "-dash"] {
        assert!(
            matches!(
                new_skill(&world.vault, name, "", None),
                Err(AuthorError::InvalidName { .. })
            ),
            "{name:?} must be refused"
        );
    }
    for group in ["../out", "a//b", ".git", "a/./b"] {
        assert!(
            matches!(
                new_skill(&world.vault, "fine", group, None),
                Err(AuthorError::InvalidGroup { .. })
            ),
            "group {group:?} must be refused"
        );
    }
    assert!(matches!(
        new_skill(&world.vault, "fine", "coding", None),
        Err(AuthorError::GroupIsSkill { .. })
    ));
    assert!(matches!(
        new_skill(&world.vault, "fine", "a/b/c/d/e", None),
        Err(AuthorError::TooDeep { .. })
    ));
    assert_eq!(staged(&world), "", "none of this touched the vault");
}

#[test]
fn a_group_folder_that_is_a_link_is_never_written_through() {
    let world = world();
    let elsewhere = world.tree.path().join("elsewhere");
    std::fs::create_dir(&elsewhere).unwrap();
    std::os::unix::fs::symlink(&elsewhere, world.vault.path.join("linked")).unwrap();

    let error = new_skill(&world.vault, "inside", "linked", None).unwrap_err();

    assert!(
        matches!(error, AuthorError::NotARealFolder { .. }),
        "{error:?}"
    );
    assert_eq!(std::fs::read_dir(&elsewhere).unwrap().count(), 0);
}

#[test]
fn when_git_refuses_the_folder_the_vault_is_left_as_it_was() {
    let world = world();
    world.tree.write("vault/.gitignore", "blocked/\n");
    world.tree.git("vault", &["add", ".gitignore"]);

    let error = new_skill(&world.vault, "blocked", "newgroup", None).unwrap_err();
    assert!(!matches!(error, AuthorError::Io(_)), "{error:?}");
    let error = new_skill(&world.vault, "blocked", "", None).unwrap_err();

    assert!(matches!(error, AuthorError::GitAdd { .. }), "{error:?}");
    assert!(!world.vault.path.join("blocked").exists(), "rolled back");
    assert!(
        !world.vault.path.join("newgroup").exists(),
        "a group made for the failed skill is taken back too"
    );
}

#[test]
fn a_file_the_vault_ignores_is_reported_as_left_out() {
    let world = world();
    world.tree.write("vault/.gitignore", "SKILL.md\n");
    world.tree.git("vault", &["add", ".gitignore"]);

    let made = new_skill(&world.vault, "ignored", "", None).unwrap();

    assert_eq!(made.left_out, [PathBuf::from("SKILL.md")]);
}
