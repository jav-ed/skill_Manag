use std::path::PathBuf;

use super::*;
use crate::Hint;
use crate::testutil::TempTree;

fn dirs_of(tree: &TempTree, options: &ScanOptions) -> Vec<String> {
    let report = scan(tree.path(), options).unwrap();
    report
        .skills_dirs
        .iter()
        .map(|d| {
            d.dir
                .strip_prefix(tree.path())
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

fn skill(tree: &TempTree, project: &str, name: &str) {
    tree.write(&format!("{project}/.agents/skills/{name}/SKILL.md"), "x");
}

#[test]
fn finds_projects_in_walk_order_and_stops_at_skills() {
    let tree = TempTree::new();
    skill(&tree, "b", "coding");
    skill(&tree, "a", "coding");
    skill(&tree, "a-b", "coding");
    tree.write("a/sub/.agents/skills/nested/SKILL.md", "x");
    tree.write("b/.agents/skills/coding/.agents/skills/inner/SKILL.md", "x");
    let found = dirs_of(&tree, &ScanOptions::default());
    assert_eq!(
        found,
        [
            "a/.agents/skills",
            "a/sub/.agents/skills",
            "a-b/.agents/skills",
            "b/.agents/skills",
        ]
    );
}

#[test]
fn noise_and_excluded_directories_are_pruned() {
    let tree = TempTree::new();
    skill(&tree, "keep", "coding");
    skill(&tree, "node_modules/x", "coding");
    skill(&tree, "deep/build/x", "coding");
    skill(&tree, "fixtures/one", "coding");
    skill(&tree, "fixtures/two", "coding");
    skill(&tree, "sub/other", "coding");
    skill(&tree, "sub/other2", "coding");
    skill(&tree, "named/skip_me", "coding");
    let options = ScanOptions {
        exclude_dirs: vec!["skip_me".into()],
        exclude_paths: vec![PathBuf::from("fixtures"), tree.path().join("sub/other")],
    };
    assert_eq!(
        dirs_of(&tree, &options),
        ["keep/.agents/skills", "sub/other2/.agents/skills"]
    );
}

#[test]
fn a_root_named_like_noise_is_still_scanned() {
    let tree = TempTree::new();
    skill(&tree, "build/project", "coding");
    let root = tree.path().join("build");
    let report = scan(&root, &ScanOptions::default()).unwrap();
    assert_eq!(report.skills_dirs.len(), 1);
}

#[test]
fn hidden_directories_other_than_noise_are_walked_and_case_matters() {
    let tree = TempTree::new();
    tree.write(".hidden/proj/.agents/skills/coding/SKILL.md", "x");
    tree.write("upper/.AGENTS/skills/coding/SKILL.md", "x");
    tree.write("upper2/.agents/Skills/coding/SKILL.md", "x");
    assert_eq!(
        dirs_of(&tree, &ScanOptions::default()),
        [".hidden/proj/.agents/skills"]
    );
}

#[test]
fn symlinks_are_never_followed() {
    let tree = TempTree::new();
    skill(&tree, "real", "coding");
    std::os::unix::fs::symlink(tree.path().join("real"), tree.path().join("link")).unwrap();
    tree.write("linked/.agents/other/x", "x");
    std::os::unix::fs::symlink(
        tree.path().join("real/.agents/skills"),
        tree.path().join("linked/.agents/skills"),
    )
    .unwrap();
    assert_eq!(
        dirs_of(&tree, &ScanOptions::default()),
        ["real/.agents/skills"]
    );
}

#[test]
fn missing_or_file_root_is_a_hard_error() {
    let tree = TempTree::new();
    let file = tree.write("f", "x");
    assert!(matches!(
        scan(&tree.path().join("nope"), &ScanOptions::default()),
        Err(ScanError::RootMissing { .. })
    ));
    assert!(matches!(
        scan(&file, &ScanOptions::default()),
        Err(ScanError::RootNotADirectory { .. })
    ));
}

#[test]
fn exclude_path_that_climbs_out_is_a_hard_error() {
    let tree = TempTree::new();
    let escaping = ScanOptions {
        exclude_paths: vec![PathBuf::from("/..")],
        ..ScanOptions::default()
    };
    assert!(matches!(
        scan(tree.path(), &escaping),
        Err(ScanError::ExcludePathEscapes { .. })
    ));
}

#[test]
fn unreadable_directory_is_reported_not_swallowed() {
    use std::os::unix::fs::PermissionsExt;
    let tree = TempTree::new();
    skill(&tree, "ok", "coding");
    let locked = tree.path().join("locked");
    std::fs::create_dir(&locked).unwrap();
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();
    let report = scan(tree.path(), &ScanOptions::default()).unwrap();
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
    if nix_is_root() {
        return;
    }
    assert_eq!(report.skills_dirs.len(), 1);
    assert_eq!(report.issues.len(), 1, "{:?}", report.issues);
    assert!(report.issues[0].path.ends_with("locked"));
}

fn nix_is_root() -> bool {
    std::fs::metadata("/proc/self").is_ok_and(|m| std::os::unix::fs::MetadataExt::uid(&m) == 0)
}

#[test]
fn targets_follow_the_opt_in_rule_and_ignore_files_and_symlinks() {
    let tree = TempTree::new();
    skill(&tree, "p", "coding");
    skill(&tree, "p", "other");
    skill(&tree, "q", "coding");
    tree.write("p/.agents/skills/loose-file", "x");
    std::os::unix::fs::symlink(
        tree.path().join("q"),
        tree.path().join("p/.agents/skills/linked"),
    )
    .unwrap();
    let report = scan(tree.path(), &ScanOptions::default()).unwrap();

    let synced = sync_targets(&report.skills_dirs, |n| n == "coding");
    let names: Vec<_> = synced
        .targets
        .iter()
        .map(|t| {
            (
                t.project.file_name().unwrap().to_str().unwrap(),
                t.skill.as_str(),
            )
        })
        .collect();
    assert_eq!(names, [("p", "coding"), ("q", "coding")]);

    let all = all_targets(&report.skills_dirs);
    assert_eq!(all.targets.len(), 3);

    let pushed = push_targets(
        &report.skills_dirs,
        &["coding".to_string(), "new".to_string()],
    );
    assert_eq!(pushed.len(), 4);
    assert!(pushed.iter().any(|t| t.skill == "new" && !t.path.exists()));
}

#[test]
fn a_root_containing_dotdot_still_honours_exclude_paths() {
    let tree = TempTree::new();
    skill(&tree, "real/keep", "coding");
    skill(&tree, "real/secret", "coding");
    std::fs::create_dir_all(tree.path().join("x")).unwrap();
    let root = tree.path().join("x/../real");
    let options = ScanOptions {
        exclude_dirs: Vec::new(),
        exclude_paths: vec![PathBuf::from("secret"), tree.path().join("real/other")],
    };
    std::fs::create_dir_all(tree.path().join("real/other")).unwrap();
    let report = scan(&root, &options).unwrap();
    let found: Vec<_> = report
        .skills_dirs
        .iter()
        .map(|d| {
            d.project
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(
        found,
        ["keep"],
        "the excluded folder stays out under a `..` root"
    );
}

#[test]
fn a_symlinked_root_is_walked_and_excludes_given_by_the_real_path_apply() {
    let tree = TempTree::new();
    skill(&tree, "real/keep", "coding");
    skill(&tree, "real/secret", "coding");
    let link = tree.path().join("link");
    std::os::unix::fs::symlink(tree.path().join("real"), &link).unwrap();
    let options = ScanOptions {
        exclude_dirs: Vec::new(),
        exclude_paths: vec![tree.path().join("real/secret")],
    };
    let report = scan(&link, &options).unwrap();
    assert_eq!(report.skills_dirs.len(), 1);
    assert!(
        report
            .skills_dirs
            .first()
            .unwrap()
            .project
            .ends_with("keep")
    );
}

#[test]
fn an_exclude_path_that_does_not_exist_is_a_hard_error() {
    let tree = TempTree::new();
    skill(&tree, "keep", "coding");
    let options = ScanOptions {
        exclude_dirs: Vec::new(),
        exclude_paths: vec![PathBuf::from("typo/in/path")],
    };
    let err = scan(tree.path(), &options).unwrap_err();
    assert!(matches!(err, ScanError::ExcludePathMissing { .. }), "{err}");
    assert!(err.hint().unwrap().contains("config.yaml"));
}
