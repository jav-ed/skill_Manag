# linker_Setup

Local setup, environment bootstrap, install steps, first-party repo shortcuts, and the manifest of external repos that get cloned into `/Repos/` for source-code or documentation reference. Anything an engineer (or agent) needs to bring this project up on a fresh machine belongs here.

## Docs

- [Internal repo paths](internal_Repo_Paths.md): maps first-party repo shortcut names to host-scoped checkout paths. Docs should use the shortcut name in prose and link here when the absolute path matters. Holds the path of the skill vault that sync mirrors from.
- [External reference repos](repos_List.md): manifest of third-party or external repos cloned into `/Repos/` (gitignored at repo root). Lists what should be present and gives the `git clone --depth 1` command for each, so the folder can be repopulated on a fresh machine.
- [Scratch cleanup](scratch.md): manual full reset of disposable files in `/Scratch/` and the four empty folders it recreates. Read before running `just scratch-clean` or before deciding whether a file belongs in Scratch.
