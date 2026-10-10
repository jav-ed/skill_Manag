//! The vault side: discovery, git, the config's mandatory list and profiles.

use super::{Report, Severity, agents_text, skills};
use crate::Hint;
use crate::agents::VAULT_FOLDER;
use crate::config::Settings;
use crate::ops::{SelectError, Selection, resolve};
use crate::vault::{IgnoredReason, discover, read_files};

pub(super) fn check(report: &mut Report, settings: &Settings) {
    report.ran("vault");
    let path = match settings.vault() {
        Ok(vault) => vault.value.clone(),
        Err(error) => {
            report.add(
                Severity::Error,
                "vault",
                None,
                error.to_string(),
                error.hint(),
            );
            return;
        }
    };
    let vault = match discover(&path) {
        Ok(vault) => vault,
        Err(error) => {
            report.add(
                Severity::Error,
                "vault",
                Some(path.display().to_string()),
                error.to_string(),
                error.hint(),
            );
            return;
        }
    };
    let text_folder = path.join(VAULT_FOLDER);
    for ignored in &vault.ignored {
        // The folder for the files that go into every project is meant to hold no skill.
        if ignored.path == text_folder {
            continue;
        }
        let (severity, message) = match ignored.reason {
            IgnoredReason::Symlink => (
                Severity::Warning,
                "a link in the vault is never taken as a skill or a group",
            ),
            IgnoredReason::NoSkillInside => (Severity::Note, "this folder holds no skill"),
        };
        report.add(
            severity,
            "vault",
            Some(ignored.path.display().to_string()),
            message,
            None,
        );
    }

    agents_text::check(report, &vault);

    report.ran("vault-git");
    let files = match read_files(&vault) {
        Ok(files) => files,
        Err(error) => {
            report.add(
                Severity::Error,
                "vault-git",
                Some(path.display().to_string()),
                error.to_string(),
                error.hint(),
            );
            return;
        }
    };
    skills::lint(report, &vault, &files);
    skills::edits(report, &vault);

    report.ran("mandatory");
    for name in settings.mandatory() {
        if !vault.skills.contains_key(name) {
            report.add(
                Severity::Error,
                "mandatory",
                Some(name.clone()),
                "listed as mandatory in the vault config, but the vault has no such skill",
                Some("`push` refuses to run until the name is right or removed".to_string()),
            );
        }
    }

    report.ran("profiles");
    for name in settings.config().profiles.keys() {
        let selection = Selection {
            profiles: vec![name.clone()],
            ..Selection::default()
        };
        match resolve(&vault, settings.config(), &selection) {
            Ok(_) => {}
            Err(SelectError::Empty) => report.add(
                Severity::Warning,
                "profiles",
                Some(name.clone()),
                "the profile selects no skill",
                None,
            ),
            Err(error) => report.add(
                Severity::Error,
                "profiles",
                Some(name.clone()),
                error.to_string(),
                error.hint(),
            ),
        }
    }
}
