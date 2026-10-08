# Review rounds: Method

How the two review rounds were done, which techniques found real bugs, how to repeat them without disturbing the working tree, and what a third round should cover. Written on 2026-10-08 from the practice of rounds 1 and 2.

## Ground rules

- **Read-only on the live tree.** No edits, no `cargo` in the repository, no git commands. Findings go to one report file; the fixing session decides what to change.
- **Work in a throwaway copy** with its own `CARGO_TARGET_DIR` and `--offline`. The copy has its own `Cargo.lock`; the repository's lock file is never touched. `/Scratch/` is ignored by git (`.gitignore`), so copies there are not committed.
- **Reproducers print, they do not assert.** One run of a whole series then shows every observation side by side, and a finding is a printed line that differs from what the rule says. Tests that assert come later, in the fixing session, as permanent tests under `Crates/`.
- **One reproducer per finding**, named by series and number (`s1`, `a2`, `u8`), so a finding can cite it and a fix can re-run it.
- **Report layout** (see the round files): summary table, then High, Medium, Low, Info; each finding has where (`file:line`), scenario, evidence, fix, status; then "Verified OK" (negative results are kept, so nobody re-checks them), and a "Missing tests" table.
- **Markers**: `[RUN]` for reproduced, `[READ]` for from reading only. A `[READ]` finding is a lead, not proof.
- **Severity** is judged by consequence: silent data loss or a protection silently off with a realistic input is High; partial state, wrong result with a rarer precondition, or a hard-error-rule violation is Medium; misleading text, cleanup gaps, robustness are Low.

## Repeat a run in an isolated copy

From the repository root:

```bash
R=Scratch/Review
mkdir -p $R/work && cp -a Cargo.toml Cargo.lock LICENSE Crates $R/work/
# then copy the reproducers in (see Repro/readme_Repro.md for the file list and the one edit in Tui tests/mod.rs)
cd $R/work && export CARGO_TARGET_DIR=$PWD/../target
cargo test --offline -p skillmirror-core --test review_apply --test review_config -- --nocapture --test-threads=1
cargo test --offline -p skillmirror-tui review -- --nocapture --test-threads=1
cargo build --offline -p skillmirror && BIN=$PWD/../target/debug/skillmirror bash ../../../Project_Manag/Docs/Investigation/Review_Rounds/Repro/Cli/cli_repro2.sh
```

- A cold build of the copy compiles all dependencies; copy an earlier `target` directory to save minutes (the workspace crates rebuild, the dependencies do not).
- Record **the tree state** a report refers to: from the repository root run `find Crates -name '*.rs' | sort | xargs sha256sum | sha256sum | cut -c1-16`. Round 1 used `f0cc86667dbdba29`, round 2 `3611e9501e92ec54`, the check on 2026-10-08 `d01061c0fe7217c4`. The value depends on the path prefix, so always run it from the root over `Crates`.
- To see what moved between a snapshot and the tip: `diff -rq Crates Scratch/Review/work/Crates`.

## Techniques that found real bugs

