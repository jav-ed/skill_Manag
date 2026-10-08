//! Starting git in an environment that cannot point it at another repository.

use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;
use std::process::Command;

/// A `git` command that ignores every `GIT_*` variable of the caller (`GIT_DIR`, `GIT_INDEX_FILE`, ...),
/// so it works on the folder it is given and nowhere else, and answers in English.
pub(crate) fn command() -> Command {
    command_without(std::env::vars_os().map(|(name, _)| name))
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
