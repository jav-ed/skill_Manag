# Handoff: open questions

*Decisions that belong to the user, each with the lead's default and what depends on it, followed by the decisions that are settled and should not be re-opened without a new reason. Ask the user in one short batch, not one question at a time.*

## Waiting for the user

| # | Question | Lead's default | Blocks |
|---|---|---|---|
| 1 | Name `skillmirror` (binary, crate, config dir `~/.config/skillmirror`)? | keep; a rename touches `Crates/Core/src/brand.rs` plus docs | docs pass, release |
| 2 | Add `Scratch` to `exclude_dirs` in the vault `config.yaml`? | yes; wherever `Scratch/Oracle/` exists it holds Go fixtures with `.agents/skills` folders that a real sync would rewrite (the lead's container had it; check the user's machine) | any real sync on a machine that has it |
| 3 | Quote the two invalid vault descriptions (`post-scheduler`, old vault `secrets`) after the user reviews the project copies? | yes, in the vault by the user's hand | `doctor` clean |
| 4 | Drop the Go behaviour "every config key can come from the environment"? | yes, only vault and root; any `SKILL_MANAG_*` variable is a hard error naming its replacement | none (implemented) |
| 5 | Provenance lock location, when built: in the project or in `$XDG_STATE_HOME`? | project (works over SSH and across machines) | lock feature |
| 6 | One-time `echo 3 \| sudo tee /proc/sys/vm/drop_caches` to measure the cold scan? | ask when it matters | registry cache decision |
| 7 | Remove the Astro leftovers in `Docs/Architecture/` and the empty `AI/` and `Matters/` folders? | done at the docs pass; the empty folders do not exist in a checkout, delete them on machines that have them | none |
| 8 | Keep a root-level `--dry-run` alias like the Go tool? | probably no; `sync --dry-run` is the documented form | CLI polish |
| 9 | Keep the 11 MB golden parity data out of the repo? | yes (it is out), regenerate from the Go commit | none |
| 10 | `.gitignore` has `fast*` and `dist*`; tighten or leave? | done: anchored to the root (`/dist*`, `/fast*`) | none |
| 11 | Tag `go-oracle` at Go commit `c7310f9` and push the tag? | yes; the merge is done without it, the tag keeps the oracle rebuildable if history is ever rewritten | `just parity` safety (A4) |
| 12 | Which `mise` tools to install: `tokei`, `cargo-deny`, `hyperfine`, `cargo-insta`? | all four | `just loc-gate`, `just deny`, timings |
| 13 | Review of the three edited project skills, then hand copy to the vault and add `secrets` to `mandatory`? | the user's pace | the skills track |
| 14 | Name for the interface project and its place: `Ui/` inside this repository (chosen) or a separate repository? | keep one repository with a self-contained `Ui/` and a documented JSON contract; splitting later is a `git subtree split` | none |
| 15 | Should the rest of CI (`check-features`, `ui-check`, the Chromium check) be added, and who adds them (needs the `workflows` permission)? | yes (A3) | CI |

Permissions given to the lead on 2026-10-08, none of them carried over to you automatically: commits, the push of the work branch and a pull request, the merge of pull request 1 into `main` after verification, no questions about steps the user plainly asked for, no external agents. Not given: tags, force-push, deleting branches, any real `sync`/`push` or other write against the real tree or vault.

## Settled (do not re-open without a new reason)

- Rust, Cargo virtual workspace `Crates/{Core,Cli,Tui,Web,Testkit}`, edition 2024, Linux only, `compile_error!` elsewhere.
- Hard errors, no fallbacks; standard Rust naming; 300 code lines per file; strict lints.
- `.agents/skills` is the only discovered location; other agent directories are symlink bridges to it and never replace a real directory.
- Files are copied (never symlinked), git-tracked files only, permission bits preserved.
- Staged write plus `renameat2` exchange, verification of the old copy, per-target failure, exit code 4 for partial runs.
- Terminal stack: ratatui on termina, own input layer, no TUI framework.
- Web view in stages (static report, loopback server, guarded writes), after the TUI. The interface is its own project, `Ui/` (Astro shell, Solid, lucide-solid, Motion), built files committed and embedded; the earlier "no SPA" ruling was overruled for `Ui/` at the user's request (decision record).
- Provenance lock: later, only if the user asks.
- Licence: Hippocratic License 3.0; dependency allow-list in `deny.toml`.
- AGENTS.md of a project (2026-10-10; the user answered some questions and let the lead decide the rest, say so if a default is wrong): the file is called `AGENTS.md` and there is no `CLAUDE.md` link; the typos of the user's five rules were fixed and the file-tree rule names both skills; the new project gets the file by default; the text lives in `<vault>/AGENTS.md` with the built-in rules as the default; the markers and the checksum are in the first file written; a hand-edited block is never replaced without `--force`; the built-in text needs its three skills in the project (hard error), a vault text is not checked; `agents add` for one project, no roll-out to every project at once; the command is called `agents`; the interface key to leave the file out of a new project is `m` (`a` is taken by "all"). Decision record section 8, contract Q57 to Q62.
- The Go tree was removed on 2026-10-08 (user: "if you need to remove the Go code, I don't care") and the Rust tool was merged into `main`; the oracle is built from commit `c7310f9`, which must stay reachable (tag it, A4).
- Build profile: `debug = false` and `incremental = false` for `dev` (user: the documentation must make the memory cost clear, and the debug options can be turned off).
