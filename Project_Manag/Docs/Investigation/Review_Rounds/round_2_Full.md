# Review rounds: Round 2, Full

Adversarial read of the code written after round 1: `ops/project.rs`, `ops/select.rs`, profiles and `save_config`, `apply/run.rs`, `plan/build.rs`, `plan/inspect.rs`, `Crates/Cli/src` and `Crates/Tui/src`. Second session, read-only, 2026-10-07 and 2026-10-08. All findings were **open** at the tip checked on 2026-10-08; the status line of each says what was done since (all fixed or decided on 2026-10-08, with the parts declined named).

## Not reviewed: the backup module

The working tree also holds a half-built, untested backup module: `Crates/Core/src/backup/` (`store.rs`, `undo.rs`, `tree.rs`, `clock.rs`, `error.rs`, and two test files that were still one line long when checked) and `Crates/Core/src/apply/place.rs`. It moves the swap and the snapshot check out of `apply/run.rs`. **Nothing in it was reviewed**, and the reproducers build `ApplyOptions { threads, ..ApplyOptions::default() }` so that they compile against the tip. Findings M3, L1 and L2 concern code that `place.rs` now shares (the snapshot comparison is `apply/place.rs:75`); re-check them after the module settles.

## Scope and state checked

- **Reviewed snapshot**: all of `Crates/` as copied on 2026-10-07, hash `3611e9501e92ec54`. Line numbers below are the tip's; files outside `apply/` are byte-identical between snapshot and tip.
- **Checked against the tip**: 2026-10-08, hash `d01061c0fe7217c4` over 135 `.rs` files (method in [review_Method.md](review_Method.md)). Differences from the snapshot: `apply/{run,error,mod,tests}.rs`, new `apply/place.rs`, new `backup/`, `error.rs`, `lib.rs`, `Core/Cargo.toml` (adds `serde_json`). Tui and Cli code is unchanged. Re-run on the tip: S1, S2, P1, P2, A1, A2, A3, U1 to U8, K1 and K2 still show the bug (outputs in [Repro/readme_Repro.md](Repro/readme_Repro.md)). Besides the backup option, `apply/run.rs` differs only by the fsync note on `apply` (round 1 L6).
- **Existing suite** in the snapshot: 180 tests pass. Real-data dry runs (real vault, real root, read only): sync 101 update and 335 same, push 127 create, 86 update, 256 same, 0 failed.
- **Markers**: `[RUN]` = reproduced. `[READ]` = from reading. **Severity** as in [round_1_Core.md](round_1_Core.md).

## Summary

| Severity | Count | IDs |
|---|---|---|
| High | 0 | none |
| Medium | 6 | M1 to M6 |
| Low | 12 | L1 to L12 |
| Info | 1 | I1 |

Fix first: M1 (the Tui plans after Enter and writes with no preview), M2 (job events on the wrong screen), M3 (cleanup race in apply), M4 (`save_config` edge cases). No panic in 15,200 draws of 19 screens at sizes 1 to 50 columns by 1 to 16 rows.

## Medium

### M1. The Tui plans after the user pressed Enter, and sync and push write on that Enter `[RUN]`

- **Where**: `Tui/src/jobs.rs:42` (`Plan::for_targets` inside the job thread), `Tui/src/screens/work_keys.rs:60-68` (`confirm_selection`: Sync and Push call `run_now`, only Delete calls `ask`), `Tui/src/screens/work.rs:124-138` (`pending` carries targets, no plan).
- **Scenario** (`u8`): the Sync page lists `astro  1 project  1 to update`. A file `my_notes.md` is added to a project copy while the page is open. One Enter: the job builds a new plan from the disk as it is now, `my_notes.md` lands in `removed`, the snapshot check passes (taken a millisecond earlier), the file is deleted, and the results page does not mention it. `u2b`: on the Push page one Enter goes to `Push/running 0/6` with only a per-skill count shown.
- **Why it matters**: round 1 M1 made the plan snapshot-checked so the human confirmation window is safe. The Tui has no confirmation window and plans after it, so the guard is unreachable. The Cli shows file rows and asks first (`Cli/src/commands/pipeline.rs:77-80`).
- **Existing test that encodes the behavior**: `Tui/src/screens/tests.rs`: `sync_starts_at_once_but_delete_asks_first`. It must change with the fix. Go also started on Enter (contract Q21 is about its missing phase guard), so this is a carry-over, now unsafe.
- **Fix**: build the `Plan` when the page loads or on Enter, keep it in `Pending`, show a confirm page listing removed and modified files (the Delete dialog has the shape), and pass that plan to `apply` so `DestinationChanged` can fire. Re-plan on entering the confirm page, never after it.
- **Status**: fixed on 2026-10-08. Sync, push and the list's `s` work out a plan in a job (`Phase::Planning`), show it on a confirmation page that names the files that would be removed, and apply that same plan, so `DestinationChanged` fires for a file added meanwhile. A plan that would write nothing goes straight through. Tests: `Tui/src/tests/round2_jobs.rs` (including a file added while the page is open) and the snapshot `the_sync_confirmation_lists_the_files_it_would_remove`; the old test is now `sync_and_push_plan_first_and_delete_asks_first`. Contract Q36.

