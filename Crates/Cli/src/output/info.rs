//! The JSON document and the text of `info`.

use std::path::PathBuf;

use serde::Serialize;
use skillmirror_core::ops::{ProjectState, SkillDetail, SkillInfo, SkillState};

use super::human::{pad, plural};
use super::status::file_counts;
use super::style::{ERROR, HEADER, MUTED, NAME, SUCCESS, WARNING};

/// Files above this count are not listed one by one in the text.
const FILES_SHOWN: usize = 12;

#[derive(Debug, Serialize)]
struct ProjectEntry {
    #[serde(serialize_with = "super::lossy::path")]
    project: PathBuf,
    /// `up_to_date`, `outdated`, `mandatory_missing`, `not_in_vault` or `problem`.
    state: &'static str,
    files_added: Option<usize>,
    files_changed: Option<usize>,
    files_removed: Option<usize>,
    message: Option<String>,
    hint: Option<String>,
}

impl ProjectEntry {
    fn of(entry: &ProjectState) -> Self {
        let mut out = Self {
            project: entry.project.clone(),
            state: "up_to_date",
            files_added: None,
            files_changed: None,
            files_removed: None,
            message: None,
            hint: None,
        };
        match &entry.state {
            SkillState::Current => {}
            SkillState::Outdated(o) => {
                out.state = "outdated";
                out.files_added = Some(o.added);
                out.files_changed = Some(o.changed);
                out.files_removed = Some(o.removed);
            }
            SkillState::MissingMandatory => out.state = "mandatory_missing",
            SkillState::NotInVault => out.state = "not_in_vault",
            SkillState::Problem(p) => {
                out.state = "problem";
                out.message = Some(p.message.clone());
                out.hint.clone_from(&p.hint);
            }
        }
        out
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct InfoJson {
    name: String,
    in_vault: bool,
    group: Vec<String>,
    description: Option<String>,
    header_problem: Option<String>,
    mandatory: bool,
    /// The files git tracks, which are the files that get copied.
    files: Vec<String>,
    /// Files in the skill folder that git does not track; never copied.
    untracked: Vec<String>,
    profiles: Vec<String>,
    projects: Vec<ProjectEntry>,
    total_projects: usize,
}

impl InfoJson {
    pub(crate) fn of(detail: &SkillDetail) -> Self {
        let info = detail.info.as_ref();
        Self {
            name: detail.name.clone(),
            in_vault: info.is_some(),
            group: info.map(|i| i.group.clone()).unwrap_or_default(),
            description: info.and_then(|i| i.description.clone()),
            header_problem: info.and_then(|i| i.header_problem.clone()),
            mandatory: info.is_some_and(|i| i.mandatory),
            files: info.map(|i| i.files.clone()).unwrap_or_default(),
            untracked: info.map(|i| i.untracked.clone()).unwrap_or_default(),
            profiles: detail.profiles.clone(),
            projects: detail.projects.iter().map(ProjectEntry::of).collect(),
            total_projects: detail.total_projects,
        }
    }

    pub(crate) fn render(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

pub(crate) fn render_info(detail: &SkillDetail) -> String {
    let mut out = String::new();
    if let Some(info) = &detail.info {
        vault_side(&mut out, info, &detail.profiles);
    } else {
        putln!(
            out,
            "{HEADER}{}{HEADER:#}\n  {WARNING}not in the vault{WARNING:#} {MUTED}(sync leaves the installed folders alone){MUTED:#}",
            detail.name
        );
    }
    if detail.projects.is_empty() {
        putln!(out, "\n{MUTED}Not installed in any project.{MUTED:#}");
        return out;
    }
    let installed = detail
        .projects
        .iter()
        .filter(|p| !matches!(p.state, SkillState::MissingMandatory))
        .count();
    putln!(
        out,
        "\nInstalled in {installed} of {}",
        plural(detail.total_projects, "project")
    );
    for entry in &detail.projects {
        project_line(&mut out, entry);
    }
    out
}

/// A label column, so that the values below one another start at the same place.
fn label(name: &str) -> String {
    format!("{name:<13}")
}

fn vault_side(out: &mut String, info: &SkillInfo, profiles: &[String]) {
    let mandatory = if info.mandatory { "  mandatory" } else { "" };
    putln!(
        out,
        "{HEADER}{}{HEADER:#}{WARNING}{mandatory}{WARNING:#}",
        info.name
    );
    let place = if info.group.is_empty() {
        "top level of the vault".to_string()
    } else {
        info.group.join("/")
    };
    putln!(out, "  {MUTED}{}{MUTED:#}{place}", label("group"));
    if let Some(text) = &info.description {
        putln!(out, "  {MUTED}{}{MUTED:#}{text}", label("description"));
    }
    if let Some(problem) = &info.header_problem {
        putln!(out, "  {ERROR}{}{ERROR:#}{problem}", label("header"));
    }
    if !profiles.is_empty() {
        putln!(
            out,
            "  {MUTED}{}{MUTED:#}{}",
            label("profiles"),
            profiles.join(", ")
        );
    }
    putln!(
        out,
        "  {MUTED}{}{MUTED:#}{} copied",
        label("files"),
        info.files.len()
    );
    file_list(out, &info.files);
    if !info.untracked.is_empty() {
        putln!(
            out,
            "  {WARNING}{}{WARNING:#}{} (git does not track {})",
            label("not copied"),
            plural(info.untracked.len(), "file"),
            if info.untracked.len() == 1 {
                "it"
            } else {
                "them"
            }
        );
        file_list(out, &info.untracked);
    }
}

fn file_list(out: &mut String, files: &[String]) {
    for file in files.iter().take(FILES_SHOWN) {
        putln!(out, "    {file}");
    }
    if files.len() > FILES_SHOWN {
        putln!(
            out,
            "    {MUTED}and {} more{MUTED:#}",
            files.len() - FILES_SHOWN
        );
    }
}

fn project_line(out: &mut String, entry: &ProjectState) {
    let (symbol, style, text) = match &entry.state {
        SkillState::Current => ("✓", SUCCESS, "up to date".to_string()),
        SkillState::Outdated(o) => ("~", WARNING, format!("outdated ({})", file_counts(o))),
        SkillState::MissingMandatory => ("+", WARNING, "mandatory, not installed".to_string()),
        SkillState::NotInVault => ("?", MUTED, "not in the vault".to_string()),
        SkillState::Problem(p) => ("✗", ERROR, p.message.clone()),
    };
    putln!(
        out,
        "  {style}{symbol}{style:#} {NAME}{}{NAME:#} {MUTED}{text}{MUTED:#}",
        pad(&entry.project.display().to_string())
    );
    if let SkillState::Problem(problem) = &entry.state
        && let Some(hint) = &problem.hint
    {
        putln!(out, "      {MUTED}hint: {hint}{MUTED:#}");
    }
}
