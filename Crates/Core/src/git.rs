//! Starting git in an environment that cannot point it at another repository.

use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::process::Command;

/// A `git` command that ignores every `GIT_*` variable of the caller (`GIT_DIR`, `GIT_INDEX_FILE`, ...),
/// so it works on the folder it is given and nowhere else, and answers in English.
pub(crate) fn command() -> Command {
    command_without(std::env::vars_os().map(|(name, _)| name))
}

/// Stages everything below the folder `name` of `dir`, new, changed and removed files alike. `dir` is
/// a folder of the work tree, not necessarily its top. On failure: what git said.
pub(crate) fn stage_folder(dir: &Path, name: &str) -> Result<(), String> {
    let out = command()
        .env("GIT_LITERAL_PATHSPECS", "1")
        .arg("-C")
        .arg(dir)
        .args(["add", "-A", "--", name])
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        return Ok(());
    }
    Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
}

fn command_without(names: impl Iterator<Item = OsString>) -> Command {
    let mut cmd = Command::new("git");
    cmd.env("LC_ALL", "C");
    for name in names.filter(|n| n.as_bytes().starts_with(b"GIT_")) {
        cmd.env_remove(name);
    }
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_git_variable_is_removed_and_others_are_left_alone() {
        let names = [
            "GIT_DIR",
            "GIT_INDEX_FILE",
            "GIT_CONFIG_COUNT",
            "HOME",
            "PATH",
        ];

        let cmd = command_without(names.iter().map(OsString::from));

        let mut removed: Vec<_> = cmd
            .get_envs()
            .filter(|(_, value)| value.is_none())
            .map(|(name, _)| name.to_string_lossy().into_owned())
            .collect();
        removed.sort();
        assert_eq!(removed, ["GIT_CONFIG_COUNT", "GIT_DIR", "GIT_INDEX_FILE"]);
    }
}
