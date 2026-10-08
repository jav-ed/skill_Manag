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
| Scan progress (DONE 2026-10-08) | Contract Q45. `Event::ScanProgress` every 256 folders from the scan's threads, an `indicatif` spinner on stderr in `sync`, `push`, `status`, `diff`, `bridge`, `list`, `delete`; hidden for `--json`, pipes and `TERM=dumb` | done: `scan/progress_tests.rs`, `ops/workspace_tests.rs`, `Cli/tests/progress.rs` (real terminal: drawn and cleared; JSON, dumb terminal and pipe: nothing). Not shown: `doctor` (it reports its own layers), the TUI (it has its own loading page) |
| `status` (DONE 2026-10-08) | Read-only table per project: up to date, would change (counts of files), mandatory missing, installed but not in the vault. `--json`, exit code like `--check` (contract Q41). Without a provenance lock it cannot say who changed a file, only that it differs | done: `ops/status.rs`, `Cli/tests/status.rs` |
| `diff [SKILL] [--project DIR]` (DONE 2026-10-08) | Unified diff (`similar`) of what a sync would write, coloured on a terminal, `--stat` (contract Q42). The TUI question uses the same view for its plan (`ops::diff_of_plan`, Q49) | done: `ops/diff.rs`, `Cli/tests/diff.rs` |
| `doctor` (DONE 2026-10-08) | Contract Q43. Layers: machine (git, pointer, old tool, backup store), config, vault (discovery, links, git), skill headers (the two real vault skills with an unquoted `: ` are caught), edits git does not know about, mandatory and profiles, scan with leftovers, root filesystem. Not done: a probe of `renameat2` on the real filesystem (it would write into the user's projects); the filesystem type is checked instead | done: `ops/doctor/`, `Cli/tests/doctor.rs` |
| `targets` bridge (DONE 2026-10-08) | Contract Q44. `targets: [claude]` in the vault config, `bridge` command, `add`/`init` link their own project after a write, `status` (drift) and `doctor` (warning) report it. Differs from the ADR: `sync` does not recreate a missing bridge, `bridge` does (reason in Q44). Not done: `.kiro/skills` and other agents (add a row to `KNOWN_TARGETS` in `config/vault_config.rs`) | done: `ops/bridge.rs` (9 tests), `Cli/tests/bridge.rs` and `bridge_links.rs` (18 tests incl. a real terminal; existing real directory, dangling link, wrong link, link above). Open: no test makes `add` apply with zero successful writes, so the `wrote > 0` guard in `commands/pipeline.rs` survives a mutation check |
| Registry cache | `$XDG_CACHE_HOME/skillmirror/` list of projects, `--rescan`, TTL, printed line "N projects from cache, age 3 h". Only after the cold scan is measured (needs `drop_caches`, ask the user) and only if it is still slow | measured gain, no silently missed project |
| Root `--dry-run` alias | Go had `skill_Manag --dry-run`; decide with the user whether to keep an alias for `sync --dry-run` | decision recorded in the contract |
| Provenance lock | Deferred by the user ("maybe later"). Would give three-way drift states and make `status` precise | do not start without the user |

## 4. [x] TUI completion

Done on 2026-10-08: scan problems (`i`, Q46), History page with undo (Q47), Add and Init pages with the links (Q48), each with flow tests, snapshots, mutation checks and a real-terminal test in `Crates/Cli/tests/interface_screens.rs`. Also done: the changes page behind `v` on the question (Q49). Left: wizard polish after real use. Keep the architecture: pure `App::handle(Event)`, background jobs over the channel with job ids, tests with synthetic events and snapshots, one real-terminal test per new screen.

## 5. [ ] Phase 6: UX pass and web view

Done on 2026-10-08: stage 0, `skillmirror report` (Q50): `Crates/Web` with `maud`, data from `ops::report_data`, checked in Chromium (filter, theme toggle, anchors, no console errors, screenshots read). Left: stage 1 `skillmirror web` (loopback `axum` 0.8 behind the cargo feature `web`, with the security minimum of the decision record, section 5) and stage 2 mutations behind `--allow-write`. They are bigger than the rest, bring an async runtime and an attack surface, and the report already covers read-only viewing, so they are deliberately not started; decide with the user whether they are wanted. The TUI live scan, detail pane and preview of the UX pass exist (scan line, `i`, changes page).

## 6. [x] Phase 7: documentation pass

Done on 2026-10-08: README rewritten for the Rust tool (install, commands, interface, configuration, safety, development); architecture docs for the crates (`Docs/Architecture/rust_Overview.md`, `core_Modules.md`, `front_Ends.md`, with the Go structure kept as `go_Legacy.md`); `sync_Concept.md` rewritten; the Descr linker and `doc_Start.md` updated; the five Astro folders under `Docs/Architecture/` removed (they described a website that is not this repository; nothing linked to them); `deny.toml` header no longer says "unvalidated"; `.gitignore` anchors `dist*` and `fast*` to the root and drops the Astro lines. The empty `AI/` and `Matters/` folders do not exist in a checkout (git does not track empty folders); delete them on the machines that have them. `grep` finds the old names only in the legacy detector, its tests, the `migrate` command, the parity harness and the Go tree itself. Left for the cutover: the README install section changes from the branch to `main`, the handoff folder is promoted into the docs and deleted.

## 7. [ ] Phase 8: cutover (needs the user's explicit yes)

Prepared, not done: the exact steps, the checks before and after, and the way back are in [cutover_Runbook.md](cutover_Runbook.md). Nothing in the Rust build depends on the Go tree (checked by grep), so deleting it breaks no Rust check. Each step that leaves the machine (a tag, a push to `main`, a branch deletion) needs the user.

## 8. [ ] Separate track: skills (not part of the rewrite)

The lead edited three skills in this repo's own `.agents/skills/` (kept out of the pushed commits): `secrets` (new: owns Sops, Age, Fnox and a new `Age/keys_And_Recipients.md`), `default-tools` (no longer owns secrets, carries its commands itself), `skill-writer` (refreshed from the vault plus an "Independence" section). The user must review them, copy them to the vault by hand and add `secrets` to `mandatory` in the vault config; until then nobody may run a real sync or push on the lead's machine. Also open: `remote-helper` is bound to one project; sibling links in `coding` and `file-tree-optimization` (`../coding`, `../refac-cli`, `../../default-tools`); `post-scheduler` and the old vault `secrets` have invalid frontmatter (an unquoted `: `). These are the user's call; do not touch the vault.

## 9. [ ] Parked ideas and one pending report

- Pending report for the user: they asked for a short list of crates and tools that are good but blocked by the Hippocratic licence. The current list is in section 7 of the decision record (`git2`, `skiller`, the `cargo-binstall` libraries, `bacon`, `slint`); search the research reports for "licen" to complete it and send the user one consolidated list.
- Parked: skill dependency hints (`file-tree-optimization` links `../coding` and `../refac-cli`, which a skill should not do; the tool could warn), further agent directories beyond `.claude/skills`, a `--commit` option after sync, watch mode. All were judged "not now" in the decision record.