| Technique | What it found | Where |
|---|---|---|
| Plan against the real vault and real root, read only | The scan root contains the Go oracle's fixture projects, so a real run would have rewritten them (round 1 M6). Also gives reference numbers. | round 1 |
| Path-shape matrix for any path option: `..` in the root, symlinked root, relative and absolute entries | `exclude_paths` silently off (round 1 H1) | `scan/` |
| Filesystem adversary scenarios in a temp tree: symlinked `.agents` and `.agents/skills`, a `0o555` subfolder, a destination that vanishes or appears after planning, non-UTF-8 names, an unreadable vault file, a read-only or symlinked `config.yaml` | Round 1 M2, M3, M5, L2, L3; round 2 M3, M4 | `ops/`, `apply/`, `config/` |
| Race loops: the same scenario 300 times with 3 threads, counting outcomes per target | Cleanup removing a directory a sibling needs: 36 to 68 percent of rounds (round 2 M3). One run of a race proves little; the count does. | `apply/` |
| Text-edit matrix: 20 hand-written shapes of a YAML file (blank line, comment at column 0, CRLF, BOM, quoted key, flow list over two lines, symlink), each printing OK or ERR and whether the file stayed untouched | Round 2 M4 | `config/save.rs` |
| Complexity ladder: grow an input one level at a time with a timer and a stop rule | Exponential profile diamonds (round 2 L3) | `config/vault_config.rs` |
| Interface draw fuzz: build every screen state (19), set the terminal to every size from 1x1 to 50x16, draw inside `catch_unwind`, count panics | Found none in 15,200 draws; cheap insurance for a ratatui app, rerun after any layout change | `Tui/` |
| Interface event-order tests: use the test harness, but take events off the job channel by hand (`rx.recv_timeout`, then `app.handle(event)`) so a user action can be placed between a job start and its report | Round 2 M2 (results on the wrong screen), L7 | `Tui/` |
| "The world changes while a page is open": change the disk after the page loaded, then press Enter | Round 2 M1 (the plan is built after Enter) | `Tui/` |
| Command-line script with an isolated `HOME`, `XDG_*` and every `GIT_*` variable unset, then deliberately set one (`GIT_DIR`) | Round 2 M6, L9 | `Cli/` |
| Grep for swallowed errors: `.ok()`, `unwrap_or`, `filter_map(Result::ok)`, `Err(_) =>`, `is_ok_and`, `to_string_lossy`, `to_str()`, `map_or(0` | Round 1 L7 (seven spots), round 2 L8, L11 | everywhere |
| Compare code with the contract and with existing tests: a test that encodes a design is evidence of intent, and may encode the bug | `sync_starts_at_once_but_delete_asks_first` (round 2 M1); contract Q18 against `the_filter_narrows_the_rows_and_selection_survives_it` (round 2 L5) | `Tui/`, [behavior_Contract.md](../../Descr/behavior_Contract.md) |

Notes on technique:

- For races, count per target, not per run; a single `Failed` among healthy siblings is the signal.
- For the `Tui`, never sleep in tests. The harness has `wait_select` and `wait_done`; the reproducers poll the channel with a timeout instead.
- Permission scenarios (`0o000`, `0o555`) must be restored at the end of the test, or the temp directory cannot be deleted.
- Quote strings such as `'====='` in zsh; unquoted they are parsed as commands.

## What a round 3 should cover

1. **The backup module, unreviewed so far**: `Crates/Core/src/backup/` (`store`, `undo`, `tree`, `clock`, `error`) and `Crates/Core/src/apply/place.rs`. Questions to answer with reproducers:
   - Crash between recording a change and doing it: does `undo` say something true?
   - `undo` twice, `undo` after the user edited the folder, `undo` of a created skill that was edited since.
   - Two runs at once into one backup store; run-id collisions; ordering of ids across midnight and clock changes.
   - Path handling in recorded entries: traversal (`..`), absolute paths, symlinks in the saved tree, non-UTF-8 names, permission bits and read-only files.
   - Disk full while saving; what happens to the old copy when the save fails.
   - Pruning or retention, if any; the JSON shape (`serde_json` was added) and its stability.
   - The snapshot check now lives in `place.rs`: re-run `a2`, `a3`, `a4` (series A) against it.
2. **Re-verify the open findings** once fixes land, using the Repro files and the "output when fixed" columns in [Repro/readme_Repro.md](Repro/readme_Repro.md), and convert each reproducer into a permanent test.
3. **New code since 2026-10-08**: diff the tree against the hash above and review only what moved, newest first.

## What to test next (ideas not yet tried)

- A destination that holds a FIFO, a socket or a device node; two `apply` runs on one project at once; a full disk during staging (small tmpfs); a read-only `.agents`.
- PTY runs with the harness in `Crates/Cli/tests/common/pty.rs`: terminal resize while a job runs, a paste into the filter, mouse clicks in a tiny terminal, Ctrl-C during a job (round 2 L6), `--json` with a terminal (L10), a closed terminal while the interface is open (L8).
- Filter input with non-ASCII text and combining characters (contract Q27 lists the Go problem).
- A vault that is a worktree, has submodules, or sits inside another repository; `GIT_*` set while the Tui scans.
- A vault with thousands of skills: timing of scan, plan and the draw of a long list.
- `config.yaml` with unknown keys and a `profiles:` block through the wizard (round trip must keep them).
- Windows-style line endings and a BOM in every file the tool reads or writes (YAML is accepted; check the other files).

## Why this is written down

The reviewer sessions are gone after a hand-off; the reports in `Scratch/` are ignored by git. The two round files and the reproducers are the durable record: [round_1_Core.md](round_1_Core.md) with the tests that closed each finding, [round_2_Full.md](round_2_Full.md) with the findings still open, and [Repro/readme_Repro.md](Repro/readme_Repro.md) with the files to run.
