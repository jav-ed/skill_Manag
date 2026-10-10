# Decision: Rewrite in Rust as skillmirror

Status: accepted by the lead on 2026-10-07 for the parts the user delegated ("you decide, mix is fine"); the items under "Rulings still open" wait for the user. Date of the evidence: 2026-10-07, Manjaro, rustc 1.98.1.

Evidence lives in [the six research reports](../Investigation/Rust_Stack/linker_Rust_Stack.md). This file holds only the decisions, the reasons that were not obvious, and where a report was overruled. Numbers and file:line proofs stay in the reports.

## Context

The Go tool mirrors skill folders from a vault into every project's `.agents/skills/<name>/`. It works but is slow: a dry run over 395 installs in 60 projects takes 13 s cold and 1.5 s warm, nearly all of it a single-threaded directory walk plus 395 `git ls-files` spawns. The user wants it in Rust, plus nested vault groups, creating a new project with skills installed at once, a better TUI and CLI, and a web view. Hard errors, no fallbacks. Linux only.

## Decisions

### 1. Name and layout

| Topic | Decision | Reason |
|---|---|---|
| Name | `skillmirror` (binary and crate), optional alias `skmir` | says what it does (a mirror, not an overlay), free on crates.io and AUR, no same-purpose project. The user may still veto it |
| Product names in code | one module, `Crates/Core/src/brand.rs` | a rename touches one file |
| Config directory | `~/.config/skillmirror/` (XDG aware), env `SKILLMIRROR_VAULT` and `SKILLMIRROR_ROOT` | the old spelling `skill_Manag` is handled only by `migrate` |
| Workspace | virtual Cargo workspace `Crates/{Core,Cli,Tui,Web,Testkit,Xtask}`, `default-members = Cli` | leaf edits rebuild 6 to 9 times faster, "core never depends on a front end" is enforced by the compiler, tokio never enters CLI or TUI builds |
| Transition | Rust at the repo root beside the frozen Go tree, tag `go-oracle`, parity tests, one cutover commit that deletes Go, merge `--no-ff` | keeps `git log -- cmd/` working, Go stays runnable as the oracle until parity is proven |
| Folders | uppercase first letter for repo folders, lowercase for ecosystem names (crate, binary, config dir) | the user's naming convention |
| Platform | Linux only, `compile_error!` on other targets, Unix APIs used freely | user decision |
| Code rules | 300 code lines per file (tokei semantics: blanks and comments excluded), one responsibility per file, no unwrap, expect or indexing in product code, comments are kept | coding skill, enforced by `[workspace.lints]` and a loc gate |

### 2. Engine (`skillmirror-core`, synchronous, no terminal, no async)

