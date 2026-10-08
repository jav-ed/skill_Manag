# Rust stack: CLI and quality

Decision: clap 4.6 (derive, env) with clap_complete for parsing and completions; thiserror in every crate with a small hand-written `Hint` trait and a roughly 80-line report printer in the binary (no anyhow, miette or color-eyre in the product); anstream plus anstyle for colour (already inside clap's tree, so free); serde_json for `--json`; indicatif for the scan progress line; nextest with assert_cmd, assert_fs, predicates, insta, rstest and proptest for tests; hyperfine and samply for performance work; clippy through a `[workspace.lints]` table plus cargo-deny for supply chain; tokei behind a 10-line script for the 300-line gate; the default rust-lld linker. Everything targets Linux only (x86_64 and aarch64, gnu and musl), so there is no Windows or macOS terminal, directory or linker handling. The evidence is mostly measured on this machine (scratch builds of eight parsers, six error stacks and a 63-package "kitchen sink" binary under seven release profiles, a flag/env/file precedence proof, pedantic clippy on 50k lines of ripgrep) plus source and docs reads in the cargo registry. Tools I could not install (hyperfine, cargo-deny, cargo-audit, cargo-llvm-cov, tokei, samply, mold, sccache, cargo-vet, cargo-shear, cargo-bloat) are judged from docs and marked "not installed, not measured". No library candidate is licence-blocked; two dev tools (cargo-binstall GPL-3.0-only, bacon AGPL-3.0) and one hidden vendored dependency (libgit2, GPL-2.0 with linking exception) are listed in the licence section.

## Evidence and method

- Machine: 12 cores, Manjaro, rustc 1.98.1 (rustup reports 1.99.0 available, released 2026-09-28), cargo-nextest 0.9.143, just 1.54.0, hk 1.56.0, mise 2026.8.10 installed. Other agents were building in parallel (load average 10 to 38), so wall times are unreliable. I report CPU seconds (user plus sys of child processes, via a small Python `getrusage` timer) and call out where wall time was read on a quiet machine.
- Versions and dates come from the crates.io API and `gh api`; dependency counts are unique `name version` pairs from `cargo tree -e normal` excluding the probe crate itself.
- `~/.cargo/config.toml` sets `debug = false` in every profile (disk-space preference). It does not block a custom `[profile.profiling]` with `debug = "line-tables-only"`: I built one and `readelf` showed `.debug_line` present (release: absent).
- Scratch files: `scratchpad/cli/{parse,err,kitchen,trees,prec,lintcheck}`. Nothing was installed and no repo file other than this one was written.

## Options considered

### Argument parsing

Same CLI (root flags `--vault --root --dry-run`, subcommands sync, list, push, delete with `--project`) built with each parser. CPU seconds are the median of three clean release builds.

| Parser | Version (release date) | Licence | Deps | Clean CPU-s | Stripped binary | Notes from running it |
|---|---|---|---|---|---|---|
| clap derive + env + clap_complete | 4.6.7 (2026-09-14), 1.2 G downloads | MIT OR Apache-2.0 | 18 | 49 | 1.05 MB | grouped help, `[env: X=]` shown, "tip: a similar argument exists: '--dry-run'", exit 2 |
| clap derive, minimal features | same | same | 10 | 40 | 0.80 MB | no colour, no suggestions |
| clap builder only | same | same | 4 | 26 | 0.76 MB | no derive macro, hand-written `Arg` tree |
| bpaf derive | 0.9.28 (2026-09-17) | MIT OR Apache-2.0 | 6 | 13.5 | 0.59 MB | terse help; usage line `[COMMAND ...]`; env shown as `[env:X: N/A]` |
| bpaf combinatoric | same | same | 1 | 7.5 | 0.59 MB | same, no macros |
| argh | 0.1.19 (2026-03-16) | BSD-3-Clause | 11 (pulls serde) | 18 | 0.39 MB | no env, no global flags, no completions |
| lexopt | 0.3.2 (2026-02-28) | MIT | 1 | 0.75 | 0.37 MB | no help generation, hand-written everything |
| pico-args | 0.5.0 (2022-06-04) | MIT | 1 | 0.75 | 0.37 MB | last release over four years ago |

Hello-world baseline: 0.35 MB stripped. The previous Go binary is 10.4 MB (`skill_manag`, with debug info). Findings:

- Compile cost is a clean-build cost only. After touching `main.rs`, every variant rebuilds in 0.3 CPU-s or less in dev, so the derive macro does not slow the edit loop. clap derive costs about 36 CPU-s more than bpaf derive on a clean build (5.8 s wall in the first, least loaded run, on 12 cores) and about 0.45 MB more binary. For a tool whose release builds are minutes anyway, neither number decides.
- Help quality is where clap wins. bpaf is close but its derive checks structure at run time: my first `delete` struct put a positional before an option and `--help` panicked with "bpaf usage BUG: all positional and command items must be placed in the right most position" (bpaf-0.9.28 `src/meta.rs:92`). clap catches the same class in `Command::debug_assert`, which I recommend as a unit test (`Cli::command().debug_assert()`).
- Global flags: clap `global = true` accepts `list --root /r` (tested). argh rejected the same line ("Unrecognized argument: --root"). bpaf accepted it.
- Env fallback is built into clap (`env` feature, `Arg::env` at `clap_builder-4.6.7/src/builder/arg.rs:2205`) and bpaf, absent in argh, lexopt and pico-args.
- Ecosystem: clap has 16.7k stars, 248 M downloads in 90 days and is what Cargo itself uses (`rust-lang/cargo` `Cargo.toml:42-43`: clap 4.6 and clap_complete with `unstable-dynamic`); bpaf (pacak/bpaf) has 462 stars.
- clap features worth knowing (`clap_builder-4.6.7/Cargo.toml:58-96`): defaults are `std color help usage error-context suggestions`; `wrap_help` adds terminal_size, rustix, linux-raw-sys, bitflags (17 deps becomes 23) and is not worth it for short help lines; `unicode` adds unicase and unicode-width. Recommended: defaults plus `derive` and `env`.
- Completions: `clap_complete 4.6.11` ahead-of-time generation is stable and covers bash, elvish, fish, powershell, zsh (`src/aot/shells/shell.rs:15-26`); output on a 4-subcommand CLI was 175 lines for zsh, 236 for bash, 48 for fish. Nushell needs the separate `clap_complete_nushell 4.6.2` (a `nu` binary is installed under `~/.cargo/bin`, though the login shell is zsh). Dynamic completion (`unstable-dynamic`, `CompleteEnv`, `COMPLETE=zsh binary` registration, tested) can complete installed skill names for `delete <skill>`; its docs warn the shell-to-binary protocol is unstable and should be re-sourced on upgrade (module docs at the top of `clap_complete-4.6.11/src/env/mod.rs`), which Cargo accepts but a one-person tool may not want. Ship AOT first; dynamic is a "maybe" for skill-name completion.
- Man pages: `clap_mangen 0.3.3` adds one dependency (roff) and renders a page per subcommand (`generate_to`, `src/lib.rs:128`; my `man` subcommand printed 37 roff lines). Keep it in an `xtask`, not in the shipped binary.
- A real bug my sample exposed: `completions` and `man` failed with "scan root missing" because config resolution ran before dispatch. In the product `--help`, `--version`, `completions` and `man` must dispatch before any config or vault lookup.

### Flag over env over config file

| Approach | Verdict basis |
|---|---|
| clap `env` plus `ArgMatches::value_source` plus a hand-written merge | Zero extra crates. Gives provenance (`CommandLine`, `EnvVariable`) for free, so `doctor` and error hints can say where a value came from. |
| figment 0.10.19 (2024-05-17), `yaml` feature | 20 deps including `serde_yaml 0.9.34+deprecated` and `unsafe-libyaml`; `Env::prefixed` would turn any stray `SKILL_MANAG_*` variable into a key (unknown keys must be hard errors here); read-only. |
| config 0.15.27 (2026-09-30) | Maintained, 15 deps (yaml-rust2, encoding_rs, hashbrown); merges into a value tree, so provenance is lost; read-only. |
| confique 0.4.0 (2025-10-27) | 18 deps including `serde_yaml`; read-only. |

Proof run (`scratchpad/cli/prec`, clap derive plus 12 lines of merge): flag only gives `Flag`; env only gives `Env`; file only gives `File`; flag plus env plus file gives the flag; env plus file gives the env; none gives a hard error naming all three sources and `skill_manag init`; an empty `SKILL_MANAG_ROOT=` or `--root ""` is rejected with exit 2 using `clap::builder::NonEmptyStringValueParser` (so an empty variable cannot silently fall through to the file, matching the no-fallbacks rule). The YAML file itself is read and edited by the engine's YAML crate ([engine_Stack.md](engine_Stack.md)); the CLI only receives a typed, deny-unknown-fields `FileConfig`. Note that viper's `AutomaticEnv` in the Go code (`cmd/root.go`, `initConfig`) also honoured `SKILL_MANAG_MANDATORY` and `SKILL_MANAG_EXCLUDE_DIRS`, which the README never documented; clap's `env` is per argument by design, so the Rust CLI supports exactly the documented variables.

### Errors and reports

Same scenario in each: unknown key at line 3 of `config.yaml` with a "valid keys" next step.

| Stack | Versions (last release) | Deps | Clean CPU-s | Stripped | Observed output |
|---|---|---|---|---|---|
| anyhow + thiserror, hand renderer | 1.0.104 (2026-07-18), 2.0.21 (2026-09-23) | 7 | 14 | 379 KB | `error:` / `caused by:` / `hint:` lines, 3 lines |
| miette, default features | 7.6.0 (2025-04-27) | 10 | 21 | 375 KB | prints a `Diagnostic { ... }` Debug dump plus a note to enable `fancy` |
| miette `fancy` | same | 55 (ICU crates) | 135 | 673 KB | rustc-style snippet with label and `help:` |
| color-eyre | 0.6.5 (2025-05-30) | 24 | 64 | 753 KB | ANSI escapes emitted with `NO_COLOR=1` on a pipe; `Suggestion:` line |
| snafu `Report` | 0.9.2 (2026-07-21) | 7 | 10 | 361 KB | message only, no hint concept |
| annotate-snippets 0.12.16 + anyhow + thiserror | (2026-05-06) | 10 | 27 | 544 KB | rustc-style snippet, plain; hint is a separate line |

- color-eyre: grepping `color-eyre-0.6.5/src` for `NO_COLOR` or `supports_color` finds nothing, and the run above confirms colour on a pipe. NO_COLOR is a stated requirement, so it is out.
- miette: `miette-7.6.0/src/handlers/theme.rs:72` does read `NO_COLOR`, and `#[diagnostic(help(...))]` is exactly "what to run next", but usable output needs `fancy` (55 packages, 135 CPU-s, about 300 KB more than anyhow). Last release is 17 months old; the repository's recent commits are `AGENTS.md` edits, last code change 2025-09-29, 91 open issues. Its real strength, source snippets for YAML errors, is available for 3 dependencies from annotate-snippets 0.12.16 (released 2026-05-06; rust-lang org; Cargo itself depends on `0.12.15`).
- Design recommended: every crate uses thiserror. Each core error enum implements `trait Hint { fn hint(&self) -> Option<String>; }`; the CLI's top-level `CliError` enum wraps them with `#[error(transparent)]` and forwards `hint()` by `match`, so no downcasting is needed (my anyhow demo needed `downcast_ref`). The binary's `report.rs` prints `error: <display>`, one `caused by:` line per `source()` link, then `next: <hint>` to stderr through anstream, and returns `ExitCode`. anyhow stays allowed in tests and the xtask only.
- Exit codes (proposal): 0 clean; 1 drift found by `--check`; 2 usage error (clap's own); 3 runtime hard error; 4 partial failure (some targets applied, some failed). With `panic = "abort"` a bug exits via SIGABRT (134), not 101.

### Terminal output, progress, tables, logging

- Colour: anstream 1.0.0 (7 deps, all already pulled by clap) decides in `anstream-1.0.0/src/auto.rs:198-217`: `NO_COLOR` wins, then `CLICOLOR_FORCE`, then `CLICOLOR=0`, then terminal plus `TERM` or `CI`. Measured with a sample binary: piped gives plain text; `CLICOLOR_FORCE=1` gives ANSI; `NO_COLOR=1 CLICOLOR_FORCE=1` gives plain; clap's own error text follows the same rules. A global `--color auto|always|never` maps to `anstream::ColorChoice::write_global`. std `IsTerminal` replaces the is-terminal crate (anstream carries a polyfill).
- Rejected colour crates: owo-colors 4.4.0 (0 deps, but no detection; with `supports-colors` 6 deps), colored 3.1.1 (MPL-2.0, global state), supports-color 3.0.2 (last release 2024-11), console 0.16.6 (3 deps, honours `NO_COLOR` only inside `unix_term.rs:30`, so it works but duplicates anstream; it arrives anyway through indicatif).
- Progress: indicatif 0.18.6 (2026-07-01, 6 deps: console, libc, portable-atomic, unicode-width, unit-prefix). `ProgressDrawTarget::is_hidden` is `!term.is_term()` (`indicatif-0.18.6/src/draw_target.rs:136-140`) and dumb terminals are skipped (`:80`), so non-TTY runs print no progress, which is the required "plain output without a TTY". Wrap it in one `Progress` type so the TUI and CLI can share the event stream.
- `--json`: serde 1.0.229 plus serde_json 1.0.151 (11 deps, of which the syn chain is shared with clap_derive and thiserror, so paid once). The engine's typed event enum gets `#[derive(Serialize)]` with `#[serde(tag = "event", rename_all = "snake_case")]`; `--json` then equals one `serde_json::to_writer` plus newline per event (JSON Lines) and a single document for `list` and `status`. Add a `"schema": 1` field to the first line.
- Tables: comfy-table 8.0.1 pulls crossterm and parking_lot by default (16 deps; `default-features = false` gives 3); tabled 0.22.0 has a proc-macro derive (12 deps); tabwriter 1.4.1 (2 deps) miscounts widths once ANSI is embedded. Recommendation: hand-format with `unicode-width` (already in the tree); comfy-table minimal is the fallback if `status` grows a matrix.
- Prompts for `--yes` confirmations: dialoguer 0.12.0 (14 deps), inquire 0.9.4 (32 deps) versus a 12-line stdin `confirm()`; hand-written wins, and the TUI covers interactive flows.
- Logging: none in v1. The engine already emits typed events and the CLI maps `-v` to "print events to stderr". `tracing 0.1.44` plus `tracing-subscriber 0.3.23` is 18 deps (15 minimal); `log` is 1 dep but `env_logger 0.11.11` is 17 (regex, jiff; 10 minimal). `tracing` is a "maybe" behind an off-by-default `profiling` feature for span timings later.
- Config location: dirs 7.0.0 pulls `option-ext` (MPL-2.0) through dirs-sys; etcetera 0.11.0 (MIT OR Apache-2.0, 2 deps) with `choose_base_strategy()` gives the XDG layout on Linux (`etcetera-0.11.0/src/lib.rs:57`), which keeps `~/.config/skill_Manag/vault`. With Linux as the only platform, `$XDG_CONFIG_HOME` falling back to `~/.config` is about 10 hand-written lines, so etcetera is a "maybe" and no platform-specific directory logic is needed.

### Tests

| Tool | Version (release) | Dev deps | Verdict basis |
|---|---|---|---|
| cargo-nextest | 0.9.146 (2026-09-21); installed 0.9.143 | binary | process per test; no doctests ("run `cargo test --doc` separately", nexte.st/docs/running); `default-filter` per profile exists (`site/src/docs/selecting.md:90-113`) so the parity suite can be excluded by default |
| assert_cmd + predicates | 2.2.2 (2026-05-11), 3.1.4 | 18 | `cargo_bin_cmd!` reads `CARGO_BIN_EXE_<name>`, which survives a custom `build-dir`; the path-guessing `cargo_bin()` has a per-Cargo-version caveat (`src/cargo.rs:221-231`, `src/macros.rs:85-92`) |
| assert_fs | 1.1.4 (2026-05-26) | 27 (34 together with assert_cmd) | temp dirs with `copy_from`; dev-only, heavier than tempfile (9) because it pulls ignore and globset |
| insta | 1.49.0 (2026-10-03) | 12; with `filters` adds regex and strip-ansi-escapes | `filters` strips ANSI and redacts paths with regexes; gitui and cargo-dist depend on it (their `Cargo.toml`); needs the `cargo-insta` binary for review |
| snapbox | 1.2.2 | 27 (`cmd`,`dir`) | used by Cargo and mdBook; directory `subset_eq` is one-directional and ignores modes (`src/assert/mod.rs:222`, `src/dir/diff.rs:32-60`), so it cannot prove parity alone |
| trycmd | 1.2.1 | 54 | markdown or TOML case files; rayon and toml_edit make it heavy |
| rstest | 0.27.0 (2026-09-06) | 14 (`default-features=false`), 27 default | parametrised fixtures for the many near-identical parity cases |
| proptest | 1.11.0 (2026-03-24) | 14 minimal, 26 default | plan then apply then plan is empty, convergence, atomic swap under injected failure |
| tempfile | 3.27.0 (2026-03-11) | 9 | already inside most trees |
| cargo-llvm-cov | 0.9.1 (2026-09-06) | binary; needs the `llvm-tools` rustup component (not installed here) | optional `just cov`, not a hook; not installed, not measured |
| cargo-mutants | 27.1.0 (2026-06-02) | binary | optional, for plan and diff code; slow; not installed |

Differential parity test against the frozen Go binary:

- Facts that shape it: the Go CLI has only two non-interactive paths, `--dry-run` and `delete <skill> [--project] [--dry-run]`; a real sync is TUI-only (`cmd/tui/sync.go:258`). Go's dry-run prints projects in map order: 8 runs on `internal/testdata` produced 3 distinct outputs. The Go fixtures need `.git` directories added at run time (`internal/fixture_test.go:80-102`) and the Rust vault must be a git repo, so the harness runs `git init && git add -A` in the vault copy.
- Oracle: build the Go binary once from a tag (`just parity-oracle`, Go is already a mise tool). For apply parity add a 40-line `cmd/oracle/main.go` on that tag (inside the module, so it can import `internal`) that runs `SyncSkill` for every target and exits; without it only dry-run and delete can be compared.
- Harness (`tests/parity/`, files under 300 lines each): copy `internal/testdata` into two `assert_fs::TempDir`s, run both binaries with `env_clear`, `HOME`, `NO_COLOR=1`, explicit `--vault --root`; normalise by `insta` filters (strip ANSI, tempdir to `[ROOT]`) and by sorting project blocks; compare exit codes, normalised stdout, and a 30-line manifest of the resulting tree (relative path, size, blake3, mode) in both directions.
- Policy: expected differences live in `known_divergences.toml` with a reason each, and the test fails if a listed divergence disappears (entries cannot rot). If `SKILL_MANAG_GO_ORACLE` is unset the parity test fails with a hint instead of skipping (no fallbacks); a nextest profile `parity` sets `default-filter = 'all()'`, while `default` excludes `binary(parity)`.

### Benchmarks and profiling

| Tool | Version (release) | Verdict basis |
|---|---|---|
| hyperfine | 1.21.0 (2026-10-06), MSRV 1.97 | end-to-end metric that matters here (warm and cold full scan); install the prebuilt binary through mise, not `cargo install`. Cold runs need `drop_caches`, which needs root: a documented manual step. Not installed, not measured. |
| divan | 0.1.21 (2025-04-10) | 18 deps, `#[divan::bench]`; fine for micro-benchmarks of hash and diff; release is 18 months old, repo active (2026-07) |
| criterion | 0.8.2 (2026-02-04) | 40 deps (48 default); heavier than divan for no gain here |
| samply | 0.13.1 (2025-02-01, repo commits to 2026-10-05) | needs `perf_event_paranoid <= 1` (here 2; a one-time user `sysctl`, README quotes it), plus a `profiling` profile |
| flamegraph 0.6.14 | needs `perf`, which is not installed | covered by samply |
| dhat 0.3.3 (2024-02) | 30 deps; heap profiling; maybe, only if allocation shows up in samply |

### Lint and supply chain

- Clippy lints: every lint name used in this section and in the Recommendation was verified against clippy 1.98.1 by building a scratch crate with the `[lints]` table (only `string_to_string` was reported as removed). Real-world reference tables: `casey/just` (pedantic deny plus 13 allows including `too_many_lines`, `needless_pass_by_value`, `similar_names`), `astral-sh/uv` (pedantic warn at priority -2, 16 allows including `must_use_candidate`, `missing_errors_doc`, `module_name_repetitions`, plus `print_stdout`, `print_stderr`, `exit`, `dbg_macro`, `get_unwrap`, `use_self` as warnings), television (pedantic plus at least 4 allows: `must_use_candidate`, `too_many_lines`, `missing_panics_doc`, `missing_errors_doc`).
- Pedantic noise on real code: `cargo clippy --workspace --lib --bins --tests -W clippy::pedantic` on `Repos/ripgrep` (50,356 lines) reported 1,437 pedantic-only hits across 55 lints on top of 768 default hits. Top: must_use_candidate 322, doc_markdown 173, missing_errors_doc 142, uninlined_format_args 97, explicit_iter_loop 71, elidable_lifetime_names 50, inline_always 42, redundant_closure_for_method_calls 41, manual_let_else 38, match_same_arms 34, enum_glob_use 30, similar_names 27. Greenfield code can satisfy most; allow only `must_use_candidate`, `missing_errors_doc`, `missing_panics_doc`, `module_name_repetitions`, `similar_names`, `needless_pass_by_value`, `struct_excessive_bools` (a CLI args struct legitimately has many bools). Keep `too_many_lines` (9 hits in 50k lines): it complements the file gate.
- Restriction lints chosen to encode "hard errors only" (all verified to exist): `unwrap_used expect_used panic todo unimplemented indexing_slicing string_slice unwrap_in_result get_unwrap` (deny), `let_underscore_must_use unused_result_ok map_err_ignore` (deny, these catch swallowed errors), `dbg_macro print_stdout print_stderr exit` (deny in lib crates; `allow` in the CLI crate's `output` and `main`), `allow_attributes_without_reason`, `undocumented_unsafe_blocks`, `mem_forget`. Rust lints: `unsafe_code = "forbid"` in every crate (reconsider only where the engine needs reflink or `libc`), `unreachable_pub`, `unused_must_use = deny`, `missing_docs` warn for libs. `clippy.toml` can add `disallowed-methods` for `std::fs::remove_dir_all` and `std::fs::write` outside the engine's atomic-swap module, which turns the "no data loss" rule into a compile-time check. Use `-D warnings` only in `just lint` and hooks, not in `Cargo.toml`.
- cargo-deny 0.20.2 (2026-07-09, MIT OR Apache-2.0, repo active 2026-10-07): checks advisories, licences, bans, sources in one run, and hk 1.56.0 ships a `cargo_deny` builtin (the pinned 1.45.0 does not, see Risks). Config keys verified against the current docs: `[advisories]` has `unmaintained` (default `all`), `unsound`, `yanked`, `maximum-db-staleness` (default P90D, so a stale DB is a hard error), and the old `vulnerability`, `notice`, `severity-threshold` keys now error; `[licenses]` lost `unlicensed`, `deny`, `copyleft`, `default`. Not installed, not run.
- cargo-audit 0.22.2: redundant, because cargo-deny reads the same RustSec database. cargo-vet 0.10.2: `cargo vet init` can start from exemptions, but every dependency update then needs an audit or an import approval (`mozilla.github.io/cargo-vet/how-it-works`); no value for a one-person tool with about 100 dependencies. cargo-geiger 0.13.0: report only; `unsafe_code = "forbid"` plus the deny list does the job.
- Unused dependencies: cargo-shear 1.14.0 (2026-09-22, MIT) parses with rust-analyzer's `ra_ap_syntax` (README), no nightly unless `--expand`; cargo-machete 0.9.2 is regex-based and "fast yet imprecise"; cargo-udeps 0.1.61 needs nightly. Pick shear, with `[package.metadata.cargo-shear] ignored` for false positives.
- cargo-bloat 0.12.1 (last release 2024-05-10) is a "maybe" for binary-size questions (`cargo bloat --release --crates`); not measured.
- Other hygiene: typos 1.51.1 and taplo 0.10.0 are hk builtins and mise registry names; rustfmt with the default style and edition 2024; `cargo-outdated 0.19.0` is already installed.

### Build profile, linker, MSRV

Kitchen-sink binary (63 packages: clap with dynamic completions, clap_mangen, anyhow, thiserror, serde_json, anstream, indicatif, ignore, rayon, blake3), seven clean release builds. "Walk" is a parallel `ignore` walk over `1_Code` (the real scan root), "hash" adds blake3 over every file under `.agents/skills`; CPU is the median of 9 warm runs.

| Profile | Settings added | Clean build CPU-s | Binary | Walk CPU ms | Hash CPU ms |
|---|---|---|---|---|---|
| p0 | cargo release default (user config: no debug) | 275 | 4.12 MB | 3960 | 4875 |
| p1 | `strip = "symbols"` | 285 | 3.13 MB | 4109 | 5148 |
| p2 | `lto = "thin"` | 302 | 3.11 MB | 3903 | 4764 |
| p3 | `lto = "fat"`, `codegen-units = 1` | 241 | 2.58 MB | 3662 | 4211 |
| p4 | `panic = "abort"` | 198 | 2.35 MB | 3851 | 4297 |
| p5 | `opt-level = "s"` | 179 | 1.88 MB | 3871 | 4886 |
| p6 | `opt-level = "z"` | 160 | 1.78 MB | 4031 | 4728 |

A second, interleaved runtime pass (11 rounds, median CPU, machine still loaded): walk 3965 (p0), 3846 (p3), 3724 (p4), 3974 (p5), 4130 ms (p6); hash 4988, 4728, 4650, 4830, 4987 ms. The spread is 7 to 11 percent, so optimiser settings are within noise for this I/O-bound tool; p4 is about 6 percent below p0 on both. Wall medians (1.7 to 2.8 s) and minima moved by factors of two between rounds and are not usable. Build CPU varies by about 15 percent between runs (p3 at 241 was cheaper than p0 at 275), so only the binary-size column and the incremental numbers are firm. Incremental release rebuild after touching `main.rs` (median of 3, CPU-s with wall in brackets on the loaded machine): p0 9 (7 s), p2 127 (35 s), p3 71 (135 s), p4 57 (104 s). Fat LTO with one codegen unit makes every release iteration cost one to two minutes, so it belongs in a separate `dist` profile, not in `release`, which stays the quick profile for `hyperfine` loops.

- Real projects agree: ripgrep `release-lto` is fat, cgu 1, abort, strip (`Repos/ripgrep/Cargo.toml:86-96`); yazi lto, cgu 1, abort, strip; television fat plus cgu 1; gitui uses `opt-level = "z"`.
- `panic = "abort"` effects: smaller and faster, but destructors do not run on a panic (leftover staging directories; `doctor` should list them), rayon or thread panics end the process, and the exit status is SIGABRT. Cargo ignores `panic` for the test profile, so tests still unwind.
- Linker: rustc 1.98.1 on x86_64 Linux already links with rust-lld (`cc -B.../gcc-ld -fuse-ld=lld`, seen in `--print link-args`). Measured on the kitchen binary (dev profile, touch `main.rs`, rebuild and relink, median of 7): rust-lld 1026 ms wall and 882 ms CPU, GNU bfd (`-C link-arg=-fuse-ld=bfd`) 1299 ms and 1339 ms, so the default is already 21 percent faster in wall time and 34 percent lighter in CPU than the classic linker. A whole edit-build cycle is about one second either way. mold 3.0.0 (MIT, mise name `mold`) is not installed and not measured; with a one-second cycle there is little left to win, so it is not needed. sccache 0.18.0 adds little for a single machine whose clean builds take 5 to 60 seconds of wall time on 12 cores.
- Targets (Linux only, per the requirements doc): `x86_64-unknown-linux-gnu` is the reference build and the only target installed here (`rustup component list --installed` shows `rust-std` for it and for `wasm32-wasip2` only), so the aarch64 and musl targets, and any speed difference between glibc and musl, are not installed and not measured. Windows `cfg` branches, Windows terminal modes, macOS directory rules and a cross-platform CI matrix are out of scope. Make the policy a hard error with `#[cfg(not(target_os = "linux"))] compile_error!("skill_manag supports Linux only");` in the lib root, so a stray non-Linux build cannot silently produce a half-working binary (matches the no-fallbacks rule). Unix APIs (`std::os::unix::fs::PermissionsExt`, symlinks, `renameat2` through rustix's safe `renameat_with`, which keeps `unsafe_code = "forbid"` intact where raw libc calls would not) are then used freely.
- MSRV and edition: edition 2024 (needs 1.85; Cargo defaults to resolver 3 there). The highest declared MSRV in the prototype tree is 1.88 (`ignore 0.4.33`, `globset 0.4.20`, also `annotate-snippets` 0.12.16 and mdBook), so `rust-version = "1.88"`. Do not set it lower than the real floor: with resolver 3 the resolver then picks older, MSRV-compatible dependency versions (Cargo reference example: `rust-version = "1.62"` resolves clap to 4.0.32). Policy: `rust-version` equals the real floor, bumped only when a dependency forces it, verified by `just msrv` (`rustup run 1.88 cargo check --locked`); commit `Cargo.lock`; pin the dev toolchain in `rust-toolchain.toml` (channel plus `clippy`, `rustfmt`). Tool MSRVs (nextest 1.91, hyperfine 1.97) matter only if built from source, so install prebuilt binaries.

## Recommendation

Product crates: `clap 4.6` (`derive`, `env`), `clap_complete 4.6` (AOT only at first), `thiserror 2`, `serde` plus `serde_json`, `anstream 1`, `anstyle 1`, `indicatif 0.18`, `etcetera 0.11` (maybe). Dev crates: `assert_cmd`, `assert_fs`, `predicates`, `insta` (`filters`), `rstest` (no default features), `proptest` (`std` only), `tempfile`. xtask only: `clap_mangen`, `anyhow`. Tools: nextest, cargo-deny, tokei, typos, taplo, cargo-shear, hyperfine, samply (cargo-llvm-cov, cargo-insta, cargo-bloat optional).

```toml
[workspace]
resolver = "3"
[workspace.package]
edition = "2024"
rust-version = "1.88"      # real floor: ignore 0.4.33, globset 0.4.20
license-file = "LICENSE"   # HL3 variant, no SPDX id
publish = false
[workspace.lints.rust]
unsafe_code = "forbid"
unreachable_pub = "warn"
unused_must_use = "deny"
[workspace.lints.clippy]
all = { level = "deny", priority = -2 }
pedantic = { level = "warn", priority = -1 }
must_use_candidate = "allow"
missing_errors_doc = "allow"
module_name_repetitions = "allow"
unwrap_used = "deny"
expect_used = "deny"
indexing_slicing = "deny"
let_underscore_must_use = "deny"
print_stdout = "deny"     # CLI output module re-allows it
# release stays default (fast to rebuild, used by hyperfine); packaging uses `cargo build --profile dist`.
[profile.dist]
inherits = "release"
lto = "fat"
codegen-units = 1
panic = "abort"
strip = "symbols"
[profile.profiling]
inherits = "release"
debug = "line-tables-only"
strip = "none"
```

Each member crate opts in with `[lints] workspace = true` and inherits package fields (`edition.workspace = true`, `rust-version.workspace = true`, `license-file.workspace = true`, `publish.workspace = true`).

Keep `opt-level = 3`: the tool is dominated by directory-walk syscalls, so `s` and `z` buy size (20 to 24 percent) at no measured walk cost for `s` (5 percent for `z`) but a 10 to 14 percent hash cost; size is not a goal against a 10 MB Go binary.

## The 300-line gate

Pick tokei 15.0.0 (2026-09-06, MIT OR Apache-2.0, 15k stars, mise name `tokei`). Its Rust definition treats `//`, `///`, `//!` and nested `/* */` as comments and understands normal and raw strings (`languages.json`, Rust entry), and one tool covers Go, TypeScript and Rust. An `xtask` lexer would have to be maintained by us; a shell loop with `grep -v '^\s*//'` miscounts block comments and multi-line strings. Not installed and not run: the report shape is read from tokei's `src/stats.rs` (`Report { stats, name }`, `CodeStats { blanks, code, comments, blobs }`) and the recursive `jq` below matches it; the first run must confirm. Embedded blobs (such as doc-test code) are reported under `blobs`, not in `code`.

```bash
#!/usr/bin/env bash
# Fails when a source file has more than 300 lines of code (comments and blanks excluded).
set -euo pipefail
limit=300
[ "$#" -gt 0 ] || { echo "usage: loc_gate.sh <files...>" >&2; exit 2; }
over=$(tokei --files --output json "$@" \
  | jq -r --argjson limit "$limit" '.. | objects | select(has("stats") and has("name")) | select(.stats.code > $limit) | "\(.stats.code)\t\(.name)"')
[ -z "$over" ] || { printf 'files over %s code lines:\n%s\n' "$limit" "$over" >&2; exit 1; }
```

Calibration with a reference Rust counter I wrote and unit-checked on 8 tricky cases (nested comments, raw strings, lifetimes): 13 of 57 files in clap_builder 4.6.7, 4 of 16 in clap_derive, 3 of 18 in clap_complete, 2 of 12 in anyhow, 10 of 28 in bpaf and 8 of 11 in indicatif exceed 300 code lines. The rule is much stricter than mature-crate practice, so modules must be small and unit tests must live in sibling files (`#[cfg(test)] mod tests;`), because inline test modules count. Open question: the coding skill does not say whether blank lines count; tokei counts code lines only.

## Proposed just and hk wiring

The repo root `justfile` currently holds only `default` and `scratch-clean` (scripts live under `Code/Development/`), and there is no `hk.pkl` yet; the shared baseline uses kebab-case recipe names (`system-tools-install`). Proposed additions (not created); the gate script would sit at `Code/Development/Scripts/loc_gate.sh`:

| Recipe | Command |
|---|---|
| `fmt`, `fmt-check` | `cargo fmt --all` (`-- --check`) |
| `lint` | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| `test` | `cargo nextest run --workspace --locked` then `cargo test --doc --workspace --locked` |
| `deny` | `cargo deny --locked check` |
| `unused-deps` | `cargo shear` |
| `loc-gate` | `bash Code/Development/Scripts/loc_gate.sh $(git ls-files '*.rs' '*.go' '*.ts')` |
| `msrv` | `rustup run 1.88 cargo check --workspace --locked` |
| `parity-oracle`, `parity` | build Go tag into `Repos/go_oracle/`; `cargo nextest run --profile parity -E 'binary(parity)'` |
| `bench`, `profile`, `cov`, `bloat` | `hyperfine --warmup 3 ...`; `cargo build --profile profiling` then `samply record`; `cargo llvm-cov nextest`; `cargo bloat --release --crates` |
| `dist` | `cargo build --profile dist --locked` (fat LTO, one codegen unit, abort, strip; Linux targets only) |
| `man`, `completions` | `cargo xtask man`; `cargo run -q -- completions zsh` |
| `check` | `fmt-check lint test deny unused-deps loc-gate` |

| Hook | Steps (staged files unless noted) |
|---|---|
| pre-commit (seconds) | `cargo_fmt` with `fix`, `typos`, `taplo`, `loc-gate` on staged sources |
| pre-push (minutes) | `just lint`, `just test`, `just deny` and `just unused-deps` when `Cargo.toml` or `Cargo.lock` changed |
| never in a hook | `parity`, `bench`, `profile`, `cov`, `msrv`, `bloat` |

mise pins (all resolved with `mise ls-remote` on 2026-10-07, none installed): `hyperfine 1.21.0`, `tokei 15.0.0`, `taplo 0.10.0`, `cargo-insta 1.49.0`, `mold 3.0.0`, `sccache 0.18.0`, `aqua:EmbarkStudios/cargo-deny 0.20.2`, `aqua:nextest-rs/nextest/cargo-nextest 0.9.146`, `aqua:taiki-e/cargo-llvm-cov 0.9.1`, `aqua:mstange/samply 0.13.1`; `typos` resolves too, but mise hides its two newest releases through `minimum_release_age` (the printed version was not captured); cargo-shear and cargo-bloat have no registry name and would use `github:Boshen/cargo-shear`. The baseline owns Go and Node through mise but Rust through rustup (`~/.cargo/bin`); keep it that way and add `rust-toolchain.toml` instead of a mise Rust entry.

## Good but licence-blocked

Project licence is Hippocratic License 3.0 with modules (`HL3-BDS-CL-ECO-EXTR-MEDIA-MIL-SV-XUAR`); SPDX knows only `Hippocratic-2.1` and `Hippocratic-3.0-core`, neither OSI-approved, and not this module set.

| Candidate | SPDX verified | Status |
|---|---|---|
| cargo-binstall 1.25.2 | GPL-3.0-only (crates.io, 2026-10-06) | dev installer only; running a GPL binary does not affect our code, but it must never be linked or vendored; not needed (mise aqua backends) |
| bacon 3.26.0 | AGPL-3.0 (crates.io, GitHub) | standalone dev watcher; skip |
| libgit2 (vendored by libgit2-sys 0.18.8 and git2 0.21.0) | crate says MIT OR Apache-2.0; `COPYING` says GPL-2.0 plus a linking exception (GitHub) | hidden: cargo-deny cannot see vendored C. The exception allows static linking and distribution; prefer gix 0.88.0 (MIT OR Apache-2.0). Flag for [engine_Stack.md](engine_Stack.md) |
| colored 3.1.1 | MPL-2.0 (marked) | allowed by policy, avoided |
| option-ext 0.2.0 | MPL-2.0 (marked); pulled by dirs 7.0.0 and directories | allowed, avoided by choosing etcetera |
| grcov 0.10.7 | MPL-2.0 (marked) | dev tool, not needed |
| r-efi 5.3.0 and 6.0.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | UEFI-target only, outside the Linux target set; the OR lets cargo-deny pass on MIT anyway, no LGPL obligation |

No GPL, AGPL, SSPL, BUSL, source-available or non-commercial library was found among the candidates. tokei and cargo-udeps show "Other" on GitHub only because of a dual-licence file layout; crates.io says MIT OR Apache-2.0. I scanned `cargo metadata` of 78 prototype manifests (354 crate versions, 345 crates; `cargo metadata` lists every platform, so the counts also include crates that only build on Windows, macOS or UEFI and that the Linux-only target list in `deny.toml` excludes) and found these SPDX strings: MIT OR Apache-2.0 (182), MIT (53), Apache-2.0 OR MIT (22), MIT/Apache-2.0 legacy slash form (19), Unicode-3.0 (18), Apache-2.0 (15), Unlicense OR MIT (8), Apache-2.0/MIT (6), Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT (5), Zlib OR Apache-2.0 OR MIT (3), Unlicense/MIT (3), BSD-3-Clause (3), MPL-2.0 (2), MIT OR Apache-2.0 OR LGPL-2.1-or-later (2), CC0-1.0 OR MIT-0 OR Apache-2.0 (2), BSD-2-Clause OR Apache-2.0 OR MIT (2), and one each of Zlib, ISC, `(MIT OR Apache-2.0) AND Unicode-3.0`, `(Apache-2.0 OR MIT) AND BSD-3-Clause` (encoding_rs), `CC0-1.0 OR Apache-2.0 OR Apache-2.0 WITH LLVM-exception`, `Apache-2.0 OR BSL-1.0` (ryu, Boost, not BUSL), `MIT OR Zlib OR Apache-2.0`, `0BSD OR MIT OR Apache-2.0`. Every crate carried an SPDX expression, so no `clarify` entry was needed.

`deny.toml` sketch (keys checked against the 0.20.2 docs and its `deny.template.toml`; cargo-deny not installed, not run):

```toml
[graph]
# Linux only: Windows, macOS and UEFI-only crates are never checked.
targets = ["x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu",
           "x86_64-unknown-linux-musl", "aarch64-unknown-linux-musl"]
[advisories]
unmaintained = "all"
unsound = "all"
yanked = "deny"
[licenses]
# Needed by the prototype trees: MIT, Apache-2.0, Unicode-3.0, BSD-3-Clause, ISC, Zlib.
# Defensive (only inside OR expressions today): the rest.
allow = ["MIT", "Apache-2.0", "Apache-2.0 WITH LLVM-exception", "BSD-2-Clause", "BSD-3-Clause",
         "ISC", "Zlib", "0BSD", "Unicode-3.0", "Unlicense", "CC0-1.0", "MIT-0"]
unused-allowed-license = "allow"
exceptions = []            # MPL-2.0 only by crate name here, never globally
confidence-threshold = 0.93
[licenses.private]
ignore = true
[bans]
multiple-versions = "warn"
wildcards = "warn"         # path dependencies between our own crates may trip "deny"; unverified
deny = [{ crate = "git2", use-instead = "gix" }, { crate = "openssl", use-instead = "rustls" }]
[sources]
unknown-registry = "deny"
unknown-git = "deny"
```

Our own licence: set `publish = false` and `license-file = "LICENSE"` on every workspace member, and `[licenses.private] ignore = true`; the cargo-deny docs (`[licenses.private] ignore`) say unpublished workspace members skip the licence-expression check. Do not invent a `LicenseRef-` plus `clarify` entry: cargo-deny supports `LicenseRef-`, but it would have to be added to `allow`, which widens the policy for every crate. The 29 crate versions using the legacy slash separator are expected to parse in cargo-deny's lax mode; unverified until the first run. Redistributed binaries also need third-party notices: `cargo-about 0.9.2` (MIT OR Apache-2.0) is a "maybe" for release packaging.

## Rejected and why

| Rejected | Reason |
|---|---|
| bpaf (runner-up), argh, lexopt, pico-args | bpaf: runtime structure panics, small ecosystem, terse help; argh: no env, globals, completions, drags in serde; lexopt and pico-args: no help; pico-args unreleased since 2022 |
| figment, config, confique | read-only, serde_yaml or 15 to 20 deps, no provenance, env prefix swallows stray variables |
| color-eyre | ignores NO_COLOR, 24 deps, +375 KB |
| miette | needs `fancy` (55 packages, 135 CPU-s); default output is a Debug dump; 17 months since release |
| snafu, eyre, error-stack | no advantage over thiserror plus a `Hint` trait; no hint concept |
| owo-colors, colored, console, supports-color, yansi | anstream is already in the tree and does the detection |
| criterion, flamegraph, cargo-udeps, cargo-machete, cargo-vet, cargo-audit, cargo-geiger, trycmd, snapbox, tabled | see tables: heavier, redundant or nightly-only |
| mold, sccache | rust-lld is already the default; clean builds are short |
| tracing, env_logger, human-panic (32 deps) | typed events are the diagnostics channel |

## Risks and open points

- The pinned hk schema in the Hk skill is 1.45.0; it has no `cargo_deny` builtin (HTTP 404 at that tag), while installed hk 1.56.0 does, and upstream is at 2.5.0. The new `hk.pkl` should amend at least 1.56.0 or define the deny step by hand.
- Wall-clock figures were taken on a loaded machine; rerun the profile table when the machine is idle before quoting speed. cargo-deny, tokei, hyperfine and samply claims rest on docs only.
- `perf_event_paranoid` is 2 here; samply needs it at 1 or lower. That is a system setting and is left to the user.
- The `clap_complete` dynamic protocol and `unstable-ext` are unstable; keep AOT as the baseline.
- The Go code accepted env overrides for every viper key; dropping that is a behaviour change to note in the migration doc. Env variable names may change with the rename; define them as constants in one module so derive attributes and docs share them.
- The 300-line rule counts inline tests and doc-heavy files; a two-file split of every large module is the expected cost. Decide whether blank lines count.
- `panic = "abort"` leaves staging directories behind after a bug; `doctor` should report them.
- Musl release builds: the parallel walk allocates heavily, and musl's allocator is reputed to be slower under threads; this is not measured here (no musl target installed). Re-run the `hyperfine` full-scan benchmark on a musl build before choosing musl over gnu, and ask [name_And_Distribution.md](name_And_Distribution.md) which libc the AUR and release channels need. Cross-building aarch64 needs a cross linker or `cargo-zigbuild`; that belongs to the distribution stream.
- Requirements doc: add "completions, man, help and version must not need config", the exit-code table, and "parity oracle needs a tiny Go `cmd/oracle` because sync apply is TUI-only".

## Related files

- [rewrite_Requirements.md](rewrite_Requirements.md): the constraints and questions this stream answers.
- [engine_Stack.md](engine_Stack.md): YAML, git index and event-channel choices that the CLI config, parity harness and `--json` output depend on.
- [tui_Stack.md](tui_Stack.md): the other consumer of the typed event stream, and the snapshot-testing choice that should match the insta recommendation here.

## Clones

None made. I reused `Repos/ripgrep` (clippy corpus with `--locked` and a scratch target dir; no file content changed) and read `Cargo.toml` files in `Repos/{gitui,television,yazi,mdBook,release-plz,cargo-dist,ratatui}` that other agents had cloned; source reads of 40 crates came from `~/.cargo/registry/src`.

## Verdicts

| Crate or tool | Verdict | One-line reason | Version | Licence |
|---|---|---|---|---|
| clap | yes | derive plus env plus value_source; what Cargo uses | 4.6.7 | MIT OR Apache-2.0 |
| clap_complete | yes | stable AOT completions for five shells | 4.6.11 | MIT OR Apache-2.0 |
| clap_complete `unstable-dynamic` | maybe | skill-name completion; protocol unstable | 4.6.11 | MIT OR Apache-2.0 |
| clap_complete_nushell | maybe | cheap, user has nu | 4.6.2 | MIT OR Apache-2.0 |
| clap_mangen | maybe | xtask only; one extra dependency | 0.3.3 | MIT OR Apache-2.0 |
| bpaf | maybe | 3.6x cheaper clean build, but runtime structure panics | 0.9.28 | MIT OR Apache-2.0 |
| argh | no | no env, globals or completions | 0.1.19 | BSD-3-Clause |
| lexopt | no | no help generation | 0.3.2 | MIT |
| pico-args | no | no help, last release 2022 | 0.5.0 | MIT |
| figment | no | serde_yaml deprecated, env noise | 0.10.19 | MIT OR Apache-2.0 |
| config | no | 15 deps, no provenance | 0.15.27 | MIT OR Apache-2.0 |
| confique | no | serde_yaml, 18 deps | 0.4.0 | MIT OR Apache-2.0 |
| etcetera | maybe | 2 deps, XDG layout; or 10 hand-written Linux lines | 0.11.0 | MIT OR Apache-2.0 |
| dirs, directories | no | pull MPL-2.0 option-ext | 7.0.0, 6.0.0 | MIT OR Apache-2.0 |
| thiserror | yes | typed errors, syn shared with clap | 2.0.21 | MIT OR Apache-2.0 |
| anyhow | maybe | tests and xtask only | 1.0.104 | MIT OR Apache-2.0 |
| miette | no | needs fancy: 55 packages | 7.6.0 | Apache-2.0 |
| color-eyre | no | ignores NO_COLOR | 0.6.5 | MIT OR Apache-2.0 |
| snafu | no | no hint concept | 0.9.2 | MIT OR Apache-2.0 |
| eyre, error-stack | no | nothing over thiserror plus Hint | 0.6.14, 0.8.0 | MIT OR Apache-2.0 |
| annotate-snippets | maybe | cheap YAML error snippets | 0.12.16 | MIT OR Apache-2.0 |
| anstream, anstyle | yes | NO_COLOR logic, already in clap | 1.0.0, 1.0.14 | MIT OR Apache-2.0 |
| owo-colors, supports-color, console, yansi | no | duplicate anstream | 4.4.0, 3.0.2, 0.16.6, 1.0.1 | MIT, Apache-2.0, MIT, MIT OR Apache-2.0 |
| colored | no | MPL-2.0, global state | 3.1.1 | MPL-2.0 |
| indicatif | yes | hides itself without a TTY, 6 deps | 0.18.6 | MIT |
| serde, serde_json | yes | `--json` and JSON Lines | 1.0.229, 1.0.151 | MIT OR Apache-2.0 |
| comfy-table | maybe | minimal features only | 8.0.1 | MIT |
| tabled, tabwriter | no | proc-macro cost; ANSI-blind | 0.22.0, 1.4.1 | MIT, Unlicense/MIT |
| dialoguer, inquire | no | 14 and 32 deps for one prompt | 0.12.0, 0.9.4 | MIT |
| tracing, tracing-subscriber | maybe | later, feature-gated | 0.1.44, 0.3.23 | MIT |
| log, env_logger | no | no consumer | 0.4.34, 0.11.11 | MIT OR Apache-2.0 |
| human-panic | no | 32 deps | 2.0.8 | MIT OR Apache-2.0 |
| cargo-nextest | yes | process per test, default-filter | 0.9.146 | Apache-2.0 OR MIT |
| assert_cmd, predicates, assert_fs | yes | CLI and fixture tests, dev only | 2.2.2, 3.1.4, 1.1.4 | MIT OR Apache-2.0 |
| insta | yes | filters strip ANSI, redact paths | 1.49.0 | Apache-2.0 |
| snapbox, trycmd | no | one-way dir diff; 54 deps | 1.2.2, 1.2.1 | MIT OR Apache-2.0 |
| rstest, proptest, tempfile | yes | parametrised and property tests | 0.27.0, 1.11.0, 3.27.0 | MIT OR Apache-2.0 |
| cargo-llvm-cov, cargo-mutants | maybe | optional, not in hooks | 0.9.1, 27.1.0 | Apache-2.0 OR MIT, MIT |
| hyperfine | yes | end-to-end timing (not installed) | 1.21.0 | MIT OR Apache-2.0 |
| samply | yes | needs paranoid <= 1 (not installed) | 0.13.1 | MIT OR Apache-2.0 |
| divan, dhat | maybe | micro and heap profiling on demand | 0.1.21, 0.3.3 | MIT OR Apache-2.0 |
| criterion, flamegraph | no | heavy; needs perf | 0.8.2, 0.6.14 | Apache-2.0 OR MIT, MIT OR Apache-2.0 |
| clippy, rustfmt | yes | `[workspace.lints]` table verified on 1.98.1 | 1.98.1 | MIT OR Apache-2.0 |
| cargo-deny | yes | advisories, licences, bans, sources | 0.20.2 | MIT OR Apache-2.0 |
| cargo-audit, cargo-vet, cargo-geiger | no | redundant, heavy process, report only | 0.22.2, 0.10.2, 0.13.0 | Apache-2.0 OR MIT, Apache-2.0/MIT, Apache-2.0 OR MIT |
| cargo-shear | yes | AST-based unused deps | 1.14.0 | MIT |
| cargo-machete, cargo-udeps | no | imprecise; nightly | 0.9.2, 0.1.61 | MIT, MIT OR Apache-2.0 |
| cargo-bloat | maybe | not measured, old release | 0.12.1 | MIT |
| tokei | yes | the 300-line gate (not installed) | 15.0.0 | MIT OR Apache-2.0 |
| typos, taplo | yes | hk builtins | 1.51.1, 0.10.0 | MIT OR Apache-2.0, MIT |
| cargo-about | maybe | third-party notices for releases | 0.9.2 | MIT OR Apache-2.0 |
| mold, sccache | no | rust-lld default; short builds | 3.0.0, 0.18.0 | MIT, Apache-2.0 |
| cargo-binstall, bacon | no | GPL-3.0-only, AGPL-3.0 dev tools | 1.25.2, 3.26.0 | GPL-3.0-only, AGPL-3.0 |