### M2. Job events land on whichever screen is open, and the mouse can leave a running job `[RUN]`

- **Where**: `Tui/src/app/jobs_events.rs:22-35` (`Finished` and `Failed` write `work.phase` of the current screen and set `session = None`), `Tui/src/screens/work_keys.rs:24` (keys are swallowed in `Running`) against `work_keys.rs:135-190` (the mouse path has no phase guard for the header arrow), `Tui/src/app/mod.rs:117-125` (`open` reuses the old `session`).
- **Scenario** (`u1`): Enter on Sync gives `Sync/running 0/3`. Click the header arrow: `menu`. Open Delete: `Delete/select`, populated from the pre-job session. The Sync job reports and the Delete page becomes `Delete/done(results of Sync)`. From that page the user can confirm a delete while the first job still writes: two jobs on the same projects. If the user is on the menu when the job ends, `work_mut()` is `None` and the results are lost.
- **Existing test**: `Tui/src/screens/tests.rs`: `running_ignores_keys_so_a_second_enter_cannot_restart_it` covers keys only.
- **Fix**: a job id and kind; `running: Option<JobId>` in `App`; while set, ignore the header arrow and `open` (or show a "job in progress" page); drop events whose id differs. One guard (`Work::busy()`) for keys and mouse.
- **Status**: fixed on 2026-10-08. Every job report carries a job id and only the job the screen waits for is heard; the header arrow and Ctrl-C are guarded while a job writes; nothing starts while another job writes. Tests: `round2_jobs.rs` (header arrow, stale reports, leaving during the plan).

### M3. A failing target removes the `skills/` directory its siblings still need `[RUN]`

- **Where**: `Core/src/apply/run.rs:124-159` (`write_one`), `:161-177` (`create_skills_dir`: each target records the directories that did not exist when it looked), `:180-` (`remove_created`). Introduced by the round 1 L8 fix.
- **Scenario** (`a1`): three new skills into a fresh project, 3 threads, `b/big.md` unreadable. `b` fails as it should. Over 300 rounds the healthy `a` and `c` failed with `Swap { ..., source: Os { code: 2, kind: NotFound } }` 203 times each on the snapshot and 109 times each on the tip (race, load dependent). `remove_dir` succeeds while `skills/` is still empty, because siblings are only staged in `.agents/.stage-*`.
- **Impact**: no data loss and a retry works, but one bad file becomes three failures, two with a bare ENOENT. This is the `init` and `add` path (`creates_skills_dir`).
- **Fix**: do not clean up per target. Create `skills/` once per project in a serial step before the pool starts. After the pool, remove `skills/` and `.agents` only for projects the run created and whose targets all failed. Test it 50 times because it is a race.
- **Status**: fixed on 2026-10-08. `apply` creates `skills/` once per new project before the pool starts and gives the directories back only when every target of that project failed (`Core/src/apply/run.rs`). Test: `apply/new_project_tests.rs`, a 50-round race loop that was red before the fix (it uses a deleted source file instead of file permissions, so it also bites as root).

### M4. `save_config` breaks symlinks, drops comments, rewrites line endings, and refuses ordinary lists `[RUN]`

