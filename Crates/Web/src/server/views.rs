//! The rows the pages and the plan answers are made of, taken from a snapshot.

use std::collections::BTreeMap;
use std::path::Path;

use super::state::Snapshot;

/// A project as the pages name it: the path below the scan root, or the last two components.
pub(crate) fn project_label(root: &Path, project: &Path) -> String {
    match project.strip_prefix(root) {
        Ok(below) if !below.as_os_str().is_empty() => below.display().to_string(),
        _ => {
            let parts: Vec<String> = project
                .components()
                .filter_map(|c| match c {
                    std::path::Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
                    _ => None,
                })
                .collect();
            let from = parts.len().saturating_sub(2);
            parts.get(from..).unwrap_or_default().join("/")
        }
    }
}

/// The scan root, or nothing when the snapshot has none.
pub(crate) fn root_of(snapshot: &Snapshot) -> &Path {
    snapshot
        .workspace
        .settings
        .root()
        .map_or(Path::new(""), |r| r.value.as_path())
}

/// One skill on the sync or push page.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct SkillRow {
    pub(crate) name: String,
    pub(crate) group: String,
    pub(crate) mandatory: bool,
    /// Projects that have the skill and that equal the vault.
    pub(crate) current: usize,
    /// Projects whose copy differs.
    pub(crate) outdated: usize,
    /// Projects that should have it (mandatory) and do not.
    pub(crate) missing: usize,
    /// Projects where the comparison failed.
    pub(crate) problems: usize,
}

impl SkillRow {
    pub(crate) fn installed(&self) -> usize {
        self.current + self.outdated
    }

    /// Whether a sync or push would write something for this skill.
    pub(crate) fn changes(&self) -> usize {
        self.outdated + self.missing
    }
}

/// The row of a skill, made when the first project mentions it.
fn row_of<'a>(
    snapshot: &Snapshot,
    rows: &'a mut BTreeMap<String, SkillRow>,
    name: &str,
) -> &'a mut SkillRow {
    rows.entry(name.to_string()).or_insert_with(|| SkillRow {
        name: name.to_string(),
        group: snapshot
            .workspace
            .vault
            .skills
            .get(name)
            .map(|s| s.group.join("/"))
            .unwrap_or_default(),
        mandatory: snapshot
            .workspace
            .settings
            .mandatory()
            .iter()
            .any(|m| m == name),
        ..SkillRow::default()
    })
}

fn rows(snapshot: &Snapshot, keep: impl Fn(&SkillRow) -> bool) -> Vec<SkillRow> {
    let mut by_name: BTreeMap<String, SkillRow> = BTreeMap::new();
    for project in &snapshot.status.projects {
        for name in &project.up_to_date {
            row_of(snapshot, &mut by_name, name).current += 1;
        }
        for outdated in &project.outdated {
            row_of(snapshot, &mut by_name, &outdated.skill).outdated += 1;
        }
        for name in &project.missing_mandatory {
            row_of(snapshot, &mut by_name, name).missing += 1;
        }
        for problem in &project.failed {
            row_of(snapshot, &mut by_name, &problem.skill).problems += 1;
        }
    }
    let mut out: Vec<SkillRow> = by_name.into_values().filter(|r| keep(r)).collect();
    out.sort_by(|a, b| a.group.cmp(&b.group).then_with(|| a.name.cmp(&b.name)));
    out
}

/// The skills a sync can refresh: the ones the vault has and some project has.
pub(crate) fn sync_rows(snapshot: &Snapshot) -> Vec<SkillRow> {
    rows(snapshot, |r| {
        r.installed() > 0 && snapshot_has(snapshot, &r.name)
    })
}

/// The mandatory skills, which a push puts everywhere.
pub(crate) fn push_rows(snapshot: &Snapshot) -> Vec<SkillRow> {
    rows(snapshot, |r| r.mandatory)
}

fn snapshot_has(snapshot: &Snapshot, name: &str) -> bool {
    snapshot.workspace.vault.skills.contains_key(name)
}
