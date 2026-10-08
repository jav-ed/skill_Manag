# Handoff: open questions

*Decisions that belong to the user, each with the lead's default and what depends on it, followed by the decisions that are settled and should not be re-opened without a new reason. Ask the user in one short batch, not one question at a time.*

## Waiting for the user

| # | Question | Lead's default | Blocks |
|---|---|---|---|
| 1 | Name `skillmirror` (binary, crate, config dir `~/.config/skillmirror`)? | keep; a rename touches `Crates/Core/src/brand.rs` plus docs | docs pass, release |
| 2 | Add `Scratch` to `exclude_dirs` in the vault `config.yaml`? | yes; on the lead's machine `Scratch/Oracle/` holds Go fixtures with `.agents/skills` folders that a real sync would rewrite | any real sync on that machine |
| 3 | Quote the two invalid vault descriptions (`post-scheduler`, old vault `secrets`) after the user reviews the project copies? | yes, in the vault by the user's hand | `doctor` clean |
| 4 | Drop the Go behaviour "every config key can come from the environment"? | yes, only vault and root; any `SKILL_MANAG_*` variable is a hard error naming its replacement | none (implemented) |
| 5 | Provenance lock location, when built: in the project or in `$XDG_STATE_HOME`? | project (works over SSH and across machines) | lock feature |
| 6 | One-time `echo 3 \| sudo tee /proc/sys/vm/drop_caches` to measure the cold scan? | ask when it matters | registry cache decision |
| 7 | Remove the Astro leftovers in `Docs/Architecture/` and the empty `AI/` and `Matters/` folders? | yes, at the docs pass | docs pass |
| 8 | Keep a root-level `--dry-run` alias like the Go tool? | probably no; `sync --dry-run` is the documented form | CLI polish |
| 9 | Keep the 11 MB golden parity data out of the repo? | yes, regenerate from the Go tag | cutover |
| 10 | `.gitignore` has `fast*` and `dist*`; tighten or leave? | tighten (`/dist/`), the user's file | none |
| 11 | Tag `go-oracle` at Go commit `c7310f9` and push the tag? | yes, before cutover | cutover |
| 12 | Which `mise` tools to install: `tokei`, `cargo-deny`, `hyperfine`, `cargo-insta`? | all four | `just loc-gate`, `just deny`, timings |
| 13 | Review of the three edited project skills, then hand copy to the vault and add `secrets` to `mandatory`? | the user's pace | the skills track |

Permissions already given: commit and push of the branch `rust-rewrite-handoff` (2026-10-08, the user asked for a separate branch to keep things clean). Not given: push to `main`, tags, pull request, any real `sync`/`push` run, any write to the vault.

## Settled (do not re-open without a new reason)

- Rust, Cargo virtual workspace `Crates/{Core,Cli,Tui,Testkit}` (later `Web`, `Xtask`), edition 2024, Linux only, `compile_error!` elsewhere.
- Hard errors, no fallbacks; standard Rust naming; 300 code lines per file; strict lints.
- `.agents/skills` is the only discovered location; other agent directories are symlink bridges to it and never replace a real directory.
- Files are copied (never symlinked), git-tracked files only, permission bits preserved.
- Staged write plus `renameat2` exchange, verification of the old copy, per-target failure, exit code 4 for partial runs.
- Terminal stack: ratatui on termina, own input layer, no TUI framework.
- Web view in stages (static report, loopback server, guarded writes), after the TUI.
- Provenance lock: later, only if the user asks.
- Licence: Hippocratic License 3.0; dependency allow-list in `deny.toml`.
- The Go tree was removed from the branch on 2026-10-08 (user: "if you need to remove the Go code, I don't care"); the oracle is built from commit `c7310f9`, which must stay reachable.
