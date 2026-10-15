//! `web`: the local server.

use skillmirror_core::config::{Settings, VaultConfig};

use super::context::load_settings;
use crate::args::{Cli, WebArgs};
use crate::exit::Exit;
use crate::report::CliError;

const NO_WEB_VIEW: &str =
    "this build of skillmirror has no web view: it is off unless it is built in";
const BUILD_IN: &str = "build it in with `cargo install --path Crates/Cli --locked --features web` (or `just install-web`)";

/// What the user has to do before `web` can start: the switch in the config, the build, or both.
fn refuse_unless_ready(settings: &Settings) -> Result<(), CliError> {
    // The switch lives in the vault config, so without a vault there is nothing to turn on yet.
    let vault = settings.vault()?;
    let switched_on = settings.config().web;
    let built_in = cfg!(feature = "web");
    let turn_on = format!(
        "turn it on with `skillmirror config web on` (sets `web: true` in {})",
        VaultConfig::path_in(&vault.value).display()
    );
    match (switched_on, built_in) {
        (true, true) => Ok(()),
        (false, true) => Err(CliError::usage(
            "the web view is switched off in the config",
            turn_on,
        )),
        (true, false) => Err(CliError::usage(NO_WEB_VIEW, BUILD_IN)),
        (false, false) => Err(CliError::usage(
            "the web view is switched off in the config, and this build of skillmirror has no web view",
            format!("{turn_on}; {BUILD_IN}"),
        )),
    }
}

pub(super) fn run(cli: &Cli, args: &WebArgs) -> Result<Exit, CliError> {
    let settings = load_settings(cli)?;
    refuse_unless_ready(&settings)?;
    serve_web(cli, args, &settings)
}

#[cfg(not(feature = "web"))]
fn serve_web(_cli: &Cli, _args: &WebArgs, _settings: &Settings) -> Result<Exit, CliError> {
    // `refuse_unless_ready` has already said so; this keeps the build without the server free of a panic.
    Err(CliError::usage(NO_WEB_VIEW, BUILD_IN))
}

#[cfg(feature = "web")]
fn serve_web(cli: &Cli, args: &WebArgs, settings: &Settings) -> Result<Exit, CliError> {
    use std::process::{Command, Stdio};
    use std::time::Duration;

    use skillmirror_core::config::{Dirs, EnvOverrides};
    use skillmirror_web::{Config, ServeConfig, serve};

    use super::context::flags;
    use crate::output;

    // The vault and the root must be known before a link is printed that leads to an error page.
    settings.vault()?;
    settings.root()?;
    let options = ServeConfig {
        config: Config {
            flags: flags(cli),
            env: EnvOverrides::from_env()?,
            dirs: Dirs::from_env()?,
            allow_write: args.allow_write,
        },
        port: args.port,
        idle: (args.idle_timeout > 0.0).then(|| Duration::from_secs_f64(args.idle_timeout * 60.0)),
    };
    let result = serve(options, |link| {
        let mode = if args.allow_write {
            "changes allowed"
        } else {
            "read-only"
        };
        output::line(&format!("skillmirror web is running ({mode})"));
        output::line(&format!("Open: {link}"));
        output::line("The link works once and only on this computer. Ctrl-C stops the server.");
        if args.open {
            let spawned = Command::new("xdg-open")
                .arg(link)
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
    });
    result.map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;
    Ok(Exit::Clean)
}
