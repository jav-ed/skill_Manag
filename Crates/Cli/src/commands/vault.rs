//! `vault init`: a new vault and, when there is none yet, the pointer to it.

use std::path::Path;

use skillmirror_core::config::{Dirs, read_pointer, write_pointer};
use skillmirror_core::ops;
use skillmirror_core::scan::same_folder;

use crate::args::{Cli, VaultCommand, VaultInitArgs};
use crate::exit::Exit;
use crate::output::{self, Pointer, VaultInitJson, render_vault_init};
use crate::report::CliError;

pub(super) fn run(_cli: &Cli, command: &VaultCommand) -> Result<Exit, CliError> {
    match command {
        VaultCommand::Init(args) => init(args),
    }
}

fn init(args: &VaultInitArgs) -> Result<Exit, CliError> {
    let dir = std::path::absolute(&args.dir)?;
    let root = args.root.as_deref().map(std::path::absolute).transpose()?;
    let dirs = Dirs::from_env()?;
    // A pointer file that cannot be read stops the command before anything is made, unless the new vault
    // is going to replace it anyway.
    let existing = match read_pointer(&dirs) {
        Ok(existing) => existing,
        Err(_) if args.use_it => None,
        Err(e) => return Err(e.into()),
    };
    ops::check_new_vault(&dir)?;
    let (pointer, other) = pointer_plan(existing.as_deref(), &dir, args.use_it);
    if args.dry_run {
        show(args, &dir, root.as_deref(), None, None, true)?;
        return Ok(Exit::Clean);
    }
    let made = ops::init_vault(&dir, root.as_deref())?;
    if pointer == Pointer::Written
        && let Err(e) = write_pointer(&dirs, &dir)
    {
        drop(made.take_back());
        return Err(e.into());
    }
    show(
        args,
        &dir,
        root.as_deref(),
        Some(pointer),
        other.as_deref(),
        false,
    )?;
    Ok(Exit::Clean)
}

/// What to do with the pointer, and the vault it points to now when that is another one.
fn pointer_plan(
    existing: Option<&Path>,
    new: &Path,
    force: bool,
) -> (Pointer, Option<std::path::PathBuf>) {
    match existing {
        None => (Pointer::Written, None),
        Some(old) if same_folder(old, new) => (Pointer::AlreadyHere, None),
        Some(_) if force => (Pointer::Written, None),
        Some(old) => (Pointer::KeptOther, Some(old.to_path_buf())),
    }
}

fn show(
    args: &VaultInitArgs,
    dir: &Path,
    root: Option<&Path>,
    pointer: Option<Pointer>,
    other: Option<&Path>,
    dry_run: bool,
) -> Result<(), CliError> {
    if args.json {
        output::line(&VaultInitJson::new(dry_run, dir, root, pointer).render()?);
    } else {
        output::print(&render_vault_init(dir, root, pointer, other));
    }
    Ok(())
}