- **Where**: `Core/src/config/save.rs:117-147` (`set_key`: the block is lines that start with space, tab or `-`; a blank or column-0 comment line ends it), `:149-156` (`write_atomically` persists a temp file over the path).
- **Scenarios** (`s1`, `s2`):
  - `config.yaml` is a symlink into a dotfiles repo: after the save it is a regular file; the dotfiles copy still says `[a]`, the vault copy says `- b`.
  - Comments inside the replaced list are deleted (an indented one inside the block as well). CRLF input becomes LF.
  - Valid YAML that `VaultConfig::load` accepts makes the save fail with the file untouched: a blank line inside the list and a column-0 comment inside the list ("the rewritten file does not hold the new mandatory list"), a quoted key `"mandatory":` and a BOM before the key ("duplicate mapping key: mandatory").
  - A read-only (0444) config is replaced without a word (permissions are kept, and there is a test for that).
- **Fix**: resolve the path with `canonicalize` and write the temp file next to the real target, or refuse a symlinked config naming the real path. Keep blank lines and comments inside the block until the next column-0 key. Detect `\r\n` and keep it. Match the key through the parser's span, or compare keys with quotes stripped. Name the blocking construct in the error.
- **Status**: fixed in part on 2026-10-08 (`config/save_edge_tests.rs`, red first). A symlinked config is edited where it lives and stays a link, a dangling link is an error, a blank line or a column-0 comment inside the list no longer breaks the save or leaves old items, quoted keys and a BOM are found, CRLF is kept. **Declined**: comments inside the replaced list go with the old items (the test `comments_other_keys_and_their_order_survive_a_rewrite` documents it), and a read-only config is still replaced with its mode kept: the save is the user's explicit choice in the wizard.

### M5. The setup wizard writes the pointer before it knows the config can be saved `[RUN]`

- **Where**: `Tui/src/app/mod.rs:174-181` (`write_pointer`, then `save_config`, then reload).
- **Scenario** (`u5`): the vault config has `mandatory: [coding, tmux]` with a column-0 comment between the items. The wizard shows `saved with error: ... mandatory lists "tmux" twice` and the pointer file already holds the new vault path. The next start uses the new vault with the old root and mandatory list.
- **Rule**: no partial state on a hard error. The pointer is the lesser write, so it goes last.
- **Fix**: save the config first, write the pointer second, and restore the previous config text if the pointer write fails (the wizard has it in memory). Test: an unsaveable config leaves the pointer file absent.
- **Status**: fixed on 2026-10-08. The wizard saves the config first and the pointer second, and puts the config back as it was if the pointer cannot be written. Tests: `round2_wizard.rs` (both orders, red without the fix).

### M6. `init` creates the project, and with `--git` the repository, even when nothing can be written `[RUN]`

- **Where**: `Cli/src/commands/pipeline.rs:73-81` (the early return covers only `changes() == 0 && failed == 0`), `Cli/src/output/view.rs:183` (`changes()` is `new + update`), `Cli/src/commands/install.rs:36-63` (`before_write` runs `create_new`).
- **Scenario** (script K1): the only mandatory skill has an untracked `SKILL.md`. `init <dir> --yes` ends with the failed row and exit 4, and `<dir>` exists, empty. `init <dir2> --git --yes` leaves `<dir2>/.git`. The second try ends with `error: ... is not empty`, so the user must delete the directory before retrying. Without `--yes` the prompt reads `Write 0 skill folder(s) in 1 project(s)?` and a yes creates the directory anyway (`[READ]`).
- **Fix**: when `changes() == 0` (failures only), behave like a dry run: show rows, return `Exit::Partial`, never prompt, never call `before_write`. In `init`, remove a directory and repository the command created if the apply fails.
- **Status**: fixed on 2026-10-08. A plan with failures only is shown like a dry run (no prompt, no `before_write`, exit 4), and `init` takes back the directory and repository it made when nothing could be installed, but never a directory that was there or anything not empty (`NewProject::take_back`). Tests: `Cli/tests/init_failures.rs`.

## Low

### L1. The snapshot check misses a same-size edit that keeps the mtime `[RUN]`

- **Where**: `Core/src/plan/inspect.rs:11-17` (`Entry::File { len, mode, modified }`), checked in `apply/place.rs:75` and `apply/run.rs:203-214`.
- **Scenario** (`a2`): `OLD1` becomes `EDIT` with the mtime set back: `Updated`, the edit is gone. An ordinary edit is caught. `rsync -t`, `cp -p`, `touch -r` defeat the guard.
- **Fix**: add `ctime` (`MetadataExt::ctime`, `ctime_nsec`) to `Entry::File`; userspace cannot set it.
- **Status**: fixed on 2026-10-08: the snapshot holds `ctime` too. Test: `apply/guard_tests.rs`.

