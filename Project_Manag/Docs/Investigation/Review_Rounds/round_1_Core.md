# Review rounds: Round 1, Core

Adversarial read of `Crates/Core` (scan, vault, plan, apply, ops, config) by a second session, 2026-10-07. Every finding is listed with severity, where, scenario, and its status at the tip of the working tree checked on 2026-10-08. For fixed findings the test in `Crates/` that now covers it is named; each name was found with grep in the tip, not guessed.

## Scope and state checked

- **Reviewed snapshot**: `Crates/Core/src` as it stood at 17:34 on 2026-10-07, hash `f0cc86667dbdba29` (sorted `sha256sum` of the `sha256sum` of every `.rs` file). The code moved while the review ran; findings in `ops/project.rs`, `ops/select.rs` and `config/vault_config.rs` profiles were left to [round 2](round_2_Full.md).
- **Checked against**: the working tree on 2026-10-08, 135 `.rs` files under `Crates/`, hash `d01061c0fe7217c4` (computed from the repository root over `Crates/`, see [review_Method.md](review_Method.md)). That tree also holds a half-built backup module that was not reviewed (see round 2).
- **Original reproducers** (series E and R, about 40 tests) lived in a throwaway copy and are not kept: every fixed finding now has a permanent test in `Crates/`, named below.
- **Markers**: `[RUN]` = reproduced with a test that printed the failure. `[READ]` = from reading the code only.
- **Severity**: High = silent data loss or a protection silently off, with a realistic input. Medium = wrong result or data loss with a less common precondition, or a hard-error-rule violation with real impact. Low = misleading error, cleanup gap, robustness. Info = observation.

## Summary

| Severity | Count | Status at the tip |
|---|---|---|
| High | 2 (H1, H2) | both fixed and tested |
| Medium | 6 (M1 to M6) | all fixed and tested; M1 has a remaining gap (round 2, L1 and L2) |
| Low | 8 (L1 to L8) | L1 to L5, L7, L8 fixed; L6 documented, not changed |
| Info | 3 (I1 to I3) | I1 open; I2 obsolete; I3 reference number |

Test paths below are relative to `Crates/Core/src/`.

## High

### H1. `exclude_paths` is ineffective when the scan root contains `..` `[RUN]`

- **Where**: `config/settings.rs` (`std::path::absolute` keeps `..`), `scan/prune.rs` (cleaned the entry but not the root), `scan/walk.rs` (the walker yields paths under the unclean root).
- **Scenario**: vault `exclude_paths: [secret]`, root `<tmp>/x/../real`. Scan found `keep`, `other` and `secret`. A symlinked root with an exclude given as the real path excluded nothing either. A user running `sync --root ../Projects` synced projects they had excluded, silently.
- **Fix applied**: the scan canonicalizes the root once; entries are canonicalized when they exist.
- **Status**: **fixed**. Tests: `scan/tests.rs`: `a_root_containing_dotdot_still_honours_exclude_paths`, `a_symlinked_root_is_walked_and_excludes_given_by_the_real_path_apply`, `exclude_path_that_climbs_out_is_a_hard_error`. Contract entry: Q31 in [behavior_Contract.md](../../Descr/behavior_Contract.md).

### H2. Untracked `SKILL.md` with other tracked files: sync succeeded and removed `SKILL.md` `[RUN]`

- **Where**: `plan/build.rs` rejected only an empty tracked list; `vault/discover.rs` decides "is a skill" from the filesystem, `vault/tracked.rs` decides the file list from git.
- **Scenario**: vault skill `sk` has tracked `ref.md` and an uncommitted `SKILL.md`. The project had a working `sk/SKILL.md`. Result: `Updated`, the installed folder held only `ref.md`, the agent no longer saw the skill.
- **Fix applied**: `PlanError::SkillFileNotTracked`, hint `git add <skill>/SKILL.md`.
- **Status**: **fixed**. Test: `plan/tests.rs`: `an_untracked_skill_file_next_to_tracked_files_is_a_hard_error_and_the_copy_stays`.

