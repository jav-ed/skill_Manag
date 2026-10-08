# Handoff: next steps

*The ordered backlog, each item with the design to follow and a "done when". Items 1 and 2 protect user data and come first; the rest builds features on top. Phases refer to the table in the decision record. When you finish an item, tick it here and note what you learned in the pitfalls file.*

Status marks: `[ ]` open, `[~]` half built, `[x]` done.

## 1. [~] Finish the backup store and `undo` (phase 5)

Why: `sync` overwrites local edits in projects and `delete` removes folders for good. A saved copy and a one-command way back remove the fear of running either.

### What exists (compiles, untested, not wired)

- `Crates/Core/src/backup/`: `error.rs` (`BackupError`, `UndoError`), `clock.rs` (run ids `YYYYMMDD-HHMMSS-mmm-pid-n`, `describe`; has tests), `tree.rs` (`move_tree`: rename, or copy then remove on `EXDEV`; `copy_tree`; has tests), `store.rs` (`Backups`, `Run`, `RunKind`, `Change`, `Entry`, `LoadedRun`, `KEEP_RUNS = 30`, `Run::finish` prunes only when something was stored), `undo.rs` (`undo(backups, id, &Filter, dry_run, observer) -> UndoReport`).
- `backup/tests.rs` and `backup/undo_tests.rs` are EMPTY placeholders declared in `backup/mod.rs`.
- `Crates/Core/src/apply/place.rs`: the "put a finished folder at its destination" step moved out of `run.rs`, now able to move the replaced copy into the store (`Keep`, `Replaces`, `place`, `discard`). `ApplyOptions<'a>` has `backup: Option<&'a Run>`. `ApplyError::Backup` and `Error::Backup` exist.
- On disk: `<state>/backups/<run-id>/<index>/entry.json` (kind, project, skill, change) plus `tree/` (absent for `Created`). `<state>` is `Dirs::state()`, normally `~/.local/state/skillmirror`.

### Design decisions already made (keep unless you find a flaw)

- Backups are always on, no flag. If the old copy cannot be stored, the update is swapped back and that target fails (hard error, no half state).
- A `Created` entry is recorded before the skill is created and forgotten if creation fails, so `undo` can remove what a `push` installed.
- Undo is itself a run (`RunKind::Undo`): what it replaces or removes is saved again, so running `undo` twice is a redo. Say so in the help text and in the output.
- Undo copies the saved tree out of the store and deletes the entry only after the new copy is in place; it never moves the only copy.
- Disk data is validated before use: `validate_name` on the skill, the project must be a directory, `first_link_above` must be clean. Run ids from the user must match `[A-Za-z0-9-]` and name an existing folder (no `../`).
- `Run::finish(backups)` returns `Finished { id, stored, prune_error }`. A prune failure is reported as a warning; the run itself is saved.

### Steps

1. `cargo test -p skillmirror-core` and clippy; the file `store.rs` is near the 300-line gate (304 non-blank lines including comments); split if tokei says so.
2. Core tests (in `backup/tests.rs`, `backup/undo_tests.rs`, and `apply/` tests): store round trip; apply with a backup for update, create, unchanged (stores nothing), and without a backup; a failing `keep` swaps the old copy back and fails only that target; delete with backup; undo of an update, a delete and a create; filters by project and skill; dry run changes nothing; undo twice is a redo; project gone; symlinked `.agents`; missing tree; run id `../x` refused; `prune` keeps the newest `keep`; `finish` prunes only when something was stored; the undo run does not collide with the stage names of the original run.
3. Wire `ops/delete.rs`: `delete(targets, dry_run, backup: Option<&Run>, observer)`: rename to `.trash-*`, `run.keep(index, target, Change::Deleted, &trash)`, rename back if that fails; a symlinked skill folder is not backed up (only the link goes). Update callers in `Cli/src/commands/delete.rs`, `Tui/src/jobs.rs` and tests.
4. CLI: in `commands/pipeline.rs`, `delete.rs`, `install.rs` begin a `Run` (`Backups::in_dirs(&Dirs::from_env()?)`, kind per command) before the write, pass `ApplyOptions { backup: Some(&run), ..Default::default() }`, print `Backup: run <id> (undo with skillmirror undo)` and prune warnings from `finish`. New commands: `undo [RUN] [--project DIR] [--skill NAME] [--dry-run] [-y] [--json]` and `history [--json]` (list runs: id as date, command, entry count). Exit codes as elsewhere (4 on partial failure, 2 on usage). The JSON documents get the run id.
5. TUI: `jobs.rs` passes a `Run`; the done screen shows the run id; a history/undo screen comes later (item 5).
6. Contract: add Q34 (backups on by default, 30 runs kept, undo is a run, hard error when the backup cannot be stored); update ADR rows 35/36 wording.

