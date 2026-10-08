# Handoff: current state

*The Rust workspace as built: crate layout, how a write flows through the engine, the invariants that keep user data safe, the command surface, measured numbers, tool availability and known gaps. Open it before changing code. Why a design was chosen is in the decision record; this file says what exists.*

## Workspace

Cargo virtual workspace, edition 2024, `rust-version = 1.88`, resolver 3, strict `[workspace.lints]` in the root `Cargo.toml` (`unsafe_code` forbidden; clippy `all` denied, `pedantic` warned; `unwrap_used`, `expect_used`, `indexing_slicing`, `let_underscore_must_use`, `print_stdout` denied; `unreachable_pub` warned). Tests may unwrap (`cfg_attr(test, allow(...))`), `Crates/Cli/src/output` is the only place allowed to print.

| Crate | Package | Role |
|---|---|---|
| `Crates/Core` | `skillmirror-core` | Engine. Synchronous, terminal-free, no async (`just check-deps` fails if `tokio` enters its tree). |
| `Crates/Cli` | `skillmirror` | The binary. clap derive, plain and `--json` output, exit codes, PTY tests. |
| `Crates/Tui` | `skillmirror-tui` | The interactive interface. Public API is only `run(Launch)`, `Launch`, `TuiError`. |
| `Crates/Testkit` | `skillmirror-testkit` | `World`: throwaway vault (git repo), scan root with projects and an isolated `HOME`/XDG environment. |

### Core modules (`Crates/Core/src`)

| Module | What it does |
|---|---|
| `config/` | Vault pointer file (stored as bytes), `<vault>/config.yaml` (`VaultConfig`: `root`, `mandatory`, `exclude_dirs`, `exclude_paths`, `profiles` with `extends`/`groups`/`skills`/`exclude`; `deny_unknown_fields`; validated whenever read), `Settings` (precedence flag > env `SKILLMIRROR_VAULT`/`SKILLMIRROR_ROOT` > pointer/vault config; legacy `SKILL_MANAG_*` variables are a hard error), `save_config` (edits the YAML text in place), `migrate` (old pointer file to new), `Dirs` (XDG directories). |
| `vault/` | `discover`: a skill is a folder containing `SKILL.md`, the folders above it are its groups (`MAX_GROUP_DEPTH` 4, `TooDeep` only when a skill sits below); names unique vault-wide; a group may not share a name with a skill. `read_files`: one `git ls-files -z --stage` spawn for the whole vault. |
| `scan/` | Parallel walk with the `ignore` crate over the canonical root; finds `.agents/skills`; prune list and `exclude_*`; `first_link_above` (refuses symlinked `.agents`/`skills`); non-UTF-8 names and leftover `.stage-*`/`.trash-*` folders become issues, not silent skips. |
| `plan/` | Per target: compare size, then bytes, then mode; remember a destination `Snapshot` (length, mode, mtime per entry). Errors per target (`SkillFileNotTracked`, `SkillFolderIsLink`, ...) never stop the others. |
| `apply/` | Writes the plan with a bounded rayon pool. See "How a write flows". |
| `backup/` | Backup store (`Backups`, `Run`, `Entry`) and `undo`. Every writing command keeps what it replaces or removes in `<state>/backups/<run-id>/<n>/`; `undo` is itself a run, so a second `undo` redoes it. Contract Q34. |
| `ops/` | What front ends call: `Workspace::open/scan`, `plan_sync`, `plan_push`, `plan_install` (add/init), `delete` (takes an optional backup `Run`), `installed`, `status`, `diff`, `doctor`, `bridge`, `resolve` (names, groups, profiles to skills). |
| `events.rs` | `Observer` callback with `Event::{ScanFinished, TargetDone}`; the CLI, the TUI and a future web view share it. |
| `error.rs` | `Error` (one variant per module) and the `Hint` trait (a second line telling the user what to do). |

### How a write flows

