# Rust stack: Requirements

What the Rust rewrite of the skill manager must do, which constraints bind it, and which questions the library research has to answer. Written from the Go code (about 3,900 lines), its README and docs, and a baseline measurement. Every other file in this folder is judged against this one.

## What the tool is

A vault holds master copies of agent skills. A scan root holds many projects. Each project may contain `.agents/skills/<name>/`. The tool mirrors vault skills into the projects that already have them (sync), force-installs mandatory skills (push), lists what is installed (list), and removes skills (delete). Files are copied, never symlinked, so projects stay git-tracked and work over SSH.

## Baseline (Go, measured 2026-10-07)

- 395 installed skills across 60 projects, scan root `1_Code`.
- `--dry-run`: 13 s on a cold cache, 1.5 s warm.
- A plain `find` with the same pruning needs 1.6 s warm, so the single-threaded directory walk is nearly the whole cost.
- Sync deletes and re-copies every file even when nothing changed.
- One `git ls-files` subprocess runs per target (395 spawns).

## Engine requirements

1. **Config.** Vault pointer file `~/.config/skill_Manag/vault`. Vault config `<vault>/config.yaml` with `root`, `mandatory`, `exclude_dirs`, `exclude_paths`. Precedence: flag, then environment (`SKILL_MANAG_VAULT`, `SKILL_MANAG_ROOT`), then file. The old code rewrites the YAML through viper, which drops comments and reorders keys; the new code must edit the file without destroying comments. Unknown or malformed keys are hard errors.
2. **Vault discovery.** Today the vault is flat. The new vault supports groups: a skill is any folder containing `SKILL.md` at any depth, and the folders above it are its group. Skill names must be unique across the vault. `SKILL.md` frontmatter (`name`, `description`) is parsed.
3. **File selection.** Only git-tracked files of a skill are copied (index contents, no subprocess). Untracked files inside a skill are detected and reported. Symlinks are excluded. Permission bits are preserved. A vault that is not a git repository is a hard error.
4. **Project scan.** Walk the root, find every `.agents/skills`, never descend below it, prune noise directories (`.git`, `node_modules`, `vendor`, `dist`, `build`, `out`, `target`, `.next`, `.nuxt`, `.venv`, `__pycache__`, `.tox`, `.pytest_cache`, `.cache`, `.turbo`, `.parcel-cache`) plus `exclude_dirs` and `exclude_paths`. Must run in parallel and stream results while running. A cached registry of known projects with explicit revalidation is wanted. Decided by the user (2026-10-07): `.agents/skills` is the canonical and only discovered location. Other agent directories (`.claude/skills` and similar) are created as symlinks to it when a project asks for them, never discovered as skill sources, and never replace an existing real directory.
5. **Plan.** For every target compute per-file changes (added, modified, removed, mode changed, unchanged) without writing. Content comparison needs a fast path (size, mtime) and a strong hash. Optional provenance lock file per project for drift detection (outdated versus locally edited).
6. **Apply.** Per-skill atomic swap: stage next to the destination, then rename, so a failure never leaves half a skill. Bounded parallelism across targets. Dry-run touches nothing. Replaced content goes to a backup location so delete and overwrite can be undone. Errors are collected and reported, never swallowed, and the exit code reflects them.
7. **Operations.** sync, push, list, delete, plus new `add`, `init` (new project from a profile or groups), `status`, diff, `doctor`.
8. **Events.** The engine reports progress as a stream of typed events (project found, plan ready, target applied, error) that the CLI, TUI and web view all consume. Needs a channel or observer design.

## Front-end requirements

**CLI.** Subcommands with generated help, `--dry-run`, `--json`, `--check` (non-zero exit on drift), `--yes`, meaningful exit codes, shell completions, `NO_COLOR` support, plain output without a TTY, progress output for long scans, readable error reports.