| Concern | Decision | Reason |
|---|---|---|
| Project scan | `ignore` parallel walker (12 threads, standard filters off, `filter_entry` for noise, `Override` for `exclude_*`), `rayon` | 4.6 times faster than the Go walker, no custom code, same crate also finds untracked files |
| Scan upgrade path | rayon plus `rustix::RawDir` walker (measured 8.5 times faster, about 60 lines) | only if the cold scan is still too slow once measured; it hides behind one function in `scan/walk.rs` |
| Vault file list | one `git` subprocess per run (`git ls-files -z -s` and `--others --exclude-standard`), run inside the vault with `GIT_*` variables removed; stderr becomes the hard error | see "Overruled" below |
| Comparison | `std` only: size differs means changed, otherwise compare bytes | 38 ms for all 395 targets; a hash adds work for nothing; no mtime fast path (false "unchanged" is the dangerous direction) |
| Copy source | the working-tree file of every tracked path (Go behaviour), a tracked symlink or gitlink makes that skill a hard error (nothing of it is copied), permission bits preserved; the skill's `SKILL.md` must itself be tracked | parity; `doctor` will inshallah warn about unstaged edits in the vault |
| Apply | stage in `.agents/.stage-<run>/` (never inside `skills/`), swap with `renameat2(RENAME_EXCHANGE)` through `rustix`, old tree goes to the backup store; `tempfile`, `fs-err`, `std::fs::copy` | a failure never leaves half a skill; 274 ms for all 395 targets on 8 threads; unsupported filesystem is a hard error |
| Backups and undo | own store under `$XDG_STATE_HOME/skillmirror/backups/<run-id>/<n>/` (a JSON note `entry.json` plus the old folder in `tree/`), always on, retention by run count (30), `undo` is itself a run so a second `undo` redoes it; contract Q34 | `undo` needs run and project, the `trash` crate has neither |
| Config read | `serde-saphyr` into `deny_unknown_fields` structs; unknown or malformed keys are hard errors | errors carry line, column and snippet |
| Config write | own line editor (about 70 lines) followed by mandatory re-parse and typed comparison, then temp file plus rename | keeps comments; `yaml-edit` has a list-replacement bug; TOML stays plan C |
| Frontmatter | manual split on the `---` fences plus `serde-saphyr`; used by `list`, groups and `doctor`, not by sync | no crate beat the 20-line split |
| Paths | `etcetera` (config, state, cache), honours `XDG_*` | the Go tool ignored `XDG_CONFIG_HOME`; `dirs` and `directories` pull MPL-2.0 and lowercase the name |
| Events | typed `Event` enum over `std::sync::mpsc`, observer takes a `Sender<Event>`; cancellation by `AtomicBool` | CLI, TUI and web all read the same stream; no tokio in the engine |
| Errors | `thiserror` enums per module plus a small `Hint` trait ("what to do next"); about 80-line report printer in the CLI | no anyhow, miette or color-eyre in the product |
| Registry | JSON file in the cache directory (rebuildable), records every `.agents/skills` directory (65 to 71, not only the 60 with a match), `stat` revalidation, `--rescan` and a 24 h TTL | built in phase 5, not in the first engine cut (see below) |
| Diff | `similar` | zero dependencies, unified and inline |
| Lock digest | `sha2`, only when the lock file is built (phase 5, user said "maybe later") | humans can verify with `sha256sum` |

Behaviour of vault discovery: a skill is any folder that contains `SKILL.md`, descent stops at the first one, dot-directories are skipped, skill names are unique vault-wide, a group name must not equal a skill name, depth is capped and exceeding it is a hard error. The check that frontmatter `name` equals the folder belongs to `doctor`, not to discovery. The installed layout stays flat (`.agents/skills/<name>/`); groups exist only in the vault, so moving a skill between groups changes no project.

Opt-in rule (unchanged): `sync` updates only skills a project already has; `push` and `add` install; `init` creates a project and installs `mandatory` plus the chosen profile or groups.

### 3. CLI (`skillmirror`)

