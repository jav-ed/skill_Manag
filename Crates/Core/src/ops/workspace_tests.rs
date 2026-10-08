//! `Workspace::scan` tells the observer how the scan went.

use std::sync::Mutex;

use super::*;
use crate::config::{Dirs, EnvOverrides, Flags, Settings};
use crate::events::Event;
use crate::testutil::TempTree;

fn workspace(tree: &TempTree) -> Workspace {
    let flags = Flags {
        vault: Some(tree.path().join("vault")),
        root: None,
    };
    let dirs = Dirs::under(&tree.path().join("home"));
    Workspace::open(Settings::load(&flags, &EnvOverrides::default(), &dirs).unwrap()).unwrap()
}

#[test]
fn scan_reports_progress_and_then_the_end() {
    let tree = TempTree::new();
    tree.write("vault/coding/SKILL.md", "coding");
    tree.write(
        "vault/config.yaml",
        &format!("root: {}/projects\n", tree.path().display()),
    );
    for n in 0..300 {
        tree.write(&format!("projects/many/d{n}/keep"), "x");
    }
    tree.write("projects/p/.agents/skills/coding/SKILL.md", "coding");
    tree.git_init_commit("vault");
    let ws = workspace(&tree);
    let heard = Mutex::new(Vec::new());

    let report = ws.scan(&|event| heard.lock().unwrap().push(event)).unwrap();

    let heard = heard.into_inner().unwrap();
    assert_eq!(report.skills_dirs.len(), 1);
    assert!(
        matches!(
            heard.first(),
            Some(Event::ScanProgress {
                directories: 256,
                ..
            })
        ),
        "{heard:?}"
    );
    assert_eq!(
        heard.last(),
        Some(&Event::ScanFinished {
            skills_dirs: 1,
            issues: 0
        }),
        "the end comes last"
    );
}
