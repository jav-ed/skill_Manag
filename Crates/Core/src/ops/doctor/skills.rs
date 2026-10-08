//! Skill by skill: the `SKILL.md` header, and edits git does not know about.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use super::{Report, Severity};
use crate::vault::{Header, Skill, Vault, VaultFiles, read_header};

/// Checks the header of every skill: it must exist, parse as YAML, name the folder and describe the skill.
pub(super) fn lint(report: &mut Report, vault: &Vault, files: &VaultFiles) {
    report.ran("skill-files");
    for skill in vault.skills.values() {
        let subject = Some(skill.name.clone());
        if files
            .get(&skill.name)
            .is_some_and(|f| !f.tracked.iter().any(|t| t.rel == Path::new("SKILL.md")))
        {
            report.add(
                Severity::Error,
                "skill-files",
                subject.clone(),
                "SKILL.md is not tracked by git, so sync refuses this skill",
                Some("`git add` it in the vault and commit".to_string()),
            );
        }
        match read_header(skill) {
            Ok(header) => judge(report, skill, &header),
            Err(problem) => {
                let hint = problem.contains("not valid YAML").then(|| {
                    "agents read name and description from the header; quote a value that contains a colon"
                        .to_string()
                });
                report.add(Severity::Warning, "skill-files", subject, problem, hint);
            }
        }
    }
}

fn judge(report: &mut Report, skill: &Skill, header: &Header) {
    let subject = Some(skill.name.clone());
    let mut say = |message: String| {
        report.add(
            Severity::Warning,
            "skill-files",
            subject.clone(),
            message,
            None,
        );
    };
    match header.name.as_deref() {
        None | Some("") => say("the header has no name".to_string()),
        Some(name) if name != skill.name => {
            say(format!(
                "the header says name {name:?}, the folder is called {:?}",
                skill.name
            ));
        }
        Some(_) => {}
    }
    if header
        .description
        .as_deref()
        .is_none_or(|d| d.trim().is_empty())
    {
        say(
            "the header has no description, so an agent cannot tell when to use the skill"
                .to_string(),
        );
    }
}

/// What git says about the skill folders. Sync copies the working-tree version of every tracked file, so
/// an uncommitted edit is already mirrored, an untracked file never is, and a tracked file missing from the
/// disk makes the skill fail.
pub(super) fn edits(report: &mut Report, vault: &Vault) {
    report.ran("skill-edits");
    let output = crate::git::command()
        .arg("-C")
        .arg(&vault.path)
        .args(["status", "--porcelain=v1", "-z", "--untracked-files=all"])
        .output();
    let output = match output {
        Ok(output) if output.status.success() => output,
        Ok(_) => {
            report.add(
                Severity::Warning,
                "skill-edits",
                None,
                "git status failed in the vault",
                None,
            );
            return;
        }
        Err(error) => {
            let message = format!("cannot ask git for the state of the vault: {error}");
            report.add(Severity::Warning, "skill-edits", None, message, None);
            return;
        }
    };
    let mut per_skill: BTreeMap<&str, Changes> = BTreeMap::new();
    // Each record is "XY path"; a renamed file is followed by its old name as a record of its own.
    for record in output.stdout.split(|b| *b == 0).filter(|r| r.len() > 3) {
        let (status, path) = record.split_at(3);
        let path = Path::new(OsStr::from_bytes(path));
        let Some(skill) = vault.skills.values().find(|s| path.starts_with(&s.rel)) else {
            continue;
        };
        let changes = per_skill.entry(skill.name.as_str()).or_default();
        match status.get(..2) {
            Some(b"??") => changes.untracked += 1,
            Some(code) if code.contains(&b'D') => changes.missing += 1,
            _ => changes.edited += 1,
        }
    }
    for (name, changes) in per_skill {
        let subject = Some(name.to_string());
        if changes.missing > 0 {
            report.add(
                Severity::Error,
                "skill-edits",
                subject.clone(),
                format!(
                    "{} tracked file(s) are missing from the folder, so sync fails for this skill",
                    changes.missing
                ),
                Some("restore them or `git rm` them and commit".to_string()),
            );
        }
        if changes.untracked > 0 {
            report.add(
                Severity::Warning,
                "skill-edits",
                subject.clone(),
                format!(
                    "{} new file(s) are not tracked by git and are not copied",
                    changes.untracked
                ),
                Some("`git add` them in the vault".to_string()),
            );
        }
        if changes.edited > 0 {
            report.add(
                Severity::Note,
                "skill-edits",
                subject,
                format!(
                    "{} file(s) changed since the last commit; sync already copies the edited version",
                    changes.edited
                ),
                None,
            );
        }
    }
}

#[derive(Default)]
struct Changes {
    edited: usize,
    untracked: usize,
    missing: usize,
}
