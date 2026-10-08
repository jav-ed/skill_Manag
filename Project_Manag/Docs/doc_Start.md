# doc_Start

*This `doc_Start.md` is the docs entry point, structured so an agent can quickly decide what to read and what to skip. It opens with a short summary of the repo and key entry-point files, then routes to each topic area through labeled links. Open a linker only when the task calls for it; the labels are written to make that decision possible without clicking.*

`skill_Manag` is a CLI and TUI that mirrors agent skill folders from one master vault into every matching `.agents/skills/` directory across a codebase: git-tracked, SSH-safe, no symlinks. The Go implementation on `main` is being rewritten in Rust on the branch `rust-rewrite-handoff` (the branch to continue on); the Go code stays as the behavior reference until cutover.

Entry point(s): `main.go`, then `cmd/root.go` (cobra setup and menu loop), then `cmd/tui/menu.go` (TUI entry). The Rust sources start at `Cargo.toml` in the repo root, then `Crates/Core/src/lib.rs` (engine) and `Crates/Cli/src/main.rs` (binary).

## Docs

- [Architecture](Architecture/linker_Architecture.md): code structure of the current Go implementation: CLI wiring in `cmd/`, TUI screens and their phase models in `cmd/tui/`, scan and copy logic in `internal/`, shared styles. Open it for any change to the Go code.
- [Descr](Descr/linker_Descr.md): what the tool does and does not do: the sync concept, the opt-in rule, vault and project relationship, push and mandatory skills, scan exclusions, config files. Open it before changing sync behaviour.
- [Setup](Setup/linker_Setup.md): the vault path on this machine, the manifest of external reference clones in `/Repos/`, and the `just scratch-clean` command for the disposable `/Scratch/` folder.
- [Decisions](Decisions/linker_Decisions.md): why the project is built the way it is, starting with the Rust rewrite: name, workspace layout, chosen and rejected crates, feature scope, licence consequences, phase plan and rulings still open. Open it before adding a dependency or changing the architecture.
- [Investigation](Investigation/linker_Investigation.md): dated research with measurements behind those decisions: library comparisons for the engine, CLI, TUI and web view, prior art among skill managers, naming and distribution. Open it to re-check a number or a licence.
- [README](../../README.md): user-facing install steps, commands and config.

## Live work

- [Rust rewrite handoff](../Live_Working/Rust_Rewrite_Handoff/handoff_Overview.md): the state of the branch `rust-rewrite-handoff` for whoever continues it: what is built and verified, what is half built (backup store and `undo`), the ordered backlog with "done when", the working agreement with the user, the verification playbook, open questions and pitfalls. Open it before touching the Rust code or when picking the next piece of work.
- [Open issues](../Live_Working/open_Issues.md): short list of active items that point into the handoff.

## Repo References

- [Internal repo paths](Setup/internal_Repo_Paths.md): first-party repo shortcut names and host-scoped checkout paths. Use this for repos the project owns or operates across machines, including the skill vault.
- [External reference repos](Setup/repos_List.md): third-party or external shallow clones that belong under the gitignored `/Repos/` folder. Use this for upstream source/docs clones used as references, such as the bubbles TUI library.
