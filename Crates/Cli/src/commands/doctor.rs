//! `doctor`: look at the vault, the configuration and the machine, read-only.

use skillmirror_core::config::{Dirs, EnvOverrides};
use skillmirror_core::ops::{self, Severity};

use super::context::flags;
use crate::args::{Cli, DoctorArgs};
use crate::exit::Exit;
use crate::output::{self, DoctorJson};
use crate::report::CliError;

pub(super) fn run(cli: &Cli, args: &DoctorArgs) -> Result<Exit, CliError> {
    let dirs = Dirs::from_env()?;
    // A legacy variable is something `doctor` has to talk about, so it reads the environment itself.
    let report = ops::doctor(&flags(cli), EnvOverrides::from_env(), &dirs);
    if args.json {
        output::line(&DoctorJson::of(&report).render()?);
    } else {
        output::print(&output::render_doctor(&report));
    }
    Ok(match report.worst() {
        Some(Severity::Error) => Exit::HardError,
        Some(Severity::Warning) => Exit::Drift,
        Some(Severity::Note) | None => Exit::Clean,
    })
}
