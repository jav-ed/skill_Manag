# core_Modules

What each module of `Crates/Core/src` owns and which file to open. The order follows the flow of a write.

| Module | Files | Owns |
|---|---|---|
| `config` | `settings.rs`, `vault_config.rs`, `env.rs`, `paths.rs`, `pointer.rs`, `save.rs`, `migrate.rs`, `error.rs` | `Settings` (flags, environment, files, with the source of each value), `VaultConfig` (`root`, `mandatory`, `exclude_*`, `profiles`, `targets`; unknown keys refused), XDG directories (`Dirs`), the vault pointer file, `save_config` (text edits that keep comments, CRLF and a BOM and follow a symlinked config), `migrate` of the old tool's pointer |
| `vault` | `discover.rs`, `tracked.rs`, `header.rs` | `discover` (skills are folders with `SKILL.md`, groups are the folders above, names unique vault-wide), `read_files` (git's tracked files per skill in two calls, with untracked files and problems such as links), `read_header` (name and description of `SKILL.md`) |
| `scan` | `walk.rs`, `prune.rs`, `targets.rs`, `links.rs` | `scan` and `scan_with_progress` (parallel walk, `ScanReport` with `skills_dirs` and `issues`), `ScanOptions` and the noise list, `Target` sets for sync, push and delete, `same_folder` and `first_link_above` |
| `plan` | `build.rs`, `inspect.rs`, `set.rs`, `model.rs`, `error.rs` | `Plan` (`for_targets`, `for_targets_creating`), `SkillPlan` (kind, changes, removed files), the snapshot of a target (length, mode, mtime, ctime) that apply compares again |
| `apply` | `run.rs`, `place.rs`, `stage.rs`, `swap.rs`, `error.rs` | `apply` (per-target results, progress events), staging beside the target, `renameat2` swap and create, leftover handling, one `skills/` creation per new project |
| `backup` | `store.rs`, `undo.rs`, `tree.rs`, `pathjson.rs`, `clock.rs`, `error.rs` | `Backups` and `Run` (a run folder with one entry per saved skill), `undo` (a run itself), `describe_run`, `now_utc`, exact-bytes project paths in notes |
| `ops` | `workspace.rs`, `sync.rs`, `push.rs`, `delete.rs`, `list.rs`, `select.rs`, `project.rs`, `status.rs`, `diff.rs`, `report.rs`, `bridge.rs`, `doctor/` | What the front ends call: `Workspace::open` and `scan`, `plan_sync`, `plan_push`, `plan_install`, `delete`, `installed`, `resolve` (names, groups and profiles to skills), `check_existing` and `create_new`, `status`, `diff` and `diff_of_plan`, `report_data`, bridges, `doctor` in layers |
| `events` | `events.rs` | `Event` (`ScanProgress`, `ScanFinished`, `TargetDone`) and `Observer`; the CLI, the TUI and any later view read one stream |
| `git`, `runid`, `brand`, `error` | one file each | `git::command` (a `git` call with every `GIT_*` variable removed), unique names for staging folders, the names the tool goes by, the top-level `Error` and the `Hint` trait |

Tests sit beside the code as `*_tests.rs`; `testutil.rs` holds the `TempTree` they share.
