# rust_Overview

How the Rust tool `skillmirror` is put together: the crates, what flows between them, the rules that keep it safe, and the checks that hold the rules in place. Read it before adding a command, a screen or a dependency. For the rules the tool follows toward the user, see the [behavior contract](../Descr/behavior_Contract.md); for why it is built this way, the [decision record](../Decisions/rust_Rewrite.md).

## Crates

A Cargo workspace (`Crates/*`, default member `Crates/Cli`). The dependency arrow only points at Core.

| Crate | Package | Owns |
|---|---|---|
| `Crates/Core` | `skillmirror-core` | The engine: configuration, vault, scan, plan, apply, backups, and `ops`, the operations the front ends call. Synchronous, no terminal, no async runtime. `just check-deps` fails if `tokio` ever enters its tree |
| `Crates/Cli` | `skillmirror` | The binary: `clap` grammar, one file per command, text and JSON output, exit codes |
| `Crates/Tui` | `skillmirror-tui` | The interactive interface on `ratatui`. Public API is only `run(Launch)`, `Launch`, `TuiError` |
| `Crates/Web` | `skillmirror-web` | Views for a browser: the static HTML report on `maud`, and behind the cargo feature `server` the local web server (`axum`, the only async runtime of the workspace) |
| `Crates/Testkit` | `skillmirror-testkit` | `World`, a throwaway vault and projects for tests |

Every file stays under 300 code lines (`just loc-gate`); a file that grows is split by responsibility.

## The flow of a write

```
settings -> vault (git file lists) -> scan (projects) -> plan (what would change) -> apply (stage, swap)
```

1. **Settings** (`config`): flags, then `SKILLMIRROR_*` variables, then the vault pointer and `<vault>/config.yaml`. Unknown keys and legacy variables are hard errors.
2. **Vault** (`vault`): discovery of skills and groups, the header of each `SKILL.md`, and one `git ls-files` call for the tracked files of every skill. The vault must be a git repository.
3. **Scan** (`scan`): a parallel walk of the root that finds every `.agents/skills`, never follows links, skips noise folders and the configured exclusions, and reports what it could not read as issues, never swallowing them.
4. **Plan** (`plan`): compares each target with the vault and produces a `Plan` of per-target results. Nothing is written. A target that cannot be planned carries its error and is skipped, never the whole run.
5. **Apply** (`apply`): builds the new copy beside the old one (`.stage-<pid>-<nanos>-<n>`), checks that the target still looks like it did when planned (length, mode, mtime, ctime), keeps the old copy in the backup run, and swaps the two folders in one step with `renameat2` (`EXCHANGE` to replace, `NOREPLACE` to create). A failure is per target and leaves the old copy in place.

Front ends add only presentation and confirmation around this. A plan can be shown, diffed (`ops::diff`, `ops::diff_of_plan`), counted (`ops::status`) or applied, and a plan that was shown is the plan that is applied.

## Core modules

See [core_Modules.md](core_Modules.md).

## Rules the code keeps

- **Hard errors, no fallbacks.** A missing vault, a vault that is not a git repository, a mandatory name the vault lacks: each is an error with a message and a hint, not a quiet default.
- **Never write through a link.** A project whose `.agents` or `skills` is a link, a destination that is a link, a tracked symlink in a skill: all refused.
- **A backup before any replace or removal.** `backup` keeps `<state>/backups/<run>/<n>/{entry.json,tree/}` and the newest 30 runs; `undo` is itself a run.
- **Per-target failure.** Exit code 4 means the command ran and some targets failed; nothing is left half written.
- **No work on the interface thread.** The TUI runs every scan, plan and write in a job thread.
- **Odd paths survive.** Paths are `OsStr` everywhere; JSON output shows lossy text, notes on disk keep the exact bytes.
- **Linux only**, enforced by `compile_error!`.

## Errors and exit codes

Each module has its own `thiserror` enum; `Hint` adds a "what to do next" line. The CLI prints one message line and an optional hint line and maps the error to an exit code: `0` clean, `1` drift (`--check`, `status`, `diff`), `2` usage, `3` hard error, `4` partial.

## Threads

Core is synchronous. The scan and the plan use `rayon` and `ignore`'s parallel walker inside a call. The TUI starts one thread per job and talks to the app over one `mpsc` channel; events carry a job id so a late report is dropped. The CLI scan reports progress through `events::Event::ScanProgress` to a spinner on stderr.

## Tests and checks

| Layer | Where | Catches |
|---|---|---|
| Unit and integration tests | `src/*_tests.rs` beside the code, `Crates/*/tests/` | behaviour of a module or a command, run against `Testkit` worlds |
| Snapshot tests (`insta`) | `Crates/Tui/src/tests/snapshots/` | what a screen shows; every change to a snapshot is read before it is accepted |
| Real terminal tests | `Crates/Cli/tests/interface*.rs`, `progress.rs`, `*_terminal.rs` (`portable-pty` + `vt100`) | start-up, the prompts, the scan line, whole screens |
| Browser check | `Code/Development/Report/` | the HTML report in Chromium |
| Parity | `just parity` | any unplanned difference from the frozen Go tool over 131 scenarios |
| Gate | fmt and clippy on Rust 1.97 and 1.99 (they differ), `just loc-gate`, `just check-deps`, `just deny`, all tests | what CI also runs on every push |

A fix starts as a test that fails on the old code; a guard is proved by breaking it and watching a test fail (a mutation check).