1. `Workspace::open` discovers the vault and reads the git file list; `scan` finds the targets.
2. `plan_*` builds a `Plan`: one entry per target with `Create`, `Update` or `Unchanged` and the snapshot of what is there now.
3. `apply` stages each changed skill in `<project>/.agents/.stage-<run>-<index>` (outside `skills/`, so no agent ever lists a half-built copy), copying the git-tracked source files with their permission bits.
4. New skill: `renameat2(NOREPLACE)`. Existing skill: `renameat2(EXCHANGE)`, then the old copy (now at the stage path) is compared with the snapshot. If someone changed it in between, the exchange is undone and the target fails with `DestinationChanged`, so a concurrent edit survives.
5. The old copy is deleted. If that fails the skill is still correct and the problem is reported as a `Leftover` warning with the path.
6. `delete` renames the folder to `<project>/.agents/.trash-<run>-<index>` first (one atomic step out of `skills/`), then removes it.
7. A failing target never stops the others. Exit code 4 reports a partial run.

No `fsync` is called: the writes are crash-safe (a killed process leaves no half skill) but not power-loss-safe. This is documented in the doc comment of `apply` and accepted because the skills are git-tracked copies that the next sync restores.

### Safety invariants (each has tests; keep them)

- Never write through a symlink: `.agents` or `.agents/skills` being a link is a hard error for plan, apply and delete (contract Q33). A symlinked skill folder can still be removed with `delete --project` (only the link goes).
- Skill names are one plain path component (`validate_name`): no separators, no leading dot, never empty. The Go tool removed `.agents` for `delete ..`.
- A skill whose `SKILL.md` or any listed file is not tracked by git is a hard error and nothing of it is copied; tracked symlinks and gitlinks are refused.
- An update that finds the destination changed after planning fails that target and restores the previous state (`DestinationChanged`).
- A failed create in a new project gives back the directories this run made for it only when every target of that project failed (apply creates `skills/` once per project before the pool starts).
- Config files are validated on read even when a flag overrides the value (contract Q32); a malformed `config.yaml` is never replaced silently.
- `save_config` edits only the keys it owns, follows a symlinked config, keeps CRLF and a BOM; comments inside the replaced list go with the old items.

## Command surface

Global options on every command: `--vault <DIR>`, `--root <DIR>`. No subcommand: opens the TUI when stdin and stdout are terminals, otherwise prints the help to stderr and exits 2.

| Command | Does |
|---|---|
| `sync [--dry-run] [--check] [-y] [--json] [--all]` | Update the skills a project already has (opt-in rule: never adds one) |
| `push [same flags]` | Install the `mandatory` skills into every project that has a skills directory |
| `list [--json]` | Every installed skill folder |
| `delete <NAME> [--project DIR] [--dry-run] [-y] [--json]` | Remove one skill from one or all projects |
| `skills [--group PATH] [--json]` | The vault's skills grouped by folder |
| `doctor [--json]` | Read-only health check of the machine, configuration, vault, skill headers and scan root; exit 0 / 1 warnings / 3 errors (contract Q43) |
| `bridge [--dry-run] [--yes] [--all] [--json]` | Links `.claude/skills` to `../.agents/skills` in every project where the vault config lists `targets: [claude]`; never replaces anything; `add` and `init` do it for their project (contract Q44) |
| `diff [SKILL] [--project DIR] [--stat] [--json]` | Read-only: the lines a sync would bring in and take away, per file (contract Q42) |
| `status [--all] [--json]` | Read-only: how every project stands against the vault (outdated, mandatory missing, not in the vault, failed); exit like `--check` (contract Q41) |
| `add [SKILL]... [--group P] [--profile N] [--project DIR] [--dry-run] [-y] [--json]` | Install skills, groups or profiles into an existing project |
| `init <DIR> [SKILL]... [--group] [--profile] [--git] [--no-mandatory] ...` | Create a project directory and install mandatory skills plus a selection |
| `undo [RUN] [--project DIR] [--skill NAME] [--dry-run] [-y] [--json]` | Bring a run back (default: the newest); undoing is a run too, so a second `undo` redoes it |
| `history [--json]` | The runs in the backup store, newest first |
| `migrate [--retire]` | Copy the old tool's vault pointer to this tool |
| `completions <shell>` | Shell completions |
| `tui` | The interactive interface |

