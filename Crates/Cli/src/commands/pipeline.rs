//! The shared flow of every command that writes skills: show the plan, ask, apply, show the result.

use skillmirror_core::apply::{ApplyOptions, Outcome, apply};
use skillmirror_core::backup::RunKind;
use skillmirror_core::events::ignore_events;
use skillmirror_core::plan::Plan;
use skillmirror_core::scan::ScanIssue;

use super::backup;
use crate::exit::Exit;
use crate::output::{self, AgentRow, BridgeRow, Row, RunJson, Summary, Tense};
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
    pub(super) kind: RunKind,
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

/// What a call of [`execute`] came to.
pub(super) struct Executed {
    pub(super) exit: Exit,
    /// Skill folders that were created or updated.
    pub(super) wrote: usize,
}

impl Executed {
    fn nothing(exit: Exit) -> Self {
        Self { exit, wrote: 0 }
    }
}

/// The AGENTS.md that `init` writes into the new project, in the same confirmed run and backup run.
pub(super) struct AgentsHook<'a> {
    /// What is planned for it, shown with the skills.
    pub(super) planned: AgentRow,
    /// Where the text comes from, for the output.
    pub(super) source: String,
    /// Writes it after the skills, into the run that kept what they replaced; the number is how many
    /// slots of that run the skills used.
    pub(super) write: &'a dyn Fn(&skillmirror_core::backup::Run, usize) -> Option<AgentRow>,
}

/// What a command does around the write.
pub(super) struct Hooks<'a> {
    /// Runs once, after the user agreed and before the first byte is written.
    pub(super) before_write: &'a dyn Fn() -> Result<(), CliError>,
    /// Runs after a write that created or updated skill folders, with how many; what it returns is reported.
    pub(super) after_write: &'a dyn Fn(usize) -> Vec<BridgeRow>,
    /// The AGENTS.md written along with the skills; only `init` has one.
    pub(super) agents: Option<AgentsHook<'a>>,
}

impl Hooks<'_> {
    pub(super) const NONE: Hooks<'static> = Hooks {
        before_write: &|| Ok(()),
        after_write: &|_| Vec::new(),
        agents: None,
    };
}

/// Plans are shown, confirmed and written here.
pub(super) fn execute(
    plan: Plan,
    issues: &[ScanIssue],
    run: &Run<'_>,
    hooks: &Hooks<'_>,
) -> Result<Executed, CliError> {
    let mode = run.mode();
    let rows: Vec<Row> = plan.entries.iter().map(Row::from_plan).collect();
    if rows.is_empty() {
        return finish_empty(run, mode).map(Executed::nothing);
    }
    let summary = Summary::of(&rows);
    // Nothing to write, whether the plan is clean or only holds failures: show it like a dry run, never ask,
    // and never run the hooks (`init` would create a directory for nothing).
    let planned_agents = hooks.agents.as_ref();
    if mode != Mode::Apply || summary.changes() == 0 {
        show_plan(run, mode, &rows, issues, planned_agents)?;
        return Ok(Executed::nothing(exit_for_plan(mode, &summary)));
    }
    if !run.yes && !confirm_write(&rows, &summary, run.json, planned_agents)? {
        output::line("Cancelled, nothing was written.");
        return Ok(Executed::nothing(Exit::Clean));
    }
    let (backups, backup_run) = backup::begin(run.kind)?;
    (hooks.before_write)()?;
    let options = ApplyOptions {
        backup: Some(&backup_run),
        ..ApplyOptions::default()
    };
    let slots = plan.entries.len();
    let done = apply(plan, options, &ignore_events)?;
    let wrote = done
        .applied
        .iter()
        .filter(|a| matches!(a.outcome, Outcome::Created | Outcome::Updated))
        .count();
    // Nothing installed means the project is taken back, so no file goes into it.
    let agents = hooks
        .agents
        .as_ref()
        .filter(|_| wrote > 0)
        .and_then(|hook| (hook.write)(&backup_run, slots).map(|row| (hook.source.as_str(), row)));
    let finished = backup_run.finish(&backups);
    for left in done.applied.iter().filter_map(|a| a.leftover.as_ref()) {
        output::warn_line(&format!(
            "the update is done, but the old copy {} could not be removed ({}); delete it by hand",
            left.path.display(),
            left.reason
        ));
    }
    let rows: Vec<Row> = done.applied.iter().map(Row::from_applied).collect();
    let bridges = if wrote > 0 {
        (hooks.after_write)(wrote)
    } else {
        Vec::new()
    };
    let agents_row = agents.as_ref().map(|(source, row)| (*source, row));
    show_done(
        run,
        &rows,
        issues,
        backup::saved(&finished),
        &bridges,
        agents_row,
    )?;
    backup::announce(&finished, run.json);
    let agents_failed = agents
        .as_ref()
        .is_some_and(|(_, row)| row.action == "failed");
    let exit = if done.failed() > 0 || agents_failed {
        Exit::Partial
    } else {
        Exit::Clean
    };
    Ok(Executed { exit, wrote })
}

