//! Building the new copy of a skill next to its destination.

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use super::ApplyError;
use crate::plan::SkillPlan;

/// Copies every source file of `plan` into the new directory `stage`, with the vault's permission bits.
pub(super) fn build(plan: &SkillPlan, stage: &Path) -> Result<(), ApplyError> {
    fs_err::create_dir(stage)?;
    for source in &plan.sources {
        let to = stage.join(&source.rel);
        if let Some(parent) = to.parent() {
            fs_err::create_dir_all(parent)?;
        }
        fs_err::copy(plan.source_dir.join(&source.rel), &to)?;
        fs_err::set_permissions(&to, std::fs::Permissions::from_mode(source.mode))?;
    }
    Ok(())
}
