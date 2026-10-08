# Parity oracle: Divergences explained

The 44 scenarios of the 131 where `skillmirror` differs from the frozen Go tool on purpose, grouped by the quirk id of [the behavior contract](../../Descr/behavior_Contract.md) that decided the change. The machine-readable table (scenario, reference, which checks it covers, long reason) is [divergences.tsv](divergences.tsv); `compare.py` reads it, so an entry that stops being true turns its scenario into UNEXPECTED. Counts at the last run (round 2, binary with sources hash `128acde33cdb8586`): 87 MATCH, 44 EXPECTED-DIVERGENCE, 0 UNEXPECTED.

Reading the lines: "Go" is the tool at commit `c7310f9`, "Rust" is `skillmirror`. Rust exit codes: 0 clean, 1 drift with `--check`, 2 usage, 3 hard error, 4 partial (some targets done, some refused). "Tree untouched" means the projects tree after the Rust step equals the tree before it (the `tree=pre` constraint in the table). Entries marked "design" cite [the rewrite decision](../../Decisions/rust_Rewrite.md) instead of a quirk id.

## Q2: names that git quotes (2)

`git ls-files` quotes non-ASCII and quote characters, Go used the quoted string as a path. Rust reads `ls-files -z`.

- `drv_sync_filename_double_quote`: Go wipes the project copy (stat fails on the quoted name); Rust copies the file, exit 0.
- `drv_sync_filename_unicode_default_git`: same with `ünï.md` under git's default `core.quotepath`; Rust copies it, exit 0.

## Q3: delete-then-copy is not atomic (2)

Go deletes the destination before copying, so any copy error leaves a partial tree. Rust stages and swaps; a refused skill keeps its old copy.

- `drv_sync_vault_file_deleted_from_disk_only`: a tracked file missing on disk. Go leaves a half-written project copy; Rust names the missing file, skill untouched, exit 4.
- `drv_sync_readonly_project_dir`: read-only skill folder in one project. Go fails midway after removing part of it; Rust's swap fails with EACCES (hint blames permissions), that copy stays complete, other targets sync, exit 4.

## Q4: skill with zero tracked files (2)

- `drv_sync_vault_skill_zero_tracked_files`: Go wipes matching project copies and reports ok with 0 files; Rust fails that skill and keeps the copies, other skills sync, exit 4.
- `scan_vault_skill_with_zero_tracked_files`: dry run. Go plans "would sync (0 files)", exit 0; Rust marks the skill failed, plans the others, exit 4.

## Q5: unvalidated skill names in `delete` (6)

Go joins the name into a path without checks. Rust rejects names that are empty, `.`, `..` or contain a separator, before touching anything (exit 2, tree untouched).

- `del_empty_name_with_project`: Go removes `.agents/skills`.
- `del_dot_name_with_project`: Go removes `.agents/skills`.
- `del_dotdot_name_with_project`: Go removes the whole `.agents` folder.
- `del_traversal_name_with_project`: Go deletes `<project>/src` through `../../src`.
- `del_subpath_name_with_project`: Go removes a subfolder of a skill (`default-tools/Hk`).
- `del_empty_name_without_project`: Go finds nothing and exits 0; Rust rejects the empty name before scanning, exit 2.

## Q6: vault directories that are not skills (1)

- `scan_vault_hidden_dir_is_a_skill`: Go treats any vault subfolder as a skill, dot-folders included; Rust skips dot-folders, so a project's `.hidden` copy is no target.

## Q11: prune list applied to the scan root (1)

- `scan_exclude_dirs_name_of_root`: `exclude_dirs: [projects]` matches the root's own name, Go skips the whole walk. Rust does not apply directory exclusions to the root itself, so all 12 targets are found.

## Q12: missing or non-directory scan root (2)

Go prints "No matching skills found" and exits 0. Rust: policy "hard errors, no fallbacks".

- `cfg_root_nonexistent`: Rust says the scan root does not exist, exit 3.
- `cfg_root_is_a_file`: Rust says the scan root is not a directory, exit 3.

## Q15: `delete --project` for something that is not there (2)

- `del_project_nonexistent`: Go prints "deleted from" for a project path that does not exist, exit 0; Rust says it is not an installed skill folder, exit 3.
- `del_project_lacks_skill`: same for a project that lacks the skill; Rust exit 3, nothing touched.

## Q16: partial failure reported as success (2)

Go exits 0 (or counts the project as updated) after a failed delete. Rust renames the folder into `.agents/.trash-*` first; on a read-only folder that rename fails with EACCES, so the skill stays complete and no `.trash-*` remains.