### L2. An `Unchanged` plan fails when a file is merely re-saved `[RUN]`

- **Where**: `Core/src/apply/run.rs:203-214` (`verify_unchanged`).
- **Scenario** (`a3`): plan says `Unchanged`, the user saves a file with identical bytes, apply reports `Failed(DestinationChanged)` and `sync` exits 4 for a skill that is in sync.
- **Fix**: on a snapshot mismatch of an Unchanged plan, compare bytes again (`same_bytes` exists) and report `Unchanged` when equal.
- **Status**: fixed on 2026-10-08: on a snapshot mismatch an `Unchanged` plan is checked again byte by byte against the vault, and only a real difference fails it. Test: `apply/guard_tests.rs`.

### L3. Profile `extends` diamonds take exponential time `[RUN]`

- **Where**: `Core/src/config/vault_config.rs:114-130` (`check_acyclic` re-walks every parent, no memo), `Core/src/ops/select.rs:125-156` (`collect_profile`).
- **Scenario** (`p1`): each level extends the two profiles below it. Load time: 16 levels 0.12 s, 18 levels 0.6 s, 20 levels 2 s, 22 levels 8 s. The loop check runs on every `VaultConfig::load`, so every command pays.
- **Fix**: depth-first search with a done set (white, grey, black); the same set in `collect_profile`.
- **Status**: fixed on 2026-10-08: the loop check and the profile resolution work out each profile once. Test: `ops/profile_tests.rs` (22 levels in milliseconds, 15.7 s before).

### L4. A profile's `exclude` depends on the order of `extends` `[RUN]`

- **Where**: `Core/src/ops/select.rs:149-153` (`out.remove(skill)` removes only what was collected so far; `out` is shared by every parent).
- **Scenario** (`p2`): `a: {skills: [x, y]}`, `c: {exclude: [y]}`. `b1: {extends: [a, c]}` gives `{x}`, `b2: {extends: [c, a]}` gives `{x, y}`. The same holds for two `--profile` flags.
- **Fix**: decide the meaning and write it in the contract. Cleanest: collect parents, groups and skills of the whole closure first, then apply all excludes.
- **Status**: fixed on 2026-10-08 with a decision, contract Q35: an `exclude` removes skills from what its own profile selects, so the order of `extends` never matters. Tests: `ops/profile_tests.rs`.

### L5. Enter after a filter acts on selected rows that are hidden `[RUN]`

- **Where**: `Tui/src/screens/work.rs:124-138` (`pending` reads `self.selected`, not the filtered view).
- **Scenario** (`u2`): on Delete press `a`, then `/astro`, Enter, Enter. The header says `3 / 3 selected`, one row is visible, the dialog says `Delete 3 skills in 2 projects?`. The dialog is honest about the count, so delete is guarded; for Sync and Push the same state starts a job at once (M1).
- **Contract and tests**: contract Q18 lists "selection survives filter changes" as a Go behavior to CHANGE, yet `Tui/src/screens/tests.rs`: `the_filter_narrows_the_rows_and_selection_survives_it` asserts the survival, and `all_toggles_only_the_visible_rows_and_flips_back` makes `a` act on visible rows only. Decide one policy.
- **Fix**: either Enter acts on visible rows only, or the header shows `1 shown, 3 selected` and the confirm page lists names.
- **Status**: fixed on 2026-10-08 with a decision, contract Q37: Enter, `s` and `d` act on the selected rows that are visible, a selection made before a filter stays and the header counts the hidden ones. Test: `round2_jobs.rs`.

### L6. Ctrl-C ends the program in the middle of a job `[RUN]`

- **Where**: `Tui/src/binding.rs:131` (`QUIT`), `Tui/src/app/handle.rs:27` (checked before the phase), `Tui/src/lib.rs:61` (the loop ends, job threads are not joined).
- **Scenario** (`u3`): `Sync/running 0/3, should_quit: true`. Each swap is atomic, so projects stay consistent, but `.agents/.stage-*` or `.trash-*` of interrupted skills can remain and some projects are updated, others not, with no results page (`[READ]` for the leftovers).
- **Fix**: while `Running`, Ctrl-C shows "waiting for the job to finish, press Ctrl-C again to quit"; the second press quits. Better: a cancel flag the pool checks between targets.
- **Status**: fixed on 2026-10-08: while a job writes, the first Ctrl-C only warns in the footer and the second quits. A cancel flag the pool checks between targets is not built. Test: `round2_jobs.rs`.

