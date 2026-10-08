//! The rows and detail cards of the skills page: the vault's skills, and what each one is in every project.

use std::collections::BTreeMap;
use std::path::Path;

use skillmirror_core::ops::{self, SkillInfo};
use skillmirror_core::vault::Skill;

use crate::items::{Item, Note, Tone};
use crate::num::plural;
use crate::results::short_path;
use crate::session::{Session, State, TargetState};

/// Files named in the card before the rest is counted.
const FILES_SHOWN: usize = 6;

/// What a skill is in one project, in words.
struct InProject<'a> {
    project: &'a Path,
    text: String,
    tone: Tone,
}

/// One row per vault skill, top-level skills first, then by group.
pub(crate) fn vault_rows(session: &Session) -> Vec<Item> {
    let mut skills: Vec<&Skill> = session.workspace.vault.skills.values().collect();
    skills.sort_by(|a, b| a.group.cmp(&b.group).then_with(|| a.name.cmp(&b.name)));
    let by_skill = states_by_skill(session);
    skills
        .into_iter()
        .map(|skill| {
            let projects = by_skill
                .get(skill.name.as_str())
                .map_or(&[][..], Vec::as_slice);
            let info = ops::skill_info(&session.workspace, skill);
            Item {
                name: skill.name.clone(),
                detail: if skill.group.is_empty() {
                    String::new()
                } else {
                    skill.group.join("/")
                },
                note: Some(summary(projects, info.mandatory)),
                targets: Vec::new(),
                preselected: false,
                card: card(session, &info, projects),
            }
        })
        .collect()
}

/// The sync plan says what each installed copy is; the push plan adds the mandatory skills a project
/// lacks. Both were made by the scan, so no folder is read again here.
fn states_by_skill(session: &Session) -> BTreeMap<&str, Vec<InProject<'_>>> {
    let mut out: BTreeMap<&str, Vec<InProject<'_>>> = BTreeMap::new();
    for state in &session.sync {
        out.entry(state.target.skill.as_str())
            .or_default()
            .push(in_project(state));
    }
    if let Ok(push) = &session.push {
        for state in push {
            let known = out
                .get(state.target.skill.as_str())
                .is_some_and(|rows| rows.iter().any(|r| r.project == state.target.project));
            if !known && !matches!(state.state, State::Same) {
                out.entry(state.target.skill.as_str())
                    .or_default()
                    .push(in_project(state));
            }
        }
    }
    out
}

fn in_project(state: &TargetState) -> InProject<'_> {
    let (text, tone) = match &state.state {
        State::Same => ("up to date".to_string(), Tone::Quiet),
        State::Update(n) => (format!("outdated ({})", plural(*n, "file")), Tone::Change),
        State::Create(_) => ("mandatory, not installed".to_string(), Tone::Change),
        State::Failed(message) => (
            message.lines().next().unwrap_or_default().to_string(),
            Tone::Problem,
        ),
    };
    InProject {
        project: &state.target.project,
        text,
        tone,
    }
}

fn summary(projects: &[InProject<'_>], mandatory: bool) -> Note {
    let count = |tone: Tone| projects.iter().filter(|p| p.tone == tone).count();
    let (problems, changes) = (count(Tone::Problem), count(Tone::Change));
    let mut parts = Vec::new();
    if mandatory {
        parts.push("mandatory".to_string());
    }
    let (tone, text) = if problems > 0 {
        (Tone::Problem, plural(problems, "problem"))
    } else if changes > 0 {
        (Tone::Change, format!("{changes} to change"))
    } else if projects.is_empty() {
        (Tone::Quiet, "not installed".to_string())
    } else {
        (
            Tone::Quiet,
            format!("in {}", plural(projects.len(), "project")),
        )
    };
    parts.push(text);
    Note {
        text: parts.join(", "),
        tone,
    }
}

fn line(text: impl Into<String>, tone: Tone) -> Note {
    Note {
        text: text.into(),
        tone,
    }
}

fn card(session: &Session, info: &SkillInfo, projects: &[InProject<'_>]) -> Vec<Note> {
    let mut out = vec![line(
        if info.mandatory {
            format!("{}  (mandatory)", info.name)
        } else {
            info.name.clone()
        },
        Tone::Heading,
    )];
    out.push(line(
        if info.group.is_empty() {
            "top level of the vault".to_string()
        } else {
            format!("group {}", info.group.join("/"))
        },
        Tone::Quiet,
    ));
    match (&info.description, &info.header_problem) {
        (Some(text), _) => out.push(line(text.clone(), Tone::Plain)),
        (None, Some(problem)) => out.push(line(format!("header: {problem}"), Tone::Problem)),
        (None, None) => out.push(line(
            "the header of SKILL.md has no description",
            Tone::Problem,
        )),
    }
    let profiles = ops::profiles_of(&session.workspace, &info.name);
    if !profiles.is_empty() {
        out.push(line(
            format!("profiles: {}", profiles.join(", ")),
            Tone::Quiet,
        ));
    }
    out.push(line(files_line(&info.files), Tone::Quiet));
    if !info.untracked.is_empty() {
        out.push(line(
            format!(
                "not copied, git does not track: {}",
                info.untracked.join(", ")
            ),
            Tone::Change,
        ));
    }
    out.push(line("", Tone::Plain));
    if projects.is_empty() {
        out.push(line("Not installed in any project.", Tone::Quiet));
    } else {
        out.push(line("Projects", Tone::Heading));
        out.extend(projects.iter().map(|p| {
            line(
                format!("{}  {}", short_path(&p.project.to_string_lossy()), p.text),
                p.tone,
            )
        }));
    }
    out
}

fn files_line(files: &[String]) -> String {
    let shown = files
        .iter()
        .take(FILES_SHOWN)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    if files.len() > FILES_SHOWN {
        format!(
            "files ({}): {shown} and {} more",
            files.len(),
            files.len() - FILES_SHOWN
        )
    } else {
        format!("files ({}): {shown}", files.len())
    }
}
