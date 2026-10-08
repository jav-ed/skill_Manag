# Handoff: next steps

*The ordered backlog, each item with the design to follow and a "done when". Items 1 and 2 protect user data and come first; the rest builds features on top. Phases refer to the table in the decision record. When you finish an item, tick it here and note what you learned in the pitfalls file.*

Status marks: `[ ]` open, `[~]` half built, `[x]` done.

## 1. [x] Finish the backup store and `undo` (phase 5)

Done on 2026-10-08. What exists now:

- `Crates/Core/src/backup/`: the store (`Backups`, `Run`, `Entry`), `undo`, and 36 Core tests in `tests.rs`, `apply_tests.rs`, `delete_tests.rs` and `undo_tests.rs` (store round trip, apply with a backup for update, create and unchanged, a failing store swaps the old copy back and fails only that target, delete with backup, undo of update, create and delete, filters, dry run, undo twice is a redo, project gone, symlinked `.agents`, lost tree, poisoned note, run id `../x`). Three mutations (no swap-back, unchecked run id, prune after every run) were each caught by a test.
- `ops::delete(targets, dry_run, backup, observer)` moves the folder to `.trash-*` first, then into the store, and renames it back when the store fails.
- CLI: `sync`, `push`, `add`, `init` and `delete` start a backup run before the write and print `Backup: run <id>`; `--json` documents carry `backup`. New commands `undo` and `history` (Cli tests in `tests/undo.rs`, two real-PTY prompt tests in `tests/undo_terminal.rs`). `undo` previews through a dry run of itself, asks unless `--yes`.
- TUI jobs pass a `Run`; the results page names the run (the snapshot test hides the id).
- Contract Q34 and the decision record row are written. Parity: the Backup line is output text, which the harness does not compare, so no divergence entry was needed.

Left from this item: a history and undo screen in the TUI (item 4); the `Run` indices are per command, so a command that applies twice in one run would reuse slots (no command does today).

## 2. [x] Fix review round 2 (Docs/Investigation/Review_Rounds/round_2_Full.md)

Done on 2026-10-08: M1 to M6, L1 to L12 and the decision I1, each as a failing test first (Core, Cli and Tui tests named in the status line of each finding). Decisions that went into the contract: Q35 (profile `exclude`), Q36 (TUI plan before confirm and job discipline), Q37 (selection and filter), Q38 (no fsync), Q39 (leftovers). Declined with a reason: comments inside a replaced `mandatory` list are dropped with the old items, a read-only `config.yaml` is still replaced with its mode kept, no cancel flag for a running job (Ctrl-C asks twice instead).

Left: a third review round with fresh eyes (no High, no Medium is the bar), and the Go parity re-run after the TUI and CLI changes (the last run, after the backup work, showed 0 unexpected).

## 3. [ ] Rest of phase 5