### L7. A scan from before a setup change is adopted afterwards `[READ]`

- **Where**: `Tui/src/app/mod.rs:117-125` (`loading` blocks a second spawn), `:177-183` (`write_setup` sets `session = None`), `Tui/src/app/jobs_events.rs:39-59` (`on_loaded` stores any result).
- **Scenario**: `u6` shows the sibling case `[RUN]`: open Sync, Esc, open List before the first scan reports, and the first scan populates the List page with no new scan. With a setup change in between, the old scan of the old vault or root becomes the session of the new settings.
- **Fix**: number the loads; `on_loaded` ignores a result whose number is not current; `write_setup` bumps the number and clears `loading`.
- **Status**: fixed on 2026-10-08: scans are numbered like every job, and saving the setup forgets the scan on its way. Test: `round2_jobs.rs`.

### L8. A read error ends the input thread silently `[READ]`

- **Where**: `Tui/src/event.rs:52` (`Err(_) => break`).
- **Scenario**: the terminal read fails (EIO after the pty closed). The thread ends, the loop keeps drawing on the tick, and no key, Ctrl-C included, ever arrives; the process spins until killed. An error is swallowed.
- **Fix**: send `Event::InputFailed(error)` before breaking, restore the terminal and exit with the error (exit 3).
- **Status**: fixed on 2026-10-08: a read error sends `Event::InputFailed`, the interface ends and `skillmirror` exits 3 with the reason. The app-level path is tested; the thread reading a real dead terminal is not.

### L9. `init --git` obeys `GIT_DIR` from the environment `[RUN]`

- **Where**: `Core/src/ops/project.rs:98-103` (`Command::new("git")` without removing `GIT_*`); `vault/tracked.rs:164-166` and `testutil.rs:47` strip them.
- **Scenario** (K2): `GIT_DIR=<dir>/other.git skillmirror init new2 --git --yes` exits 0, `new2/.git` does not exist and `other.git` was created.
- **Fix**: one shared `git_command()` helper that removes every `GIT_*` variable, used for vault and project.
- **Status**: fixed on 2026-10-08: one `git::command()` helper removes every `GIT_*` variable for the vault and for `init --git`. Tests: `git.rs` and `Cli/tests/init_failures.rs`.

### L10. `--json` with a terminal prints the human rows before the JSON document `[READ]`

- **Where**: `Cli/src/commands/pipeline.rs:149-162` (`confirm_write` prints `render_rows`, then asks), reached with `run.json` set and no `--yes`.
- **Scenario**: `skillmirror sync --json` in a terminal prints colored rows, the question, then the JSON; `jq` fails (script K7 is a note, it needs a tty).
- **Fix**: with `--json` and neither `--yes` nor `--dry-run`, return the usage error ("refusing to write without confirmation"), or print the rows to stderr.
- **Status**: fixed on 2026-10-08: `--json` without `--yes` or `--dry-run` is refused with a usage error before anything is printed. Test: `Cli/tests/json_terminal.rs` (real PTY, red without the guard).

### L11. `leftovers()` swallows per-entry errors and matches on the name only `[READ]`

- **Where**: `Core/src/scan/walk.rs:152-173`.
- **Details**: `.filter_map(Result::ok)` drops entry read errors. Any entry named `.stage-*` or `.trash-*` is reported as "left over from an interrupted run", including a file the user named so and the `.stage-<run>-<index>` of a run still in progress (a Tui scan while a Cli sync runs elsewhere).
- **Fix**: turn entry errors into issues, check `file_type().is_dir()`, and parse the `<pid>-<hex>-<index>` run id (skip a live pid).
- **Status**: fixed on 2026-10-08: only folders count, a run whose process still exists is skipped, entry errors become issues. `undo` names its temporary folders in the same `<pid>-<time>-<index>` form. Tests: `scan/safety_tests.rs`.

### L12. The wizard reads the whole vault on the UI thread `[READ]`

