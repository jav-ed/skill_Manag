//! What a sync or push is about to do, in the numbers and names the confirmation page shows.

use skillmirror_core::plan::{ChangeKind, Plan, PlanKind};

use crate::results::short_path;

/// A file the run would delete from a project copy because the vault does not have it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Removal {
    pub(crate) project: String,
    pub(crate) skill: String,
    pub(crate) path: String,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Preview {
    /// Skill folders that would be created.
    pub(crate) created: usize,
    /// Skill folders that would change.
    pub(crate) updated: usize,
    pub(crate) files_added: usize,
    pub(crate) files_changed: usize,
    pub(crate) removals: Vec<Removal>,
    /// Targets that cannot be written, with the reason.
    pub(crate) failures: Vec<String>,
}

impl Preview {
    pub(crate) fn of(plan: &Plan) -> Self {
        let mut preview = Self::default();
        for entry in &plan.entries {
            let skill_plan = match &entry.result {
                Ok(skill_plan) => skill_plan,
                Err(error) => {
                    preview.failures.push(format!(
                        "{} in {}: {error}",
                        entry.target.skill,
                        short_path(&entry.target.project.to_string_lossy())
                    ));
                    continue;
                }
            };
            match skill_plan.kind {
                PlanKind::Create => preview.created += 1,
                PlanKind::Update => preview.updated += 1,
                PlanKind::Unchanged => continue,
            }
            for change in &skill_plan.changes {
                match change.kind {
                    ChangeKind::Added => preview.files_added += 1,
                    ChangeKind::Modified | ChangeKind::ModeChanged => preview.files_changed += 1,
                }
            }
            for path in &skill_plan.removed {
                // A folder goes with the files named below it; only files are worth a line.
                let on_disk = skill_plan.target.path.join(path);
                if std::fs::symlink_metadata(&on_disk).is_ok_and(|m| m.is_dir()) {
                    continue;
                }
                preview.removals.push(Removal {
                    project: short_path(&skill_plan.target.project.to_string_lossy()),
                    skill: skill_plan.target.skill.clone(),
                    path: path.to_string_lossy().into_owned(),
                });
            }
        }
        preview
    }

    /// Whether anything would be written or removed.
    pub(crate) fn writes(&self) -> bool {
        self.created + self.updated > 0
    }
}
