//! `report`: write the static HTML report.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use skillmirror_core::backup::now_utc;
use skillmirror_core::ops;
use skillmirror_web::{MARKER, render_report};

use super::context::{Context, open};
use crate::args::{Cli, ReportArgs};
use crate::exit::Exit;
use crate::output;
use crate::report::CliError;

pub(super) fn run(cli: &Cli, args: &ReportArgs) -> Result<Exit, CliError> {
    let Context { workspace, report } = open(cli, args.output == Path::new("-"))?;
    let data = ops::report_data(&workspace, &report)?;
    let html = render_report(&data, &now_utc());
    if args.output == Path::new("-") {
        output::print(&html);
        return Ok(Exit::Clean);
    }
    let path = write_report(&args.output, &html)?;
    output::line(&format!(
        "Wrote the report to {} ({} skills, {} projects).",
        path.display(),
        data.skills.len(),
        data.projects.len()
    ));
    if args.open {
        open_in_browser(&path);
    }
    Ok(Exit::Clean)
}

/// Writes the file next to its place and renames it over, so a reader never sees half a report. A file
/// that is already there is replaced only if it is a report of this tool.
fn write_report(target: &Path, html: &str) -> Result<PathBuf, CliError> {
    let path = std::path::absolute(target)?;
    match fs_err::read(&path) {
        Ok(existing) => {
            let head = existing.get(..200).unwrap_or(&existing);
            if !String::from_utf8_lossy(head).contains(MARKER) {
                return Err(CliError::refused(
                    format!("{} exists and is not a skillmirror report", path.display()),
                    "choose another name with --output, or remove the file yourself",
                ));
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut staged = tempfile::Builder::new()
        .prefix(".skillmirror-report-")
        .tempfile_in(dir)?;
    staged.write_all(html.as_bytes())?;
    staged.flush()?;
    staged.persist(&path).map_err(|e| e.error)?;
    Ok(path)
}

/// Opening a browser is best effort: a missing `xdg-open` must not turn a written report into a failure.
fn open_in_browser(path: &Path) {
    let spawned = Command::new("xdg-open")
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    match spawned {
        Ok(mut child) => {
            std::thread::spawn(move || drop(child.wait()));
        }
        Err(e) => output::warn_line(&format!("could not open a browser: {e}")),
    }
}