- `clap` 4.6 (derive, env) with `clap_complete` (ahead-of-time completions), `anstream` and `anstyle` for colour (already inside clap's tree), `indicatif` for the scan progress line, `serde_json` for `--json`. `NO_COLOR` and non-TTY output are plain.
- Commands in the first CLI cut: `sync`, `push`, `list`, `delete`, `migrate`. The Go tool has no `sync` or `push` subcommand (sync is the root action, push exists only in the TUI), and only `--dry-run` and `delete [name] [--project] [--dry-run]` are non-interactive, so the parity surface is `sync --dry-run` against Go `--dry-run`, and `delete` against `delete` (see [behavior contract](../Descr/behavior_Contract.md), sections 1 and 7). The 30 quirks tagged CHANGE or UNCLEAR there seed `known_divergences.toml`; the dangerous ones (wipe on non-ASCII file names, `delete ..`, sync of a skill missing from the vault, zero tracked files treated as success, root directory named `build` scanning nothing) are fixed from the start. Then `add`, `init`, `status`, `diff`, `doctor`, `undo`, `completions`.
- Exit codes, defined once: 0 clean, 1 drift or `--check` failed, 2 usage error, 3 hard error, 4 partial success.
- Environment overrides: only `SKILLMIRROR_VAULT` and `SKILLMIRROR_ROOT` (plus flags). The Go tool let every config key come from the environment; dropping that is a deliberate behaviour change, recorded in the contract. Any `SKILL_MANAG_*` variable that is set is a hard error that names its replacement, because silently ignoring it could sync the wrong vault.
- Release profile stays the quick one for `hyperfine` loops; packaging uses `--profile dist` (fat LTO, one codegen unit, `panic = "abort"`, strip). Because `abort` skips destructors, `doctor` lists leftover `.stage-*` directories.

### 4. TUI (`skillmirror-tui`)

- `ratatui` 0.30.2 with the termina backend (`ratatui-termina`), not crossterm: crossterm 0.29 stops reading input after a burst of about 1 KB (reproduced, upstream bug open), which matters over SSH and when pasting paths. The app talks to an own `Input` type, so crossterm is a 90-line drop-in once its fix lands. A flood test in CI guards the choice.
- No framework (tui-realm, rat-salsa, cursive rejected): one `App`, a `Screen` enum as router, pure `App::handle(Event) -> Effects`, `std::thread` plus one `mpsc` channel, zero idle CPU.
- Mouse: rectangles are recorded while drawing (`HitMap`) and resolved topmost first. Row arithmetic like the Go `msg.Y - itemsStart` is banned.
- Small crates: `tui-input` (state only), `tui-tree-widget` (group tree), `nucleo-matcher` (fuzzy filter, MPL-2.0, marked), `terminal-colorsaurus` (light or dark, `theme = auto|dark|light` in config). Written ourselves: spinner, key-binding table that also drives the `?` overlay, directory picker (169 lines in the prototype), OSC 8 link behind a runtime flag, virtual lists with `Scrollbar` instead of paginator.
- Tests: `TestBackend` plus `insta` snapshots, headless mouse tests, and PTY end-to-end tests with `portable-pty` and `vt100`.
- The working prototype (1,347 lines, 25 tests) was ported into `Crates/Tui`; its source is archived in [the prototypes folder](../Investigation/Prototypes/linker_Prototypes.md).

### 5. Web view (`skillmirror-web`, cargo feature `web`)

- Stage 0, built early: `skillmirror report` writes one static, self-contained HTML file (matrix, group browser, skill detail, diffs, client-side filter, dark mode). No server, no attack surface. Renderer: `maud` (automatic escaping).
- Stage 1, decided after the TUI exists: `skillmirror web`, a loopback server on `axum` 0.8 with minimal features, server-rendered HTML, about 150 lines of vanilla JS, server-sent events for scan progress. Read-only by default. Security minimum: random port on 127.0.0.1, Host allow-list, one-time launch token exchanged for an HttpOnly SameSite=Strict cookie, `Sec-Fetch-Site` and `Origin` checks on every request, POST-only mutations, no CORS, CSP header.
- Stage 2: mutations only behind `--allow-write` with a plan and confirm step, after the drift guard exists. Both stages were built on 2026-10-08 (contract Q56), with polling instead of server-sent events for progress: a job is one run of at most a few seconds and polling needs no streaming code.
- Rejected at the time: SPA (hundreds of npm packages), WASM full-stack, desktop shells, `tiny_http` (open CVEs, no release since 2022). **Overruled by the user on 2026-10-08** for the interface: "you can use something like Astro ... SolidJS ... Lucide icons ... Vite ... MotionJS, you don't have to hold back". The interface is `Ui/`: Astro as the static shell, Solid views, `lucide-solid` icons, `motion`, Vite through Astro (255 npm packages, all dev-time, pinned by `Ui/package-lock.json`). What keeps the original worries in check: the project is separate (its own folder, its own tooling, meeting Rust only in the JSON API and the built files, so it can move to its own repository later), the built files are committed and embedded so `cargo install` needs no node, no Astro islands (they would need an inline script and so `unsafe-inline`), text goes in as text, and the server's strict Content-Security-Policy plus a test over every built page and a browser check enforce it. `lucide-animated` is React-only (needs `react` and `motion`), so the icons are `lucide-solid` animated with CSS and Motion.
- tokio appears only through the web crate; `cargo tree -p skillmirror-core -i tokio` must fail (`just check-deps`).

### 6. Features

| Build now (phases 2 to 6) | Build later | Do not build |
|---|---|---|
| groups (vault-only), profiles with `extends`, `add`, `init`, `status`, `diff`, change-aware sync, backup and `undo`, `--json`, `--check`, `doctor` with SKILL.md lint, `targets` (`agents` canonical, `claude` symlink bridge) | provenance lock with three-way drift states (user: "maybe later"), `adopt`, `hold`, dependency closure for `add`, web stage 1 and 2 | commit-after-sync, watch mode, enable and disable, version pinning, merge with conflict markers, marketplace, security audit of skill text |

`.agents/skills` is the only discovered location. Other agent directories (`.claude/skills`, `.kiro/skills`) are created as relative symlinks to it when a project asks for them, are never read as sources, never replace a real directory, and are recreated by `sync` when missing (a gitignored bridge vanishes on clone). The kernel enforces "never replace a real directory": `rename` of a link onto a directory fails with `EISDIR`. Decided later (contract Q44): `sync` does not recreate a missing bridge; the `bridge` command does, `add` and `init` call it for their project, and `status` and `doctor` report a missing one.

### 7. Quality, licence, supply chain

- Tests: `cargo nextest`; unit tests in sibling `tests.rs` files (inline test modules would count toward 300 lines); integration tests per crate; end-to-end and parity tests in `Crates/Cli/tests/`; `assert_cmd`, `assert_fs`, `predicates`, `insta`, `rstest`, `proptest`, `tempfile`; generated fixture trees, never a committed folder named `.agents/skills` (the real tool would sync into it).
- Parity: the frozen Go binary is the oracle. A differential harness runs both binaries on the same generated tree and compares output and resulting trees; every intended difference sits in `known_divergences.toml` with a reason. Cutover requires the parity job green and that file reviewed.
- Lints: `[workspace.lints]` as in the scaffold (`unsafe_code = "forbid"`, clippy all deny, pedantic warn, unwrap and expect and indexing deny, print macros deny outside the CLI output module).
- Licence: the project is Hippocratic License 3.0 (not OSI). Permissive and MPL-2.0 dependencies are fine (MPL marked: `nucleo-matcher`, `termina` used under MIT). `license-file = "LICENSE"`, `publish = false`. `deny.toml` allow-list in phase 2. Blocked or not used: `git2` (libgit2 is GPL-2.0 with a linking exception, xdiff is LGPL), `skiller` (SUL-1.0, read only), cargo-binstall libraries (GPL-3.0), `bacon` (AGPL), `slint` (GPL). Distribution: Homebrew core, Debian main and probably Fedora are closed by the licence; AUR and `just install` (`cargo install --path Crates/Cli --locked`) are the channels.
- Tools: nextest and just are installed; `cargo-deny`, `tokei`, `typos`, `taplo`, `cargo-shear`, `hyperfine`, `samply` will inshallah come through mise (prebuilt binaries, not `cargo install`).

### 8. The AGENTS.md of a project (2026-10-10, asked for by the user)

The user wanted every new project to start with the same `AGENTS.md` (five rules for coding agents), the text to be changeable later in one place, an easy way to see which projects have it and which are out of date, a place on the interface to push a new text, and the rest of each file left free for the project's own sections.

Options weighed:

| Option | For | Against | Decision |
|---|---|---|---|
| A. One marked block inside each `AGENTS.md`, the text in the vault | The text is inline, so every agent tool reads it; the project's own sections stay its own; one command updates all; a read-only check exists | Needs its own plan, write and undo for a file (the backup store knew only skill folders) | **Chosen** |
| B. The text in a mandatory skill, `AGENTS.md` holds only a pointer | Updates ride the existing sync, push, interface and web with no new code | An agent has to choose to follow a pointer; inline text is far more reliable | Rejected as the only mechanism |
| C. Mirror the whole file like a skill | Simplest | Each sync would overwrite the project's own sections | Rejected |
| D. Fold it into `sync` and `push` as an extra item | Natural to use | The plan and apply code is skill-folder shaped and the web JSON depends on it; can still be added on top of A, the engine is the same | Later, if wanted |

Choices inside A: the markers carry a checksum of what was written, so an out of date block (the text changed) is told from one edited by hand (never replaced unless `--force`); the text is `<vault>/AGENTS.md` when there is one and the built-in five rules otherwise (the vault file is how it is changed without a release); the built-in text names three skills, so `init` and `agents add` stop with a hard error when the project would not have them, while a vault text is not checked; the markers are in the very first file written because an inline copy without them could not be recognised later; the file is created only where none exists (`init`, `agents add`), never overwritten, and `add` for skills does not write it. Contract rows Q57 to Q62.

Not done: the web view does not show or write AGENTS.md (the JSON API and the pages have no row for it); `doctor` does not check the vault text; `status` does not say how many projects have an out of date block.

## Overruled or adjusted research recommendations

1. **Git index: one subprocess per run instead of `gix-index`.** The engine report recommends `gix-index` (0.10 ms, 83 crates, 136 CPU-s compile, +0.6 MB) and itself offers one `git ls-files` spawn (1.9 ms, zero crates) as the alternative. Speed is no argument (the Go cost came from 395 spawns, one per target, not from spawning). The cost rule of the requirements ("a crate that saves 30 lines but adds 40 dependencies is a no") decides: the spawn wins. The rule "no subprocess" in the requirements was written from the Go flaw and is relaxed to "no spawn per target". Revisit `gix-index` only if the tool must run where `git` is absent.
2. **Registry not in the first cut.** The walk is 0.4 s warm with `ignore`; a registry saves that but can silently miss a project cloned elsewhere. It comes in phase 5 after the cold scan is measured, with `--rescan`, a TTL and a printed "N projects from cache, age 3 h" line.
3. **Provenance lock and `sha2` deferred** to phase 5, following the user's "maybe later".
4. **Web feature default.** Decided with stage 1: the Cli feature `web` is on by default (a plain `cargo install` has `skillmirror web`); `--no-default-features` leaves out `axum` and `tokio`. Measured cost of the server stack: about 12 s of compile time, 100 MB of build folder and 360 MB of peak memory, 1.6 MB of binary (`Docs/Setup/build_Resources.md`).
5. **`musl` builds** are not planned: nothing is measured about allocator speed, and the only channel is a native `cargo install`.
6. **Frontmatter strictness applies to `doctor`, `list` and groups, not to sync.** Two real vault skills (`post-scheduler`, and the old vault `secrets`) have an unquoted `: ` in `description`, so every YAML parser rejects them. Sync copies files and never parses `SKILL.md`, so the 395 installs keep working; `doctor` reports the fix. The project copy of the new `secrets` skill is valid.

## Rulings still open (my default applies unless the user objects)

| # | Question | Default |
|---|---|---|
| 1 | May I commit on `rust-rewrite` at phase boundaries (local only, no push)? | needs a yes; nothing is committed yet |
| 2 | Keep the name `skillmirror`? | yes; a rename later is a one-file change plus docs |
| 3 | Quote the two invalid vault descriptions (`post-scheduler`, `secrets`) after the user reviews the project copies? | yes, in the vault, by hand, after approval |
| 4 | Drop the "every config key from the environment" behaviour of the Go tool? | yes, only vault and root |
| 5 | Provenance lock location when it is built: in the project, or in `$XDG_STATE_HOME`? | project (works over SSH and across machines) |
| 6 | Run `echo 3 \| sudo tee /proc/sys/vm/drop_caches` once so the cold scan can be measured? | ask when the first Rust scan exists |
| 7 | Remove the Astro leftovers in `Docs/Architecture/` and the empty `AI/` and `Matters/` folders at the docs pass? | yes, phase 7 |

## Phases

| Phase | Content | Exit criterion |
|---|---|---|
| 0 | branch, ignored scratch folder, doc-start structure, workspace skeleton, frozen Go oracle with golden outputs | done except the oracle (delegated) |
| 1 | behavior contract and this decision | contract reviewed by the lead |
| 2 | core engine: config, vault discovery, scan, plan, apply, events, with parity tests | plan and apply output equals the oracle on the golden scenarios |
| 3 | CLI: `sync`, `push`, `list`, `delete`, `migrate`, `--json`, `--check`, completions | parity job green for these commands |
| 4 | TUI port from the prototype, parity first | every Go screen present, PTY flood test green |
| 5 | new features: groups, profiles, `add`, `init`, `status`, `diff`, `undo`, `doctor`, targets, registry, lock | each with tests |
| 6 | UX pass: review before apply, live scan, detail pane, fuzzy filter; web stage 0 | user acceptance |
| 7 | docs pass: architecture docs, README, doc-start check, Astro leftovers removed | `grep` finds no old name outside the legacy detector |
| 8 | cutover: delete Go in one commit, release build, push, PR | user approval |

Work split: the lead codes the engine and CLI personally. By the user's rule (few agents) parallel work goes to the two peer Claude Code sessions the user opened, each in its own git worktree and on a distinct crate or document, never on the same files.
