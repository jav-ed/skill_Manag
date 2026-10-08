//! List: every skill folder installed in any project.

use crate::scan::{ScanIssue, ScanReport, Target, all_targets};
use crate::vault::Vault;

/// One installed skill folder.
#[derive(Debug, Clone)]
pub struct Installed {
    pub target: Target,
    /// Whether the vault has a skill of this name. `None` when no vault is configured.
    pub in_vault: Option<bool>,
}

#[derive(Debug, Default)]
pub struct InstalledSet {
    pub rows: Vec<Installed>,
    pub issues: Vec<ScanIssue>,
}

/// Lists installed skills in walk order. The vault is optional here, as it is for the Go tool.
pub fn installed(report: &ScanReport, vault: Option<&Vault>) -> InstalledSet {
    let set = all_targets(&report.skills_dirs);
    let rows = set
        .targets
        .into_iter()
        .map(|target| {
            let in_vault = vault.map(|v| v.skills.contains_key(&target.skill));
            Installed { target, in_vault }
        })
        .collect();
    InstalledSet {
        rows,
        issues: set.issues,
    }
}
