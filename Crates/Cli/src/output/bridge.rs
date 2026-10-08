//! The text and the JSON document of `bridge`.

use std::path::PathBuf;

use serde::Serialize;

use super::human::{pad, plural};
use super::style::{ERROR, HEADER, MUTED, NAME, SUCCESS, WARNING};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum BridgeStatus {
    WouldLink,
    Linked,
    InPlace,
    /// Something is in the way and the user has to act.
    Blocked {
        message: String,
        hint: String,
    },
    /// The link could not be made.
    Failed {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BridgeRow {
    #[serde(serialize_with = "super::lossy::path")]
    pub(crate) project: PathBuf,
    pub(crate) target: String,
    #[serde(serialize_with = "super::lossy::path")]
    pub(crate) link: PathBuf,
    pub(crate) status: BridgeStatus,
}

impl BridgeRow {
    /// The link as it is written inside its project, such as `.claude/skills`.
    fn short_link(&self) -> &std::path::Path {
        self.link.strip_prefix(&self.project).unwrap_or(&self.link)
    }
}

#[derive(Serialize)]
pub(crate) struct BridgeJson<'a> {
    dry_run: bool,
    bridges: &'a [BridgeRow],
}

impl<'a> BridgeJson<'a> {
    pub(crate) fn new(dry_run: bool, bridges: &'a [BridgeRow]) -> Self {
        Self { dry_run, bridges }
    }

    pub(crate) fn render(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Projects with something to say are listed; the ones whose bridge is in place only count in the summary,
/// unless `show_all`.
pub(crate) fn render_bridges(rows: &[BridgeRow], dry_run: bool, show_all: bool) -> String {
    let mut out = String::new();
    let mut current: Option<&std::path::Path> = None;
    for row in rows {
        if matches!(row.status, BridgeStatus::InPlace) && !show_all {
            continue;
        }
        if current != Some(row.project.as_path()) {
            current = Some(row.project.as_path());
            putln!(out, "\n{HEADER}● {}{HEADER:#}", row.project.display());
        }
        row_lines(&mut out, row);
    }
    let count =
        |wanted: fn(&BridgeStatus) -> bool| rows.iter().filter(|r| wanted(&r.status)).count();
    let linked = count(|s| matches!(s, BridgeStatus::Linked | BridgeStatus::WouldLink));
    let in_place = count(|s| matches!(s, BridgeStatus::InPlace));
    let trouble = count(|s| {
        matches!(
            s,
            BridgeStatus::Blocked { .. } | BridgeStatus::Failed { .. }
        )
    });
    let mut parts = vec![format!(
        "{} {}",
        plural(linked, "bridge"),
        if dry_run { "to link" } else { "linked" }
    )];
    parts.push(format!("{in_place} already in place"));
    if trouble > 0 {
        parts.push(format!("{trouble} need attention"));
    }
    let style = if trouble > 0 { WARNING } else { SUCCESS };
    let note = if dry_run { " (dry run)" } else { "" };
    put!(out, "\n{style}{}{note}{style:#}\n", parts.join(", "));
    out
}

/// The lines of one bridge, indented under its project.
fn row_lines(out: &mut String, row: &BridgeRow) {
    let name = pad(&row.target);
    match &row.status {
        BridgeStatus::WouldLink => putln!(
            out,
            "  {WARNING}+{WARNING:#} {NAME}{name}{NAME:#} {MUTED}would link {}{MUTED:#}",
            row.short_link().display()
        ),
        BridgeStatus::Linked => putln!(
            out,
            "  {SUCCESS}✓{SUCCESS:#} {NAME}{name}{NAME:#} {MUTED}linked {}{MUTED:#}",
            row.short_link().display()
        ),
        BridgeStatus::InPlace => putln!(
            out,
            "  {SUCCESS}✓{SUCCESS:#} {NAME}{name}{NAME:#} {MUTED}already in place{MUTED:#}"
        ),
        BridgeStatus::Blocked { message, hint } => {
            putln!(out, "  {ERROR}✗{ERROR:#} {NAME}{name}{NAME:#} {message}");
            putln!(out, "      {MUTED}hint: {hint}{MUTED:#}");
        }
        BridgeStatus::Failed { message } => {
            putln!(out, "  {ERROR}✗{ERROR:#} {NAME}{name}{NAME:#} {message}");
        }
    }
}

/// What `add` and `init` say after they linked a project: only what was done or is in the way.
pub(crate) fn render_bridge_notes(rows: &[BridgeRow]) -> String {
    let mut out = String::new();
    for row in rows {
        if !matches!(row.status, BridgeStatus::InPlace) {
            row_lines(&mut out, row);
        }
    }
    out
}
