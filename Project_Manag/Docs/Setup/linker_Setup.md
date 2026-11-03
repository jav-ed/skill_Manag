# linker_Setup

Local setup, environment bootstrap, install steps, first-party repo shortcuts, and the manifest of external repos that get cloned into `/Repos/` for source-code or documentation reference. Anything an engineer (or agent) needs to bring this project up on a fresh machine belongs here.

## Docs

- [Internal repo paths](internal_Repo_Paths.md): maps first-party repo shortcut names to host-scoped checkout paths. Docs should use the shortcut name in prose and link here when the absolute path matters. Holds the path of the skill vault that sync mirrors from.
- [External reference repos](repos_List.md): manifest of third-party or external repos cloned into `/Repos/` (gitignored at repo root). Lists what should be present and gives the `git clone --depth 1` command for each, so the folder can be repopulated on a fresh machine.
- [Build resources](build_Resources.md): what a Rust build costs in memory, disk and time (measured), what the repository turns off by default, and the knobs to use less or to get debug information back. Read before building on a small machine.
- [Sandbox world](../../../Code/Development/World/README.md): a throwaway home with a vault and five projects in different states, to try the tool by hand (`just world`), the check that reads the tool against it (`just world-check`), and why it is built on demand and not committed (a project committed here would be found and written into by a real sync). Open it before testing a behaviour by hand or before writing an example for the docs site.
- [Scratch cleanup](scratch.md): manual full reset of disposable files in `/Scratch/` and the four empty folders it recreates. Read before running `just scratch-clean` or before deciding whether a file belongs in Scratch.