fn finish_empty(run: &Run<'_>, mode: Mode) -> Result<Exit, CliError> {
    if run.json {
        output::line(&RunJson::new(run.kind.name(), mode.name(), &[], &[]).render()?);
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
    agents: Option<&AgentsHook<'_>>,
) -> Result<(), CliError> {
    if run.json {
        let document = RunJson::new(run.kind.name(), mode.name(), rows, issues)
            .with_agents(agents.map(|a| &a.planned));
        output::line(&document.render()?);
        return Ok(());
    }
    output::print(&output::render_rows(rows, Tense::Plan, run.all));
    if let Some(hook) = agents {
        output::print(&output::render_agents_line(&hook.planned, &hook.source));
    }
    let note = if mode == Mode::DryRun { "dry run" } else { "" };
    output::print(&output::render_summary(
        &Summary::of(rows),
        Tense::Plan,
        note,
    ));
    Ok(())
}

fn show_done(
    run: &Run<'_>,
    rows: &[Row],
    issues: &[ScanIssue],
    backup: Option<&str>,
    bridges: &[BridgeRow],
    agents: Option<(&str, &AgentRow)>,
) -> Result<(), CliError> {
    if run.json {
        let document = RunJson::new(run.kind.name(), Mode::Apply.name(), rows, issues);
        output::line(
            &document
                .with_backup(backup)
                .with_bridges(bridges)
                .with_agents(agents.map(|(_, row)| row))
                .render()?,
        );
        return Ok(());
    }
    output::print(&output::render_rows(rows, Tense::Done, run.all));
    if let Some((source, row)) = agents {
        output::print(&output::render_agents_line(row, source));
    }
    output::print(&output::render_summary(&Summary::of(rows), Tense::Done, ""));
    output::print(&output::render_bridge_notes(bridges));
    Ok(())
}

/// Shows what will be written and asks. Without a terminal the answer must come from `--yes`.
fn confirm_write(
    rows: &[Row],
    summary: &Summary,
    json: bool,
    agents: Option<&AgentsHook<'_>>,
) -> Result<bool, CliError> {
    // The rows and the question would land on stdout in front of the document that `--json` promises.
    if json {
        return Err(CliError::usage(
            "--json cannot ask for confirmation",
            "pass --yes to apply, or --dry-run to preview",
        ));
    }
    if !output::can_prompt() {
        return Err(CliError::usage(
            "refusing to write without confirmation",
            "pass --yes to apply, or --dry-run to preview",
        ));
    }
    output::print(&output::render_rows(rows, Tense::Plan, false));
    if let Some(hook) = agents {
        output::print(&output::render_agents_line(&hook.planned, &hook.source));
    }
    let and_file = if agents.is_some() {
        " and AGENTS.md"
    } else {
        ""
    };
    let question = format!(
        "Write {} skill folder(s){and_file} in {} project(s)?",
        summary.changes(),
        summary.projects
    );
    Ok(output::confirm(&question)?)
}