## Medium

### M1. The plan was stale at apply time `[RUN]`

- **Where**: `apply/run.rs` never re-read the destination; `plan/build.rs` `compare` was the only look.
- **Scenario**: plan built (Update), then the user adds `my_notes.md` and edits `SKILL.md`, then apply runs: `my_notes.md` deleted, never shown in `plan.removed`. Also true for `Unchanged` plans.
- **Fix applied**: the plan keeps the snapshot of the destination (`Entry::File { len, mode, modified }`); after the directory exchange the old copy is walked and compared, and a mismatch undoes the swap with `ApplyError::DestinationChanged`.
- **Status**: **fixed for the Core API**. Tests: `apply/stale_tests.rs`: `an_edit_made_after_planning_is_never_overwritten`, `an_unchanged_plan_is_not_trusted_after_an_edit`, `an_edit_that_keeps_the_size_is_noticed_too`. Two gaps remain, see round 2 L1 (same size and mtime) and L2 (spurious failure on a mere touch). The Tui does not benefit from the guard at all, see round 2 M1.

### M2. Delete (and any explicit `--project`) followed a symlinked `.agents` or `.agents/skills` `[RUN]`

- **Where**: `ops/delete.rs` (`target_in_project` checked only the last component), `plan/build.rs` (`skills_dir_exists` only in the Create branch).
- **Scenario**: `proj/.agents -> shared`; `target_in_project(proj, "coding")` succeeded and `shared/skills/coding` was deleted, data outside the project. Apply had the same property by reading.
- **Status**: **fixed**. Tests: `ops/safety_tests.rs`: `delete_through_a_symlinked_agents_directory_is_refused`, `delete_through_a_symlinked_skills_directory_is_refused_even_for_a_ready_made_target`; `plan/tests.rs`: `planning_through_a_symlinked_agents_directory_is_refused_in_every_case`; `ops/project_tests.rs`: `check_existing_refuses_missing_projects_and_linked_agent_directories`. Contract entry: Q33.

### M3. Non-UTF-8 names were lossy or silently dropped in three places `[RUN]`

