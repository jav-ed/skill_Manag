//! The text and the JSON document of `changes`.

use std::path::{Path, PathBuf};

use serde::Serialize;
use skillmirror_core::ops::{Edit, EditKind, Found, LocalChanges, LocalSkill, LocalState};

use crate::output::human::{pad, plural};
use crate::output::style::{ADDED, HEADER, MUTED, NAME, REMOVED, SUCCESS, WARNING};

/// How the project was chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Origin {
    /// With `--project`.
    Named,
    /// The current folder is the project.
    Here,
    /// The current folder lies inside it.
    Above,
    /// It lies below the current folder, the only one there.
    Below,
}

impl From<Found> for Origin {
    fn from(found: Found) -> Self {
        match found {
            Found::Here => Self::Here,
            Found::Above => Self::Above,
            Found::Below => Self::Below,
        }
    }
}

impl Origin {
    fn words(self) -> &'static str {
        match self {
            Self::Named => "given with --project",
            Self::Here => "this folder",
            Self::Above => "the project around this folder",
            Self::Below => "the only project below this folder",
        }
    }
}

#[derive(Debug, Serialize)]
struct EditRow {
    path: String,
    change: &'static str,
}

#[derive(Debug, Serialize)]
struct SkillRow {
    name: String,
    /// `same`, `edited`, `behind`, `new` or `unusable`.
    state: &'static str,
    /// The vault folders above the skill, such as `web/seo`; empty when it is not in the vault.
    group: String,
    #[serde(serialize_with = "crate::output::lossy::path")]
    path: PathBuf,
    /// For `behind`: how many changes the vault made since (0: the newest is not committed yet).
    behind_by: Option<usize>,
    /// For `unusable`: why.
    reason: Option<String>,
    edits: Vec<EditRow>,
}

#[derive(Debug, Default, Clone, Copy, Serialize)]
pub(crate) struct Counts {
    pub(crate) edited: usize,
    behind: usize,
    new: usize,
    unusable: usize,
    same: usize,
}

impl Counts {
    pub(crate) fn of(changes: &LocalChanges) -> Self {
        let mut counts = Self::default();
        for skill in &changes.skills {
            match skill.state {
                LocalState::Same => counts.same += 1,
                LocalState::Edited(_) => counts.edited += 1,
                LocalState::Behind { .. } => counts.behind += 1,
                LocalState::New => counts.new += 1,
                LocalState::Unusable(_) => counts.unusable += 1,
            }
        }
        counts
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct ChangesJson {
    #[serde(serialize_with = "crate::output::lossy::path")]
    project: PathBuf,
    found: Origin,
    skills: Vec<SkillRow>,
    summary: Counts,
    /// What could not be read while listing the skills.
    problems: Vec<String>,
}

impl ChangesJson {
    pub(crate) fn of(changes: &LocalChanges, origin: Origin) -> Self {
        Self {
            project: changes.project.clone(),
            found: origin,
            skills: changes.skills.iter().map(skill_row).collect(),
            summary: Counts::of(changes),
            problems: changes
                .issues
                .iter()
                .map(|i| format!("{}: {}", i.path.display(), i.message))
                .collect(),
        }
    }

    pub(crate) fn render(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

pub(in crate::output) fn kind_word(kind: EditKind) -> &'static str {
    match kind {
        EditKind::Added => "added",
        EditKind::Modified => "modified",
        EditKind::Removed => "removed",
    }
}

fn skill_row(skill: &LocalSkill) -> SkillRow {
    let (state, behind_by, reason, edits) = match &skill.state {
        LocalState::Same => ("same", None, None, Vec::new()),
        LocalState::Edited(edits) => (
            "edited",
            None,
            None,
            edits
                .iter()
                .map(|e| EditRow {
                    path: e.rel.display().to_string(),
                    change: kind_word(e.kind),
                })
                .collect(),
        ),
        LocalState::Behind { commits } => ("behind", Some(*commits), None, Vec::new()),
        LocalState::New => ("new", None, None, Vec::new()),
        LocalState::Unusable(why) => ("unusable", None, Some(why.clone()), Vec::new()),
    };
    SkillRow {
        name: skill.name.clone(),
        state,
        group: skill.group.join("/"),
        path: skill.path.clone(),
        behind_by,
        reason,
        edits,
    }
}

/// At most this many files are named in a line; the rest is counted.
const NAMED: usize = 3;

/// The files of an edit in one line: `SKILL.md modified, notes.md added, 2 more`.
pub(in crate::output) fn edits_words(edits: &[Edit]) -> String {
    let mut parts: Vec<String> = edits
        .iter()
        .take(NAMED)
        .map(|e| format!("{} {}", e.rel.display(), kind_word(e.kind)))
        .collect();
    if edits.len() > NAMED {
        parts.push(format!("{} more", edits.len() - NAMED));
    }
    parts.join(", ")
}

pub(crate) fn render_changes(
    changes: &LocalChanges,
    origin: Origin,
    cwd: &Path,
    show_all: bool,
) -> String {
    let mut out = String::new();
    putln!(
        out,
        "{HEADER}● {}{HEADER:#} {MUTED}({}){MUTED:#}",
        changes.project.display(),
        origin.words()
    );
    if origin != Origin::Here && origin != Origin::Named {
        putln!(out, "  {MUTED}looked from {}{MUTED:#}", cwd.display());
    }
    for skill in &changes.skills {
        let group = if skill.group.is_empty() {
            String::new()
        } else {
            format!("{}/", skill.group.join("/"))
        };
        let name = pad(&skill.name);
        match &skill.state {
            LocalState::Same if show_all => putln!(
                out,
                "  {SUCCESS}✓{SUCCESS:#} {NAME}{name}{NAME:#} {MUTED}same as the vault's{MUTED:#}"
            ),
            LocalState::Same => {}
            LocalState::Edited(edits) => putln!(
                out,
                "  {ADDED}~{ADDED:#} {NAME}{name}{NAME:#} edited here ({}) {MUTED}vault: {group}{}{MUTED:#}",
                edits_words(edits),
                skill.name
            ),
            LocalState::Behind { commits } => putln!(
                out,
                "  {WARNING}↓{WARNING:#} {NAME}{name}{NAME:#} {MUTED}an older version of the vault's, {} since (`skillmirror sync` updates it){MUTED:#}",
                if *commits == 0 {
                    "its newest change is not committed yet".to_string()
                } else {
                    plural(*commits, "change")
                }
            ),
            LocalState::New => putln!(
                out,
                "  {WARNING}+{WARNING:#} {NAME}{name}{NAME:#} {MUTED}not in the vault (`skillmirror adopt {} --from {}` adds it){MUTED:#}",
                skill.name,
                changes.project.display()
            ),
            LocalState::Unusable(why) => putln!(
                out,
                "  {REMOVED}!{REMOVED:#} {NAME}{name}{NAME:#} {MUTED}cannot be compared: {why}{MUTED:#}"
            ),
        }
    }
    for issue in &changes.issues {
        putln!(
            out,
            "  {REMOVED}!{REMOVED:#} {MUTED}{}: {}{MUTED:#}",
            issue.path.display(),
            issue.message
        );
    }
    let counts = Counts::of(changes);
    putln!(
        out,
        "\n{} edited, {} behind, {} new, {} same",
        counts.edited,
        counts.behind,
        counts.new,
        counts.same
    );
    if counts.edited > 0 {
        putln!(
            out,
            "{MUTED}`skillmirror promote --all` puts the edited ones into the vault; add --sync to update the other projects too.{MUTED:#}"
        );
    }
    out
}