Done when: all of the above tests pass, a real PTY or shell run shows `sync -y` followed by `undo -y` restoring the previous bytes and modes, the parity run still shows 0 unexpected divergences (the extra backup line is an expected divergence, add it to the table), and the line gate holds.

## 2. [ ] Fix review round 2 (Docs/Investigation/Review_Rounds/round_2_Full.md)

Read the file: each finding has where, scenario, evidence and a proposed fix, and reproducers sit in `Review_Rounds/Repro/`. Make each reproducer a normal regression test, red first, then fix.

- **M1 (Tui)**: the job plans again after Enter (`jobs.rs`), so `sync`/`push` can delete files the user never saw (a `my_notes.md` added to a project copy while the sync page was open vanished without a word). Build the plan before the confirm page, show added, changed and removed files, and apply that same plan.
- **M2 (Tui)**: job events land on whatever screen is open; the mouse header arrow leaves a running job; a second job can start. Add a job id to every event, ignore stale ones, and one busy guard shared by keys and mouse.
- **M3 (Core apply)**: when one new-project target fails, `remove_created` deletes the `skills/` folder its siblings still need. Create `skills/` once per project before the pool starts and clean up after the pool.
- **M4 (`save_config`)**: replaces a symlinked `config.yaml` by a regular file; drops comments inside the list; turns CRLF into LF; refuses a blank line or a column-0 comment inside the list, a quoted key, a BOM. Edit through the link target, keep line endings, accept the YAML forms it refuses.
- **M5 (wizard)**: the pointer is written before the config is saved, so a failed save leaves the pointer moved. Save the config first.
- **M6 (`init`)**: when every skill fails to plan, `init` leaves an empty project directory (and `.git` with `--git`), and the retry fails with "not empty". With `changes() == 0` behave like a dry run and never call `before_write`.
- Lows worth doing: L1 a same-size edit with a restored mtime evades the snapshot (add `ctime`); L2 an `Unchanged` plan fails when a file was just re-saved; L3 profile `extends` diamonds are exponential (22 levels took 8.5 s on every config load; memoise); L4 `exclude` depends on the `extends` order; L6 Ctrl-C quits mid-job; L9 `init --git` obeys `GIT_DIR`; L10 `--json` on a terminal prints the rows before the JSON. The rest are in the file.
- Not decided yet: I1 (the unmerged-index scope) and whether to document the no-fsync trade-off in the contract.

Done when: every finding is fixed or explicitly declined with a reason in `round_2_Full.md`, each with a test; a third review round (helper 1 offered to do it) finds no High and no Medium.

## 3. [ ] Rest of phase 5

| Item | Design | Done when |
|---|---|---|
| Scan progress | `indicatif` line on stderr while scanning (`Event::ScanFinished` exists; add progress events), off for `--json` and non-terminals | a long scan shows a live line; tests prove nothing is printed without a terminal |
| `status` | Read-only table per project: up to date, would change (counts of files), mandatory missing, installed but not in the vault. `--json`, exit code like `--check`. Without a provenance lock it cannot say who changed a file, only that it differs | works over the real tree read-only; JSON stable and documented |
| `diff [SKILL] [--project DIR]` | Unified diff (`similar`) of what a sync would write, coloured on a terminal, `--stat`; the TUI change preview reuses it | diff of a changed, added and removed file are covered by tests |
| `doctor` | Vault: `SKILL.md` frontmatter lint (name equals folder, description present, valid YAML; two real vault skills have an unquoted `: `), untracked or unstaged edits in skill folders (the copy uses working-tree files of tracked paths), ignored folders from discovery, duplicate or clashing names. Environment: config validity, pointer state, legacy `~/.config/skill_Manag`, git present, `renameat2` support, leftover `.stage-*`/`.trash-*` folders (`panic = "abort"` skips destructors), backup store size | each check has a failing-case test; exit codes documented |
| `targets` bridge | Config key (vault `config.yaml`) naming extra agent dirs such as `claude`: create `<project>/.claude/skills` as a relative symlink to `../.agents/skills`; never replace a real directory (the kernel refuses with `EISDIR`); `sync` recreates a missing bridge | tests incl. an existing real directory, a dangling link, a wrong link |
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
