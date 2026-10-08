//! `skill_detail`: one skill, from the vault side and from every project's side.

use super::status_tests::{built, open, world};
use super::*;
use crate::events::ignore_events;

fn detail(ws: &Workspace, name: &str) -> Result<SkillDetail, crate::Error> {
    let scanned = ws.scan(&ignore_events).unwrap();
    skill_detail(ws, &scanned, name)
}

fn states(detail: &SkillDetail) -> Vec<(String, &SkillState)> {
    detail
        .projects
        .iter()
        .map(|p| {
            (
                p.project
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                &p.state,
            )
        })
        .collect()
}

#[test]
fn a_vault_skill_shows_its_group_files_and_where_it_is_outdated() {
    let (_tree, ws) = world("tmux");
    let info = detail(&ws, "astro").unwrap();

    let skill = info.info.as_ref().expect("astro is in the vault");
    assert_eq!(skill.group, ["web"]);
    assert_eq!(skill.files, ["SKILL.md", "ref.md"]);
    assert!(!skill.mandatory);
    let found = states(&info);
    assert_eq!(found.len(), 1, "only project b has astro: {found:?}");
    assert_eq!(found[0].0, "b");
    assert!(matches!(found[0].1, SkillState::Outdated(o) if o.added == 1 && o.changed == 1));
    assert_eq!(info.total_projects, 4);
    assert!(info.drifts(), "an outdated copy is drift");
}

#[test]
fn a_mandatory_skill_names_the_projects_that_lack_it() {
    let (_tree, ws) = world("tmux");
    let info = detail(&ws, "tmux").unwrap();

    assert!(info.info.as_ref().unwrap().mandatory);
    let found: Vec<(String, bool)> = states(&info)
        .into_iter()
        .map(|(p, s)| (p, matches!(s, SkillState::MissingMandatory)))
        .collect();
    assert_eq!(
        found,
        [
            ("a".to_string(), false),
            ("b".to_string(), false),
            ("c".to_string(), true),
            ("d".to_string(), true)
        ]
    );
}

#[test]
fn a_folder_the_vault_lacks_is_still_described() {
    let (_tree, ws) = world("tmux");
    let info = detail(&ws, "local-only").unwrap();

    assert!(info.info.is_none(), "the vault has no local-only");
    assert_eq!(states(&info).len(), 1);
    assert!(matches!(states(&info)[0].1, SkillState::NotInVault));
    assert!(!info.drifts(), "nothing the tool does could fix it");
}

#[test]
fn a_name_that_is_nowhere_is_an_error() {
    let (_tree, ws) = world("tmux");
    let error = detail(&ws, "ghost").unwrap_err();
    assert!(
        matches!(
            error,
            crate::Error::Select(SelectError::UnknownSkill { .. })
        ),
        "got {error:?}"
    );
}

#[test]
fn the_profiles_that_select_a_skill_are_named() {
    let tree = built(
        "tmux",
        "profiles:\n  websites:\n    groups: [web]\n  base:\n    skills: [coding]\n  site:\n    extends: [websites]\n    exclude: [astro]\n",
    );
    let ws = open(&tree);
    let info = detail(&ws, "astro").unwrap();
    assert_eq!(info.profiles, ["websites"], "site excludes astro");
}

#[test]
fn files_git_does_not_track_are_listed_apart() {
    let tree = built("tmux", "");
    tree.write("vault/coding/draft.md", "half finished");
    let ws = open(&tree);
    let info = detail(&ws, "coding").unwrap();
    let skill = info.info.unwrap();
    assert_eq!(skill.files, ["SKILL.md"]);
    assert_eq!(skill.untracked, ["draft.md"]);
}

#[test]
fn a_skill_that_cannot_be_compared_counts_as_a_failure() {
    // Mandatory, so that the plan looks at the folder even though the scan does not list a link.
    let tree = built("tmux, coding", "");
    // The project copy of coding is a symlink: the plan refuses to write through it.
    std::fs::remove_dir_all(tree.path().join("projects/c/.agents/skills/coding")).unwrap();
    std::os::unix::fs::symlink(
        tree.path().join("projects/a/.agents/skills/coding"),
        tree.path().join("projects/c/.agents/skills/coding"),
    )
    .unwrap();
    let ws = open(&tree);
    let info = detail(&ws, "coding").unwrap();
    assert_eq!(info.failed(), 1, "got {:?}", states(&info));
}
