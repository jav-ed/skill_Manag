# Review rounds: Round 3, Self review

A review of the work of 2026-10-08 (backup store and undo, the round 2 fixes, the plan-first interface) by the same session that wrote it, because the user ruled out any other agent for this work. It is weaker than a second pair of eyes, so a fresh review of the same code is still welcome. The method was to read each change asking what it broke, to turn every suspicion into a test, and to count a suspicion as a finding only when the test failed on the code as it was.

## Findings

| # | Severity | Finding | Status |
|---|---|---|---|
| S1 | Medium | A project folder whose name is not valid UTF-8 could not be synced any more: the backup note stores the project path as JSON, and JSON cannot hold such a path, so every write into that project failed with "path contains invalid UTF-8 characters". No data was at risk, but a project that worked before the backups stopped working. | Fixed. `backup/pathjson.rs` stores a path as text when it is valid UTF-8 and as its raw bytes otherwise. Test: `backup::apply_tests::a_project_whose_path_is_not_valid_utf8_can_still_be_synced_with_a_backup` (red first). |
| S2 | Medium | The same project made `sync --yes --json` fail with exit 3 after the files were written, because the JSON rows hold the path as a string (found while testing S1; older than the backup work). | Fixed. Project paths in the JSON documents are shown with replacement characters (`output/lossy.rs`); the backup note keeps the exact bytes. Test: `Cli/tests/odd_paths.rs` (red first). |
| S3 | Low | The note of a created skill that was removed by hand stayed after `undo` said "already gone", so it stayed the newest run and a plain `undo` could never reach the older runs. | Fixed. The note is spent. Test: `backup::undo_tests::a_note_for_a_skill_that_is_already_gone_does_not_block_older_runs` (red first). |
| S4 | Low | `undo --project` compared the folder as text with the stored path, so a path given through a symlink matched nothing. | Fixed. Folders are compared by what they are (`canonicalize`). Test: `backup::undo_tests::a_project_filter_given_through_a_symlink_still_finds_the_project`. |
| S5 | Low | A second Enter while the interface was still working out a plan counted as "back" and dropped the plan. | Fixed. Only back and Esc leave the planning page. Test: `Tui/src/tests/round2_jobs.rs::a_second_enter_while_the_plan_is_made_does_not_cancel_it` (mutation-checked). |
| S6 | Low | The confirmation page looked at the disk (one `lstat` per removed file) on every redraw, including every mouse move. | Fixed. The preview is worked out once, in the planning job, and carried in the pending run. |
| S7 | Info | The error for a backup note that cannot be written read "cannot read the backup note". | Fixed (wording). |

## Suspicions that did not hold

- **`delete --dry-run` needs the state directory** (it began a backup run before the dry-run check). Tried without `HOME` and the XDG variables: the old code worked too, because the XDG lookup falls back to the passwd entry and `begin` writes nothing. No change was made.

## Left as it is, on purpose

- Pruning after an undo can remove the oldest run even when that run still holds notes an undo could not apply (only when exactly the limit of 30 runs exists). The retention rule is "the newest 30", and the user can name the run before it reaches the limit.
- A crash between "note a created skill" and "create it" leaves a note for a skill that never existed; a later `undo` would remove a skill the user made with that name. The removed folder is kept in the store by the undo itself, so a second `undo` brings it back.
- `undo` on a folder that the user changed after the run replaces it; the replaced folder is stored the same way.