Exit codes: `0` clean, `1` drift found by `--check`, `2` usage error (also: writing without `--yes` and without a terminal), `3` hard error, `4` partial failure. Writing commands ask for confirmation in a terminal; without a terminal they require `--yes` or refuse.

## TUI

ratatui 0.30 on the `termina` backend through `ratatui-termina` (crossterm 0.29 stalls on input bursts of about 1 KB, which a mouse sweep or a paste produces). Own input layer (`input.rs`, `binding.rs`, `hit.rs`), `tui-input` for text fields, `nucleo-matcher` for the fuzzy filter, an OSC 8 link via `CellDiffOption::ForcedWidth`. Background work (scan, plan, apply, delete, the wizard's look at a vault) runs in threads and reports over one mpsc channel; every report carries a job id and the app hears only the job its screen waits for (contract Q36); events are coalesced and the tick runs only while something animates. Screens: menu (one line per entry when two would not leave room for the text), work screens (sync and push: select, plan, confirm with the files that would be removed, run, results; delete: select, confirm, run, results; the results page names the backup run; `i` lists the scan problems), list, add and init (folder picker, name field, skill rows, plan, confirm, install, links; contract Q48), history (backup runs, undo question, results; Q47), setup wizard (vault and root with a folder picker, mandatory skills, save), help overlay. Mouse works everywhere. The app is tested by driving `App` with synthetic input against a `TestBackend` (insta snapshots in `Crates/Tui/src/tests/snapshots/`) and, for the real terminal path, with portable-pty and vt100 in `Crates/Cli/tests/interface.rs`.

## Numbers (measured on the user's real data, read-only)

- Real vault: 24 skills in the vault; the tree under the scan root holds 436 installed skill folders in 65 projects, the same count as the Go tool.
- Scan: about 0.7 s for the whole root with the Rust tool; the Go dry run took 4.8 to 7 s in the same comparison (13 s cold earlier, 1.5 s warm on a smaller tree). Plain `find` with the same pruning needs about 1.6 s, so the directory walk dominates; the Rust walk is parallel.
- Plan: comparing all 395 targets (the earlier measurement) took 38 ms; apply of all of them 274 ms with 8 threads.
- `hyperfine` is not installed; the numbers came from simple timers.

## Parity with the Go tool

131 oracle scenarios: 87 match, 44 expected divergences (each cites a contract Q-id), 0 unexpected. Re-run on the handoff tip on 2026-10-08 (`just parity`, about 75 s): the same 87 / 44 / 0. Material: `Docs/Investigation/Parity_Oracle/` (setup, table, report) and `Code/Development/Parity/` (harness). The Go source stays in the tree until cutover and can always be rebuilt from commit `c7310f9`.

## Tools

Installed and used by the gate script (`Scratch/gate.sh`, local only): `cargo`, `just`, `cargo-nextest`, `tokei`, `cargo-deny`, `hyperfine`, `cargo-insta`, rustc 1.97 (default) and 1.99.0 (what CI runs; clippy lints differ between them). CI (`.github/workflows/ci.yml`: fmt, clippy, test, loc-gate, check-deps, deny) runs on every push to the PR and has been green on every commit since `3172930`.

## Known gaps

- Review rounds 1 to 3 are fixed (see `Docs/Investigation/Review_Rounds/`, every status line says what was done and what was declined). Round 3 was a self review; a fresh pair of eyes on the backup and undo code and on the new interface screens is still welcome.
- No root-level `--dry-run` alias (the Go tool had `skill_Manag --dry-run`; the Rust tool uses `sync --dry-run`; decided: no alias).
- The TUI confirmation lists the files that would be removed but has no diff view yet.
- `exclude_dirs` in the user's vault config does not yet contain `Scratch`; this repo's `Scratch/Oracle/` holds Go fixtures with `.agents/skills` folders that a real sync would rewrite (only matters on the lead's machine).
- The repo `.gitignore` contains `fast*` and `dist*` (user-written); they would hide any future file or folder whose name starts with these words.
