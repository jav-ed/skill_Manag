//! A vault and its configuration, opened and checked once per run.

use crate::Result;
use crate::config::Settings;
use crate::events::{Event, Observer};
use crate::scan::{ScanReport, scan_with_progress};
use crate::vault::{Vault, VaultFiles, discover, read_files};

/// Everything read from the vault side: settings, discovered skills and git's file lists.
#[derive(Debug)]
pub struct Workspace {
    pub settings: Settings,
    pub vault: Vault,
    pub files: VaultFiles,
}

impl Workspace {
    /// Discovers the vault skills and reads their git file lists. The vault must be set and a git repository.
    pub fn open(settings: Settings) -> Result<Self> {
        let vault = discover(&settings.vault()?.value)?;
        let files = read_files(&vault)?;
        Ok(Self {
            settings,
            vault,
            files,
        })
    }

    /// Scans the configured root for `.agents/skills` directories.
    pub fn scan(&self, observer: Observer<'_>) -> Result<ScanReport> {
        let report = scan_with_progress(
            &self.settings.root()?.value,
            &self.settings.scan_options(),
            &|counts| {
                observer(Event::ScanProgress {
                    directories: counts.directories,
                    projects: counts.projects,
                });
            },
        )?;
        observer(Event::ScanFinished {
            skills_dirs: report.skills_dirs.len(),
            issues: report.issues.len(),
        });
        Ok(report)
    }
}
