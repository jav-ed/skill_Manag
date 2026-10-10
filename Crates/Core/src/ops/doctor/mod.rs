//! Doctor: look at the vault, the configuration and the machine, and say what is wrong before a sync
//! trips over it. It reads, never writes, and it keeps going when a layer is broken: a broken config is a
//! finding, not the end of the report.

mod agents_text;
mod env;
mod skills;
mod vault;

#[cfg(test)]
pub(in crate::ops) use env::unsupported_magic;

use crate::Hint;
use crate::config::{ConfigError, Dirs, EnvOverrides, Flags, Settings};

/// How much a finding matters. `Error` means sync or push would fail or do the wrong thing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Worth knowing, nothing to fix.
    Note,
    /// Something is off but the tool still works.
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub severity: Severity,
    /// The check that found it, such as `skill-files`.
    pub check: &'static str,
    /// The skill, folder or file it is about.
    pub subject: Option<String>,
    pub message: String,
    pub hint: Option<String>,
}

#[derive(Debug, Default)]
pub struct Report {
    /// Every check that ran, in order, so a clean report can say what it looked at.
    pub checks: Vec<&'static str>,
    pub findings: Vec<Finding>,
}

impl Report {
    pub(super) fn ran(&mut self, check: &'static str) {
        self.checks.push(check);
    }

    pub(super) fn add(
        &mut self,
        severity: Severity,
        check: &'static str,
        subject: Option<String>,
        message: impl Into<String>,
        hint: Option<String>,
    ) {
        self.findings.push(Finding {
            severity,
            check,
            subject,
            message: message.into(),
            hint,
        });
    }

    pub fn count(&self, severity: Severity) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == severity)
            .count()
    }

    /// The most serious severity found, if any.
    pub fn worst(&self) -> Option<Severity> {
        self.findings.iter().map(|f| f.severity).max()
    }
}

/// Runs every check. `env` is the result of reading the environment, so that a legacy variable becomes a
/// finding instead of stopping the command.
pub fn doctor(flags: &Flags, env: Result<EnvOverrides, ConfigError>, dirs: &Dirs) -> Report {
    let mut report = Report::default();
    env::machine(&mut report, flags, dirs);

    report.ran("config");
    let env = match env {
        Ok(env) => env,
        Err(error) => {
            report.add(
                Severity::Error,
                "config",
                None,
                error.to_string(),
                error.hint(),
            );
            return report;
        }
    };
    let settings = match Settings::load(flags, &env, dirs) {
        Ok(settings) => settings,
        Err(error) => {
            report.add(
                Severity::Error,
                "config",
                None,
                error.to_string(),
                error.hint(),
            );
            return report;
        }
    };
    vault::check(&mut report, &settings);
    env::root(&mut report, &settings);
    report
}