- **Where**: `vault/discover.rs` (folder name passed through `to_string_lossy`, so git's raw bytes never matched and the skill showed zero tracked files), `scan/targets.rs` (installed folder dropped by `.to_str()` with no issue), `vault/discover.rs` group parts (`filter_map(|p| p.to_str())`, `[READ]`).
- **Status**: **fixed** for the first two. Tests: `vault/safety_tests.rs`: `a_skill_folder_with_a_non_utf8_name_is_a_hard_error`; `scan/safety_tests.rs`: `an_installed_folder_with_a_non_utf8_name_is_reported_not_dropped`. The third place still has `filter_map(|p| p.to_str())` (`vault/discover.rs:190`) but should be unreachable now because line 122 rejects a non-UTF-8 name first (`[READ]`, no test). Tidy it into a hard error to keep the rule visible.

### M4. A deep folder without any skill made the whole vault unusable `[RUN]`

- **Where**: `vault/discover.rs` raised `TooDeep` before knowing whether the subtree held a skill.
- **Scenario**: vault with `coding/SKILL.md` and `notes/a/b/c/d/e/readme.txt` failed every command.
- **Status**: **fixed**. Tests: `vault/safety_tests.rs`: `a_deep_folder_without_any_skill_does_not_break_the_vault`, `a_deep_folder_that_does_hold_a_skill_is_still_too_deep`.

### M5. A successful update was reported as Failed, and `.stage-*` orphans were never reported `[RUN]` + `[READ]`

- **Where**: `apply/run.rs` (`OldCopyKept` returned as a failure), no sweeper or detector anywhere.
- **Scenario**: the old copy had a `0o555` subdirectory; the exchange succeeded, `SKILL.md` was new, but the outcome was `Failed(OldCopyKept)` and `.agents/.stage-<run>-0` stayed forever, as an untracked folder in the project's git. After a crash between staging and rename the same orphan stayed unnoticed.
- **Status**: **fixed**. Tests: `apply/stale_tests.rs`: `an_old_copy_that_cannot_be_removed_is_a_warning_not_a_failure`; `scan/safety_tests.rs`: `stage_and_trash_folders_of_an_interrupted_run_are_reported`. The scan reports leftovers as issues; it does not delete them. The detector has its own weaknesses, see round 2 L11.

### M6. `exclude_paths` entries that match nothing were never reported; a real run would touch the oracle fixtures `[RUN]` + `[READ]`

- **Where**: `scan/prune.rs`, `config/vault_config.rs` (no validation).
- **Scenario**: with the real vault config and the real root, a plan included the Go oracle's fixture projects under `Scratch/Oracle/...` and would have removed `STALE_FILE.md` from a fixture. A typo in `exclude_paths` fails the same way, silently, as H1.
- **Status**: **fixed** (code). Test: `scan/tests.rs`: `an_exclude_path_that_does_not_exist_is_a_hard_error`. Contract entry: Q32. Operational note, not code: before any real `sync` or `push`, check that the real vault config excludes the oracle and test-data folders, because the scan root contains them.

## Low

- **L1. A submodule that is itself the skill gave a misleading error** `[RUN]`: `vault/tracked.rs` never attributed the gitlink entry to its own skill. **Fixed.** Test: `vault/safety_tests.rs`: `a_submodule_that_is_the_skill_folder_itself_is_named_as_such`.
- **L2. Delete was not atomic** `[RUN]`: `remove_dir_all` on a skill with a read-only subfolder removed `SKILL.md` first and failed, leaving a half-deleted skill that still counted as installed. **Fixed** by renaming into `.agents/.trash-<run>-<i>` first, then removing. Tests: `ops/safety_tests.rs`: `a_delete_that_cannot_finish_still_takes_the_skill_out_of_the_skills_directory`, `a_delete_leaves_no_trash_folder_behind`. Whether the remains count as a failure is round 2 I1.
- **L3. The vault pointer was written lossily for non-UTF-8 paths** `[RUN]`: `/tmp/v\xffault` came back as another path. **Fixed.** Test: `config/safety_tests.rs`: `a_vault_path_that_is_not_utf8_survives_the_pointer_file`.
- **L4. Empty or unvalidated `root` gave a context-free error** `[RUN]`: `root: ''` failed with "cannot make an empty path absolute" without naming key or file. **Fixed.** Test: `config/safety_tests.rs`: `an_empty_root_or_exclude_path_names_the_key`. Contract entry: Q32.
- **L5. A symlinked scan root is walked, unlisted difference from Go** `[RUN]`: Go finds nothing without a trailing slash, Rust finds the projects either way. **Fixed** by recording it: contract entry Q31 plus test `scan/tests.rs`: `a_symlinked_root_is_walked_and_excludes_given_by_the_real_path_apply`.
- **L6. No `fsync`: atomic against a crash of the process, not against power loss** `[READ]`: after power loss the exchanged directory can hold zero-length files. **Documented, not changed**: the doc comment of `apply` in `apply/run.rs` now says the guarantee holds for kills and failed writes, not for power loss, and gives the reason (one `fsync` per file over hundreds of targets costs seconds; the copies are git-tracked and the next sync restores them). No test (a doc decision).
- **L7. Hard-error-rule nits** `[READ]`: seven spots that hid a failure path. Status per bullet at the tip:
  - clock error swallowed in `run_id`: **fixed**, `runid.rs` returns `io::Result` and `apply` propagates it with `?`. No test (a clock error cannot be provoked).
  - `file_type()` error treated as "not a directory" in `scan/targets.rs`: **fixed**, the error becomes a `ScanIssue`. No dedicated test.
  - `remove_dir(dir).ok()` in `config/migrate.rs`: **fixed**, only `DirectoryNotEmpty` is tolerated. Test: `config/safety_tests.rs`: `retiring_the_old_pointer_keeps_a_config_directory_that_holds_other_files`.
  - issue with an empty path in `scan/walk.rs`: **fixed**, `error_path` returns an `Option` and the scan root is used when the error has no path. No dedicated test.
  - `get_mut(filled..).unwrap_or_default()` in `plan/inspect.rs`: **fixed**, uses `split_at_mut_checked`. No test (unreachable branch).
  - `filter_map` over a function that always returns `Some` in `config/error.rs`: **fixed**, now `map`.
  - empty `exclude_paths` entries dropped silently in `scan/prune.rs`: **changed to a hard error** (Q32), test `an_empty_root_or_exclude_path_names_the_key`.
- **L8. `create_dir_all` before staging left an empty `.agents/skills` on failure** `[READ]`: **fixed**, a failed create removes the directories it made. Test: `apply/stale_tests.rs`: `a_failed_create_in_a_new_project_leaves_no_empty_agents_directory`. The fix has a side effect that round 2 M3 reports: the cleanup races with sibling targets.

## Info

- **I1 `[RUN]`, open**: one unmerged index entry anywhere in the vault, even `README.md` outside every skill, makes `read_files` fail for the whole vault (`vault/tracked.rs:73`). It is a hard error, so it is safe, but the scope is wider than the skills. Consider failing only for entries that belong to a skill. No test.
- **I2, obsolete**: the review said deleting a symlinked skill folder was unreachable. The code now allows it on purpose through an explicit `--project` (Q33): the link goes, nothing behind it. Tests: `ops/safety_tests.rs`: `an_explicit_project_may_delete_a_symlinked_skill_folder_and_only_the_link_goes`; `ops/tests.rs`: `deleting_a_symlinked_skill_removes_the_link_not_its_target`.
- **I3, reference numbers** `[RUN]`: a scan over the real root (67 skills directories, 0 issues) took 2.09 s in one run with unknown cache state; `plan_sync` over 436 targets took 134 ms.

## Verified OK in round 1 (negative results, `[RUN]`)

- Swap races: a symlink swapped into the destination after planning (the linked directory is untouched), a destination that vanished (`Failed`, no leftover stage), one that appeared (`AlreadyExists`, the other party's file kept), `.agents/skills` removed after planning (hard fail, stage cleaned).
- Four duplicate targets for one path applied in parallel, 20 rounds: all `Updated`, no leftover, content correct.
- File names with a tab, a newline and a leading space round-trip through `ls-files -z --stage` and apply. A symlink inside the old destination pointing outside leaves the outside tree intact.
- Scan order equals depth-first byte order for awkward names; entries `.` and `<root>` exclude everything, as in Go.
- Zero tracked files over an existing destination ends in `Failed(NoTrackedFiles)` and the destination is intact (contract Q4).
- YAML: duplicate keys, tab indentation, wrong-case and unknown keys are hard errors with line and column; CRLF and a BOM are accepted.
- Real data, plan only: 24 skills, 338 tracked entries, sync plan 101 update and 335 unchanged, push plan 127 create, 86 update and 256 unchanged, 0 failed.
- `cargo clippy --all-targets -- -D warnings` and `rustfmt --check` were clean on the snapshot.

## Test gaps that remain from round 1

| Behavior | Covered at the tip? |
|---|---|
| Tab and newline in tracked file names | no (only `non_ascii_file_names_survive_the_round_trip`) |
| Unmerged index entry, inside and outside a skill (I1) | no |
| Two `apply` runs of one project at once | no |
| Destination containing a FIFO or other special file | no |
| Swap failures: destination vanished, appeared, skills directory removed | no (behavior verified OK in round 1) |
| Non-UTF-8 group names (M3, third place) | no |
