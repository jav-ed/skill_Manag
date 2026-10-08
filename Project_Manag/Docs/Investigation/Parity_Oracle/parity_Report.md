# Parity oracle: Report

Result of the last full parity run (round 2, 2026-10-07): the Go tool at commit `c7310f9` against the `skillmirror` debug binary built from the working tree with source hash `128acde33cdb8586` (hash over `Crates/`, `Cargo.toml`, `Cargo.lock`; binary sha256 starts with `704bad01ce78a25e`, reports `skillmirror 0.1.0`). This is a condensed copy of the generated `report.md` and the notes in `observations.md`; the full report (every differing step with commands and output heads) is rebuilt by `check_Parity.sh` and is not committed. Later runs will inshallah replace the numbers here; the table at the top says which round they belong to.

## Result

| class | scenarios |
|---|---|
| MATCH | 87 |
| EXPECTED-DIVERGENCE | 44 |
| UNEXPECTED | 0 |
| SKIPPED | 0 |
| **total** | **131** |

Why each of the 44 differs: [divergences_Explained.md](divergences_Explained.md). Round 1 (123 scenarios, an earlier build) had 85 MATCH, 37 EXPECTED-DIVERGENCE and 1 UNEXPECTED (`cfg_root_is_symlink`, a Go quirk the contract did not list yet; it became Q31).

## What was compared

Per step: the whole projects tree after the call (type, mode, sha256, path; exact), the exit code (Rust codes 0 clean, 1 drift, 2 usage, 3 hard error, 4 partial, mapped from the Go code: Go 0 without error is 0; Go 0 with an error line is 3 or 4; Go 1 with a cobra usage error is 2; Go 1 otherwise is 3; Go 124 is 124), and the set of (project, skill, file-count) from Go's dry-run text or driver lines against `sync` or `push --dry-run --json --all` taken before the real call. Per scenario: everything outside `projects/` and `home/` (vault files, sentinel files).

Not compared: stdout and stderr text, help text, the order of project blocks, the Rust stale and changed counts, the config directory (`skill_Manag` against `skillmirror`, different by design).

Coverage: 153 steps; tree compared in 153, exit code in 153, the set in 69 (84 steps have no Go set because Go printed none, or no Rust plan JSON because the call was a usage error).

## Round 2 changes

- Class moves: `cfg_root_is_symlink` UNEXPECTED to EXPECTED-DIVERGENCE (Q31); `del_skill_dir_is_symlink` EXPECTED-DIVERGENCE to MATCH (Rust now removes a symlinked skill folder with `delete --project`, link only, like Go); `scan_exclude_paths_outside_root` MATCH to EXPECTED-DIVERGENCE (Q32: the entries do not exist).
- Eight new scenarios: `cfg_config_root_empty`, `cfg_root_is_symlink_delete`, `del_project_through_symlinked_agents`, `del_project_through_symlinked_skills_dir`, `scan_exclude_empty_entries`, `scan_exclude_paths_nonexistent` (all EXPECTED-DIVERGENCE), `cfg_vault_git_zero_skills`, `del_project_symlinked_skill_dangling` (MATCH).
- Fixes confirmed by re-running: the read-only skill folder hint names permissions (`drv_sync_readonly_project_dir`); `--vault <regular file>` says "is not a directory" (`cfg_vault_is_a_file`); a tracked symlink makes the skill a hard error, which the decision file now says too.

## Observations (not parity failures)

1. Delete goes through `.agents/.trash-*`. On a read-only skill folder the rename fails with EACCES, the skill stays complete and no `.trash-*` remains (`del_readonly_skill_*`). A leftover after a crash between rename and removal cannot be provoked from outside.
2. Empty git vault (`cfg_vault_git_zero_skills`): Rust prints "No skills found in vault." and exits 0; Go prints "No matching skills found in any project." because it counts its own `.git` folder as a skill (Q6). Both end with an empty target set, so the scenario is MATCH.
3. Canonical paths in the output: Rust prints absolute canonical project paths even for a relative `--root` and for a symlinked root. Output text is not compared; the set comparison maps both spellings.
4. A no-op sync rewrites every file in Go (mtimes change; `drv_sync_noop/observations.txt` counts 2 rewrites against 0 in Rust). The trees are identical.
5. Run-to-run stability of the Rust side: two full replays agree in every compared artifact. The only byte differences are the `.stage-<pid>-<timestamp>-<n>` and `.trash-<pid>-<timestamp>-<n>` names inside the stdout of three failure scenarios (`drv_sync_readonly_project_dir`, `del_readonly_skill_by_name`, `del_readonly_skill_with_project`).

## Known gaps

- No scenario uses a scan root that contains `..`. Q31 mentions it (the root is canonicalized and `exclude_paths` match the real path), but the scenarios only use a symlink. A scenario `cfg_root_dotdot` (`--root <TMP>/projects/../projects`, plus an `exclude_paths` entry spelled through the `..` form) will inshallah close this.
- The Go sync-apply and push flows exist only through the driver stand-in (`oracle_drv.go.txt`), because the real binary needs a terminal for them. A run of the Go TUI under a real pseudo-terminal would be the faithful check of the stand-in; it was not done (no faked terminal).
- Rust-only features have no Go counterpart and no scenario: `--check`, `--json` on apply, `list --json`, `migrate`, `completions`, and the new `status`, `diff`, `doctor`, backup store and `undo` (see the last section of [oracle_Setup.md](oracle_Setup.md)).
- The recording pins git's config but not the git version, and the Go binary hash depends on the toolchain; see the reproducibility notes in [oracle_Setup.md](oracle_Setup.md).
- The numbers describe one build. Any later change to `Crates/` makes them stale until `check_Parity.sh` runs again.
