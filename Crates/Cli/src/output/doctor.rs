//! The text and the JSON document of `doctor`.

use serde::Serialize;
use skillmirror_core::ops::{DoctorReport, Finding, Severity};

use super::human::plural;
use super::style::{ERROR, MUTED, NAME, SUCCESS, WARNING};

fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Note => "note",
        Severity::Warning => "warning",
        Severity::Error => "error",
    }
}

#[derive(Debug, Serialize)]
struct FindingRow<'a> {
    severity: &'static str,
    check: &'a str,
    subject: Option<&'a str>,
    message: &'a str,
    hint: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct Summary {
    errors: usize,
    warnings: usize,
    notes: usize,
}

#[derive(Serialize)]
pub(crate) struct DoctorJson<'a> {
    checks: &'a [&'static str],
    findings: Vec<FindingRow<'a>>,
    summary: Summary,
}

impl<'a> DoctorJson<'a> {
    pub(crate) fn of(report: &'a DoctorReport) -> Self {
        Self {
            checks: &report.checks,
            findings: report
                .findings
                .iter()
                .map(|f| FindingRow {
                    severity: severity_name(f.severity),
                    check: f.check,
                    subject: f.subject.as_deref(),
                    message: &f.message,
                    hint: f.hint.as_deref(),
                })
                .collect(),
            summary: Summary {
                errors: report.count(Severity::Error),
                warnings: report.count(Severity::Warning),
                notes: report.count(Severity::Note),
            },
        }
    }

    pub(crate) fn render(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Every check in the order it ran: a tick when it found nothing, its findings when it did.
pub(crate) fn render_doctor(report: &DoctorReport) -> String {
    let mut out = String::new();
    let mut shown: Vec<&str> = Vec::new();
    for check in &report.checks {
        if shown.contains(check) {
            continue;
        }
        shown.push(check);
        let findings: Vec<&Finding> = report
            .findings
            .iter()
            .filter(|f| f.check == *check)
            .collect();
        if findings.is_empty() {
            putln!(out, "{SUCCESS}✓{SUCCESS:#} {check}");
            continue;
        }
        for finding in findings {
            out.push_str(&finding_text(finding));
        }
    }
    let (errors, warnings, notes) = (
        report.count(Severity::Error),
        report.count(Severity::Warning),
        report.count(Severity::Note),
    );
    let style = if errors > 0 {
        ERROR
    } else if warnings > 0 {
        WARNING
    } else {
        SUCCESS
    };
    let verdict = if errors + warnings == 0 {
        "nothing needs attention".to_string()
    } else {
        format!(
            "{}, {}",
            plural(errors, "error"),
            plural(warnings, "warning")
        )
    };
    let notes = if notes > 0 {
        format!(" ({})", plural(notes, "note"))
    } else {
        String::new()
    };
    put!(out, "\n{style}{verdict}{notes}{style:#}\n");
    out
}

fn finding_text(finding: &Finding) -> String {
    let (mark, style) = match finding.severity {
        Severity::Note => ("·", MUTED),
        Severity::Warning => ("!", WARNING),
        Severity::Error => ("✗", ERROR),
    };
    let mut out = String::new();
    let subject = finding
        .subject
        .as_deref()
        .map(|s| format!(" {NAME}{s}{NAME:#}"))
        .unwrap_or_default();
    putln!(
        out,
        "{style}{mark}{style:#} {}{subject}: {}",
        finding.check,
        finding.message
    );
    if let Some(hint) = &finding.hint {
        putln!(out, "    {MUTED}hint: {hint}{MUTED:#}");
    }
    out
}