**TUI.** Screens: menu, sync, list, delete, push (with mandatory-edit overlay), setup wizard (filesystem picker, three steps, confirmation), plus new init, status and diff, group tree. Behaviour to keep or improve: full mouse support (hover, click, wheel), paginated or scrolling lists with 400 or more rows, live filter (fuzzy wanted), progress bar and spinner, `?` help overlay, back navigation (`alt+left`, `q`, clickable header arrow), clickable header link, alternate screen, resize handling, correct unicode widths. One persistent application with a screen router and shared scan state, not one program per screen. Rendering must be testable without a terminal.

**Web view (new, optional).** Local only. Started from the CLI, served on loopback, assets embedded in the binary. Candidate contents: overview matrix of projects against skills with status, group browser, diff view, actions with confirmation, live scan progress. Because it can delete files it needs a security model (loopback bind, per-run token, origin checks). Must not force an async runtime or a JavaScript toolchain onto users who only want the CLI and TUI unless the research shows that is worth it.

## Quality and constraints

- **Speed.** Target set after the Phase 0 baseline; warm full dry-run should be a fraction of a second. Cold runs are disk-bound, so the walk must overlap I/O.
- **Safety.** No data loss on failure, no silent overwrite of local edits, hard errors only (coding skill: no fallbacks).
- **Code rules.** Standard Rust naming, at most 300 lines of code per file, one responsibility per file, no deep nesting, comments preserved.
- **Platforms.** Linux only (Manjaro is the reference machine). No Windows and no macOS: Unix-only APIs (permission bits, symlinks, `renameat2`, `std::os::unix`) are used freely, and no effort goes into portability, cross-platform key events, or non-Linux release channels.
- **Licence.** The project is Hippocratic License 3.0. Dependencies must be permissive (MIT, Apache-2.0, BSD, ISC, Zlib, MPL-2.0 acceptable). Any GPL, AGPL or LGPL dependency is a blocker until reviewed.
- **Toolchain.** Rust 1.98.1 is installed; edition 2024 or newer is fine.
- **Cost.** Compile time, binary size and dependency count count against a crate. A crate that saves 30 lines but adds 40 transitive dependencies is a no.
- **Health.** Prefer crates with a release in the last twelve months, an active maintainer, and a source tree that can be read. Unmaintained crates need a strong reason.

## Questions per research stream

Each stream writes one leaf file here, ends it with a verdict table (crate, verdict yes, no or maybe, one-line reason, version checked, licence), and names the crates it cloned.

- **TUI stack** ([tui_Stack.md](tui_Stack.md)): ratatui and its widget ecosystem, framework layers on top of it, alternatives, mouse hit-testing, event loop and threading, snapshot testing, filesystem picker, fuzzy filter, progress and spinner widgets.
- **Engine stack** ([engine_Stack.md](engine_Stack.md)): parallel walking, gitignore matching, reading the git index without a subprocess, hashing, atomic writes, copy-on-write copies, timestamps, diffing, YAML parsing and comment-preserving editing, frontmatter parsing, event channels.
- **CLI and quality stack** ([cli_Stack.md](cli_Stack.md)): argument parsing, completions and man pages, config paths, error reporting, JSON output, progress bars outside the TUI, tests (integration, snapshot, property), benchmarks, linting, licence and supply-chain checks, MSRV policy.
- **Web view** ([web_View.md](web_View.md)): static report versus local server versus SPA versus WASM full-stack, server frameworks, templating, embedded assets, live updates, browser launching, security model for a local tool that can delete files, cost of the async runtime and of a frontend build.
- **Prior art** ([prior_Art.md](prior_Art.md)): existing skill and agent-config managers, the `SKILL.md` format and directory conventions across agents, what users of such tools ask for, and which of our candidate features are worth building.
- **Name and distribution** ([name_And_Distribution.md](name_And_Distribution.md)): candidate names checked against crates.io, GitHub, AUR, Homebrew and common binaries, workspace layout options, release tooling, installation channels, config migration from the old name.