- **Where**: `Tui/src/screens/setup.rs:102-120` (`choose_vault` calls `discover`, `VaultConfig::load`, and imports `read_files`).
- **Scenario**: choosing a big vault runs `git ls-files` and reads every tracked file while the UI is frozen (no redraw, input or resize).
- **Fix**: run it as a job like `spawn_load` and show a "checking the folder" state.
- **Status**: fixed on 2026-10-08: choosing a vault runs `check_vault` as a job, the page shows "Looking at the folder", the picker keeps still, and an answer for a folder the user walked away from is dropped. Tests: `round2_wizard.rs`.

## Info

### I1. Delete and apply treat leftover remains differently `[RUN]`

`Core/src/ops/delete.rs:135-139` returns `RemainsKept` as an error, so the row is `Failed` and the exit code is 4 although the skill is gone from `skills/`. `apply` reports an old copy that cannot be removed as a `Leftover` warning and keeps the row `Updated`. Both are honest; pick one policy and write it in the contract. **Status**: decided on 2026-10-08, no code change, contract Q39: a leftover is a warning when the goal was reached (the new copy of an update is in place) and a failure when it was not (a delete that left remains in the trash is incomplete). The delete path can only be exercised as a non-root user, so it stays covered by the existing non-root test.

## Verified OK (negative results)

- Round 1 fixes hold at the tip (see [round_1_Core.md](round_1_Core.md)); undo of a stale swap keeps the user's edit (`a4`: `DestinationChanged`, `notes.md` kept, `.agents` holds only `skills`).
- `save_config` edge cases that work: indented comment, flow list over two lines, column-0 items, trailing comment on the key, `---` marker, no trailing newline, missing key appended, comment-only file, nested `root:` under `profiles`, unparsable file (untouched), a root with quotes, backslash and `#`, and 15 awkward mandatory names (`"null"`, `"yes"`, `"1e3"`, `"~"`, all quoted).
- Cli (script K): `--dry-run` writes nothing for `add` and `init`; `add` twice reports `up to date`; `add` where `.agents` is a link is refused with a hint; unknown group, unknown profile and an empty selection are hard errors with hints.
- Tui: 15,200 draws, 19 screens (menu, help, selects with and without filter, delete confirm, results with details, failed, loading, running, five wizard steps), 0 panics.
- Code rule: largest non-test file `Tui/src/ui/setup.rs` with 233 code lines, `Core/src/apply/run.rs` 210; largest test file `Core/src/ops/tests.rs` with 247 (awk count, may differ from tokei by a few lines).

## Missing tests

Judged from test function names, not from reading every body.

| Behavior | Covered? |
|---|---|
| Tui: sync or push after a file was added to a project copy while the page was open (M1) | no |
| Tui: confirmation page for sync and push listing removed files | no (feature missing) |
| Tui: header arrow or `open` while a job runs, results on the wrong screen, two jobs (M2) | no |
| Tui: Ctrl-C during `Running` (L6) | no (`ctrl_c_quits_from_anywhere_even_while_typing` covers typing only) |
| Tui: Enter after a filter with hidden selected rows (L5) | partly (`the_filter_narrows_the_rows_and_selection_survives_it` asserts survival only) |
| Tui: wizard with an unsaveable config leaves the pointer unchanged (M5) | no |
| Tui: scan result arriving after a setup change (L7) | no |
| Tui: input thread read error (L8) | no |
| Tui: wizard on a big vault keeps drawing (L12) | no |
| Apply: one failing target among new-project siblings, repeated (M3) | no (`one_failing_target_does_not_stop_the_others` uses existing projects) |
| Apply: same-size edit with restored mtime; touch with identical bytes (L1, L2) | no |
| Scan: stale `.stage-*` of a live run is not a leftover (L11) | no |
| `save_config`: symlinked config, blank line, column-0 comment, quoted key, BOM, CRLF, comments in the block (M4) | no (`save_tests.rs` covers comments outside the block, quoting, malformed, permissions) |
| Profiles: diamond depth, `exclude` in a sibling, both orders (L3, L4) | no (`profiles_parse_and_are_checked_for_unknown_parents_and_loops` only) |
| `init` with every skill failing leaves nothing behind and a retry works (M6) | no (`init_with_an_unknown_profile_creates_no_directory` covers the profile case) |
| `init --git` with `GIT_DIR` in the environment (L9) | no |
| `--json` with a terminal and no `--yes` (L10) | no (needs a pty; `Cli/tests/interface.rs` has a pty harness) |
| `delete` with `RemainsKept` on the Tui results page (I1) | no |