- `del_readonly_skill_by_name`: other projects are deleted, exit 4 (Go: exit 0, counted as updated).
- `del_readonly_skill_with_project`: skill stays complete, exit 4, tree untouched (Go removes part of it, exit 1).

## Q20: unknown names in the mandatory list (2)

- `drv_push_mandatory_missing_in_vault`: `mandatory = [ghost, coding]`. Go drops `ghost` silently and pushes `coding`; Rust refuses the whole push naming `ghost`, exit 3, nothing written.
- `drv_push_mandatory_none_in_vault`: `mandatory = [ghost]`. Go: nothing to push, exit 0; Rust: hard error naming `ghost`, exit 3.

## Q26 and Q29: symlinks in the vault and the non-git fallback (8)

Go dereferences tracked file symlinks, fails on directory symlinks after wiping, and falls back to a plain directory walk when the vault is not a git repository. Rust refuses the skill that contains a tracked symlink, and a vault that is not a git repository is a hard error.

- `drv_sync_symlink_to_file`: Go copies the target's content as a regular file; Rust fails that skill ("contains a tracked symlink"), exit 4, copies untouched.
- `drv_sync_symlink_outside_vault`: Go copies a file from outside the vault through the link; Rust fails the skill, exit 4.
- `drv_sync_symlink_to_dir`: Go leaves an empty file named like the link after wiping; Rust refuses the skill up front, exit 4.
- `drv_sync_symlink_dangling`: Go fails after wiping (partial tree); Rust refuses the skill up front, exit 4.
- `scan_dry_run_symlinks_in_git_vault`: Go counts tracked symlinks as files; Rust marks the skill failed, plans the others, exit 4.
- `drv_sync_vault_not_git`: Go walks the folder (untracked files in, `build/` out); Rust exit 3, nothing written.
- `drv_push_vault_not_git`: same for `push`; Go pushes including untracked files, Rust exit 3.
- `scan_dry_run_vault_not_git`: Go counts untracked files in the fallback walk; Rust exit 3.

## Q31: scan root that is a symlink (2)

Go's walker does not descend into a symlinked root unless the path ends in a slash. Rust resolves the root to its canonical path, so both spellings find the same projects.

- `cfg_root_is_symlink`: `--root link` finds nothing in Go and all 12 targets in Rust (a trailing slash agrees).
- `cfg_root_is_symlink_delete`: `delete coding --root link` deletes nothing in Go and from all 4 projects in Rust.

## Q32: config values that cannot mean anything (4)

Rust checks a config file whenever it is read, even when a flag overrides the key. Go ignores all of these silently.

- `cfg_config_root_empty`: an empty `root:` with an explicit `--root` works in Go, exit 3 in Rust ("root must not be empty").
- `scan_exclude_empty_entries`: an empty `exclude_paths` entry or `exclude_dirs` name; Rust exit 3, tree untouched.
- `scan_exclude_paths_nonexistent`: entries that match nothing on disk, relative or absolute; Rust exit 3, tree untouched.
- `scan_exclude_paths_outside_root`: the entries `../elsewhere` and `/nonexistent/abs` do not exist, so Rust exits 3 (this scenario matched in round 1 before Q32 existed).

## Q33: skill folders behind a symlinked `.agents` (2)

- `del_project_through_symlinked_agents`: Go's `RemoveAll` follows the link and deletes a folder outside the project; Rust refuses (exit 3), nothing deleted.
- `del_project_through_symlinked_skills_dir`: the same through a symlinked `.agents/skills`; Rust exit 3.

## Design decisions without a quirk id (6)

- `cfg_config_yaml_malformed`: Go ignores a malformed `config.yaml`; Rust fails with line and column (unknown keys are errors too), exit 3. Reference: rewrite decision, section 2, config read.
- `cfg_vault_empty_dir`: an empty vault directory that is not a git repository. Go says "No skills found", Rust says "not a git repository", exit 3. (The empty vault that is a git repository is `cfg_vault_git_zero_skills`, a MATCH.)
- `scan_exclude_dirs_from_env`: Go reads `SKILL_MANAG_EXCLUDE_DIRS`; in Rust any `SKILL_MANAG_*` variable is a hard error with a hint to use `config.yaml`, exit 3.
- `tty_delete_interactive_no_tty`: Go `delete` without a name opens a TUI; Rust requires the name (usage error, exit 2).
- `tty_list_no_tty`: Go `list` is a TUI and fails without a terminal; Rust `list` is a plain listing, exit 0.
- `tty_menu_no_tty`: Go opens the menu TUI (fails without a terminal, exit 1); Rust prints help and exits 2 until the TUI exists (phase 4 of the plan). This entry will inshallah go away when the menu lands.
