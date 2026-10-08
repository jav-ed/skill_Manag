//! `bridge`: link the other agent folders named in the vault config to `.agents/skills`.

use std::path::Path;

use skillmirror_core::ops::{self, Bridge, BridgeState};

use super::context::{Context, open};
use crate::args::{BridgeArgs, Cli};
use crate::exit::Exit;
use crate::output::{self, BridgeJson, BridgeRow, BridgeStatus};
use crate::report::CliError;

pub(super) fn run(cli: &Cli, args: &BridgeArgs) -> Result<Exit, CliError> {
    let Context { workspace, report } = open(cli, args.json)?;
    let targets = &workspace.settings.config().targets;
    if targets.is_empty() {
        let message = "No targets in the vault config. Add a line such as `targets: [claude]` to <vault>/config.yaml.";
        if args.json {
            output::line(&BridgeJson::new(args.dry_run, &[]).render()?);
        } else {
            output::line(message);
        }
        return Ok(Exit::Clean);
    }
    let bridges = ops::plan_bridges(targets, &report.skills_dirs);
    let missing = bridges
        .iter()
        .filter(|b| b.state == BridgeState::Missing)
        .count();
    if !args.dry_run && missing > 0 && !args.yes && !confirm(missing)? {
        output::line("Cancelled, nothing was linked.");
        return Ok(Exit::Clean);
    }
    let rows: Vec<BridgeRow> = bridges.iter().map(|b| row(b, args.dry_run)).collect();
    if args.json {
        output::line(&BridgeJson::new(args.dry_run, &rows).render()?);
    } else {
        output::print(&output::render_bridges(&rows, args.dry_run, args.all));
    }
    let trouble = rows.iter().any(|r| {
        matches!(
            r.status,
            BridgeStatus::Blocked { .. } | BridgeStatus::Failed { .. }
        )
    });
    // Like `sync --dry-run --check`: something to link is drift (1); something in the way, or a link that
    // could not be made, is a partial result (4).
    Ok(if trouble {
        Exit::Partial
    } else if args.dry_run && missing > 0 {
        Exit::Drift
    } else {
        Exit::Clean
    })
}

fn confirm(missing: usize) -> Result<bool, CliError> {
    if !output::can_prompt() {
        return Err(CliError::usage(
            "refusing to link without confirmation",
            "pass --yes to link, or --dry-run to preview",
        ));
    }
    Ok(output::confirm(&format!("Link {missing} bridge(s)?"))?)
}

/// Links the missing bridges of one project, as `add` and `init` do after they wrote skills there.
pub(super) fn link_project(targets: &[String], project: &Path) -> Vec<BridgeRow> {
    ops::plan_project_bridges(targets, project)
        .iter()
        .map(|b| row(b, false))
        .collect()
}

fn row(bridge: &Bridge, dry_run: bool) -> BridgeRow {
    let status = match (&bridge.state, bridge.problem()) {
        (BridgeState::InPlace, _) => BridgeStatus::InPlace,
        (BridgeState::Missing, _) if dry_run => BridgeStatus::WouldLink,
        (BridgeState::Missing, _) => match ops::create_bridge(bridge) {
            Ok(()) => BridgeStatus::Linked,
            Err(error) => BridgeStatus::Failed {
                message: error.to_string(),
            },
        },
        (_, Some((message, hint))) => BridgeStatus::Blocked { message, hint },
        (_, None) => BridgeStatus::InPlace,
    };
    BridgeRow {
        project: bridge.project.clone(),
        target: bridge.name.clone(),
        link: bridge.link.clone(),
        status,
    }
}
