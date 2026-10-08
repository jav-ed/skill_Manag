//! `web`: the local server.

use crate::args::{Cli, WebArgs};
use crate::exit::Exit;
use crate::report::CliError;

#[cfg(not(feature = "web"))]
pub(super) fn run(_cli: &Cli, _args: &WebArgs) -> Result<Exit, CliError> {
    Err(CliError::usage(
        "this build of skillmirror has no web server",
        "build with the default features: `cargo install --path Crates/Cli --locked`",
    ))
}

#[cfg(feature = "web")]
pub(super) fn run(cli: &Cli, args: &WebArgs) -> Result<Exit, CliError> {
    use std::process::{Command, Stdio};
    use std::time::Duration;

    use skillmirror_core::config::{Dirs, EnvOverrides};
    use skillmirror_web::{Config, ServeConfig, serve};

    use super::context::{flags, load_settings};
    use crate::output;

    // The vault and the root must be known before a link is printed that leads to an error page.
    let settings = load_settings(cli)?;
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
