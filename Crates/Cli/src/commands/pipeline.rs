//! The shared flow of every command that writes skills: show the plan, ask, apply, show the result.

use skillmirror_core::apply::{ApplyOptions, apply};
use skillmirror_core::events::ignore_events;
use skillmirror_core::plan::Plan;
use skillmirror_core::scan::ScanIssue;

use crate::exit::Exit;
use crate::output::{self, Row, RunJson, Summary, Tense};
use crate::report::CliError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    DryRun,
    Check,
    Apply,
}

impl Mode {
    fn name(self) -> &'static str {
        match self {
            Self::DryRun => "dry-run",
            Self::Check => "check",
            Self::Apply => "apply",
        }
    }
}

/// How one run was asked to behave.
// The flags mirror independent command-line switches.
#[allow(clippy::struct_excessive_bools, clippy::struct_field_names)]
#[derive(Clone, Copy)]
pub(super) struct Run<'a> {
    pub(super) command: &'a str,
    pub(super) dry_run: bool,
    pub(super) check: bool,
    pub(super) yes: bool,
    pub(super) json: bool,
    pub(super) all: bool,
    /// Said when the plan holds no target at all.
    pub(super) empty_message: &'a str,
}

impl Run<'_> {
    fn mode(&self) -> Mode {
        if self.check {
            Mode::Check
        } else if self.dry_run {
            Mode::DryRun
        } else {
            Mode::Apply
        }
    }
}

/// Plans are shown, confirmed and written here. `before_write` runs once, after the user agreed and before the first byte is written.
pub(super) fn execute(
    plan: Plan,
    issues: &[ScanIssue],
    run: &Run<'_>,
    before_write: &dyn Fn() -> Result<(), CliError>,
) -> Result<Exit, CliError> {
    let mode = run.mode();
    let rows: Vec<Row> = plan.entries.iter().map(Row::from_plan).collect();
    if rows.is_empty() {
        return finish_empty(run, mode);
    }
    let summary = Summary::of(&rows);
    if mode != Mode::Apply {
        show_plan(run, mode, &rows, issues)?;
        return Ok(exit_for_plan(mode, &summary));
    }
    if summary.changes() == 0 && summary.failed == 0 {
        show_plan(run, mode, &rows, issues)?;
        return Ok(Exit::Clean);
    }
    if !run.yes && !confirm_write(&rows, &summary)? {
        output::line("Cancelled, nothing was written.");
        return Ok(Exit::Clean);
    }
    before_write()?;
    let done = apply(plan, ApplyOptions::default(), &ignore_events)?;
    for left in done.applied.iter().filter_map(|a| a.leftover.as_ref()) {
        output::warn_line(&format!(
            "the update is done, but the old copy {} could not be removed ({}); delete it by hand",
            left.path.display(),
            left.reason
        ));
    }
    let rows: Vec<Row> = done.applied.iter().map(Row::from_applied).collect();
    show_done(run, &rows, issues)?;
    Ok(if done.failed() > 0 {
        Exit::Partial
    } else {
        Exit::Clean
    })
}

fn finish_empty(run: &Run<'_>, mode: Mode) -> Result<Exit, CliError> {
    if run.json {
        output::line(&RunJson::new(run.command, mode.name(), &[], &[]).render()?);
    } else {
        output::line(run.empty_message);
    }
    Ok(Exit::Clean)
}

fn exit_for_plan(mode: Mode, summary: &Summary) -> Exit {
    if summary.failed > 0 {
        Exit::Partial
    } else if mode == Mode::Check && summary.changes() > 0 {
        Exit::Drift
    } else {
        Exit::Clean
    }
}

fn show_plan(
    run: &Run<'_>,
    mode: Mode,
    rows: &[Row],
    issues: &[ScanIssue],
) -> Result<(), CliError> {
    if run.json {
        output::line(&RunJson::new(run.command, mode.name(), rows, issues).render()?);
        return Ok(());
    }
    output::print(&output::render_rows(rows, Tense::Plan, run.all));
    let note = if mode == Mode::DryRun { "dry run" } else { "" };
    output::print(&output::render_summary(
        &Summary::of(rows),
        Tense::Plan,
        note,
    ));
    Ok(())
}

fn show_done(run: &Run<'_>, rows: &[Row], issues: &[ScanIssue]) -> Result<(), CliError> {
    if run.json {
        output::line(&RunJson::new(run.command, Mode::Apply.name(), rows, issues).render()?);
        return Ok(());
    }
    output::print(&output::render_rows(rows, Tense::Done, run.all));
    output::print(&output::render_summary(&Summary::of(rows), Tense::Done, ""));
    Ok(())
}

/// Shows what will be written and asks. Without a terminal the answer must come from `--yes`.
fn confirm_write(rows: &[Row], summary: &Summary) -> Result<bool, CliError> {
    if !output::can_prompt() {
        return Err(CliError::usage(
            "refusing to write without confirmation",
            "pass --yes to apply, or --dry-run to preview",
        ));
    }
    output::print(&output::render_rows(rows, Tense::Plan, false));
    let question = format!(
        "Write {} skill folder(s) in {} project(s)?",
        summary.changes(),
        summary.projects
    );
    Ok(output::confirm(&question)?)
}