| Item | Design | Done when |
|---|---|---|
| Scan progress | `indicatif` line on stderr while scanning (`Event::ScanFinished` exists; add progress events), off for `--json` and non-terminals | a long scan shows a live line; tests prove nothing is printed without a terminal |
| `status` (DONE 2026-10-08) | Read-only table per project: up to date, would change (counts of files), mandatory missing, installed but not in the vault. `--json`, exit code like `--check` (contract Q41). Without a provenance lock it cannot say who changed a file, only that it differs | done: `ops/status.rs`, `Cli/tests/status.rs` |
| `diff [SKILL] [--project DIR]` (DONE 2026-10-08) | Unified diff (`similar`) of what a sync would write, coloured on a terminal, `--stat` (contract Q42). The TUI change preview does not use it yet: its confirmation page lists the removed files only | done: `ops/diff.rs`, `Cli/tests/diff.rs`; a TUI diff view is part of item 4 |
| `doctor` (DONE 2026-10-08) | Contract Q43. Layers: machine (git, pointer, old tool, backup store), config, vault (discovery, links, git), skill headers (the two real vault skills with an unquoted `: ` are caught), edits git does not know about, mandatory and profiles, scan with leftovers, root filesystem. Not done: a probe of `renameat2` on the real filesystem (it would write into the user's projects); the filesystem type is checked instead | done: `ops/doctor/`, `Cli/tests/doctor.rs` |
| `targets` bridge (DONE 2026-10-08) | Contract Q44. `targets: [claude]` in the vault config, `bridge` command, `add`/`init` link their own project after a write, `status` (drift) and `doctor` (warning) report it. Differs from the ADR: `sync` does not recreate a missing bridge, `bridge` does (reason in Q44). Not done: `.kiro/skills` and other agents (add a row to `KNOWN_TARGETS` in `config/vault_config.rs`) | done: `ops/bridge.rs` (9 tests), `Cli/tests/bridge.rs` and `bridge_links.rs` (18 tests incl. a real terminal; existing real directory, dangling link, wrong link, link above). Open: no test makes `add` apply with zero successful writes, so the `wrote > 0` guard in `commands/pipeline.rs` survives a mutation check |
| Registry cache | `$XDG_CACHE_HOME/skillmirror/` list of projects, `--rescan`, TTL, printed line "N projects from cache, age 3 h". Only after the cold scan is measured (needs `drop_caches`, ask the user) and only if it is still slow | measured gain, no silently missed project |
| Root `--dry-run` alias | Go had `skill_Manag --dry-run`; decide with the user whether to keep an alias for `sync --dry-run` | decision recorded in the contract |
| Provenance lock | Deferred by the user ("maybe later"). Would give three-way drift states and make `status` precise | do not start without the user |

## 4. [ ] TUI completion

Add/init screens (use `ops::plan_install` and `ops::resolve`), a history and undo screen, show scan issues and leftover warnings, wizard polish after real use, review-before-apply with the file list (M1 first). Keep the architecture: pure `App::handle(Event) -> Effects`, background jobs over the channel, tests with synthetic events and snapshots, one PTY test per new screen.

## 5. [ ] Phase 6: UX pass and web view

Live scan, detail pane, preview of what changes. Then the web view in stages (decision record, section 5): stage 0 `skillmirror report` writes one static self-contained HTML file with `maud` (matrix, group browser, skill detail, diffs, filter, dark mode); stage 1 `skillmirror web` is a loopback `axum` server behind the cargo feature `web` with the security minimum listed in the ADR; stage 2 mutations only behind `--allow-write`. `Crates/Web` does not exist yet. Done when the user accepts it.

## 6. [ ] Phase 7: documentation pass

README rewrite for the Rust tool (install via `just install`, commands, config, groups, profiles, undo), architecture docs for the Rust crates (the Architecture area still describes the Go code and carries Astro leftovers), `doc_Start.md` check, remove the empty `AI/` and `Matters/` folders, `grep` finds no old name outside the legacy detector (`SKILL_MANAG_*`, `skill_Manag`). Follow the doc-start rules (no em-dashes, `will inshallah`, linkers).

## 7. [ ] Phase 8: cutover (needs the user's explicit yes)

1. Parity table reviewed, 0 unexpected, new-feature tests green, CI green.
2. Tag the Go commit as `go-oracle` (`git tag go-oracle c7310f9`) before deleting it, so the oracle can be rebuilt. Decide with the user whether the golden data (11 MB, regenerable) stays out of the repo (recommended: out).
3. One commit that deletes `main.go`, `cmd/`, `internal/`, `go.mod`, `go.sum`, `styles/`; `git worktree remove --force Scratch/Oracle/src` on any machine that has the oracle worktree.
4. `cargo build --profile dist`, install check, README and docs final.
5. Merge `--no-ff` into `main`, push, PR. Each step is outward-facing: ask.

## 8. [ ] Separate track: skills (not part of the rewrite)

The lead edited three skills in this repo's own `.agents/skills/` (kept out of the pushed commits): `secrets` (new: owns Sops, Age, Fnox and a new `Age/keys_And_Recipients.md`), `default-tools` (no longer owns secrets, carries its commands itself), `skill-writer` (refreshed from the vault plus an "Independence" section). The user must review them, copy them to the vault by hand and add `secrets` to `mandatory` in the vault config; until then nobody may run a real sync or push on the lead's machine. Also open: `remote-helper` is bound to one project; sibling links in `coding` and `file-tree-optimization` (`../coding`, `../refac-cli`, `../../default-tools`); `post-scheduler` and the old vault `secrets` have invalid frontmatter (an unquoted `: `). These are the user's call; do not touch the vault.

## 9. [ ] Parked ideas and one pending report

- Pending report for the user: they asked for a short list of crates and tools that are good but blocked by the Hippocratic licence. The current list is in section 7 of the decision record (`git2`, `skiller`, the `cargo-binstall` libraries, `bacon`, `slint`); search the research reports for "licen" to complete it and send the user one consolidated list.
- Parked: skill dependency hints (`file-tree-optimization` links `../coding` and `../refac-cli`, which a skill should not do; the tool could warn), further agent directories beyond `.claude/skills`, a `--commit` option after sync, watch mode. All were judged "not now" in the decision record.
