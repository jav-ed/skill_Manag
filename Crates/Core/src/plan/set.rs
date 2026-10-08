//! Planning many targets at once.

use rayon::prelude::*;

use super::build::plan_skill;
use super::{PlanError, PlanKind, SkillPlan};
use crate::scan::Target;
use crate::vault::{Vault, VaultFiles};

/// The plan result of one target. A failure here stops this target only.
#[derive(Debug)]
pub struct PlanEntry {
    pub target: Target,
    pub result: Result<SkillPlan, PlanError>,
}

/// Plans for a list of targets, in the order the targets were given.
#[derive(Debug, Default)]
pub struct Plan {
    pub entries: Vec<PlanEntry>,
}

/// Totals over a plan, for summaries.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PlanCounts {
    pub create: usize,
    pub update: usize,
    pub unchanged: usize,
    pub failed: usize,
}

impl Plan {
    /// Compares every target with the vault in parallel. A target whose skill is not in the vault fails; it is never touched.
    pub fn for_targets(vault: &Vault, files: &VaultFiles, targets: Vec<Target>) -> Self {
        Self::build(vault, files, targets, false)
    }

    /// Like [`Plan::for_targets`], but a missing `.agents/skills` directory is created when the plan is applied.
    pub fn for_targets_creating(vault: &Vault, files: &VaultFiles, targets: Vec<Target>) -> Self {
        Self::build(vault, files, targets, true)
    }

    fn build(
        vault: &Vault,
        files: &VaultFiles,
        targets: Vec<Target>,
        create_skills_dir: bool,
    ) -> Self {
        let entries = targets
            .into_par_iter()
            .map(|target| {
                let result = match (vault.skills.get(&target.skill), files.get(&target.skill)) {
                    (Some(skill), Some(skill_files)) => {
                        plan_skill(&target, skill, skill_files, create_skills_dir)
                    }
                    _ => Err(PlanError::NotInVault {
                        skill: target.skill.clone(),
                    }),
                };
                PlanEntry { target, result }
            })
            .collect();
        Self { entries }
    }

    pub fn counts(&self) -> PlanCounts {
        let mut counts = PlanCounts::default();
        for entry in &self.entries {
            match &entry.result {
                Ok(plan) => match plan.kind {
                    PlanKind::Create => counts.create += 1,
                    PlanKind::Update => counts.update += 1,
                    PlanKind::Unchanged => counts.unchanged += 1,
                },
                Err(_) => counts.failed += 1,
            }
        }
        counts
    }
}
