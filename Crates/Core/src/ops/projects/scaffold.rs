//! The scaffold of a new project (`Project_Manag/Docs`, `Scratch/`, `Repos/`, `.gitignore`, `just scratch-clean`)
//! is made by the doc-start skill's own script, `Code/bootstrap.sh`. `init` runs the project's copy of it right
//! after the skills are installed, so the skill stays the one place that says what a new project looks like.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Stdio;

use crate::ops::{ProjectError, Workspace};

/// The skill whose script makes the scaffold.
const SKILL: &str = "doc-start";
/// The script, relative to the skill folder.
const SCRIPT: &str = "Code/bootstrap.sh";
/// What the script calls or copies, relative to the skill folder; it fails without them.
const NEEDED: [&str; 3] = [
    SCRIPT,
    "Code/install_Scratch.sh",
    "Templates/clean_Scratch.sh",
];

/// Checked before anything is made: the skill is part of the install and the vault tracks the files the script needs.
pub fn require_scaffold(
    workspace: &Workspace,
    skills: &BTreeSet<String>,
) -> Result<(), ProjectError> {
    if !skills.contains(SKILL) {
        return Err(ProjectError::ScaffoldNeedsSkill);
    }
    let tracked = workspace.files.get(SKILL).map(|files| &files.tracked);
    for file in NEEDED {
        let found = tracked.is_some_and(|files| files.iter().any(|t| t.rel == Path::new(file)));
        if !found {
            return Err(ProjectError::ScaffoldFileMissing {
                file: file.to_string(),
            });
        }
    }
    Ok(())
}

/// Runs the project's copy of the script in `project` and returns what it printed, line by line. The script
/// keeps what exists, so running it again is harmless.
pub fn run_scaffold(project: &Path) -> Result<Vec<String>, ProjectError> {
    let script = project
        .join(".agents")
        .join("skills")
        .join(SKILL)
        .join(SCRIPT);
    let failed = |detail: String| ProjectError::ScaffoldRun {
        script: script.clone(),
        project: project.to_path_buf(),
        detail,
    };
    let out = std::process::Command::new("bash")
        .arg(&script)
        .arg(project)
        .current_dir(project)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| {
            failed(if e.kind() == std::io::ErrorKind::NotFound {
                "bash is not installed or not on PATH".to_string()
            } else {
                e.to_string()
            })
        })?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        let detail = if stderr.is_empty() {
            format!("the script ended with {}", out.status)
        } else {
            format!("{stderr} ({})", out.status)
        };
        return Err(failed(detail));
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(str::to_string)
        .collect())
}
