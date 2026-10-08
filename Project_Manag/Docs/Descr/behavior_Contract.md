# behavior_Contract

Observable behavior of the Go `skill_Manag` tool as frozen at commit `c7310f9` (branch `rust-rewrite`), written so a reader who never saw the Go code can reimplement it exactly. Citations are `file:line` into that commit (module `skill_Manag`: `main.go`, `cmd/*.go`, `cmd/tui/*.go`, `internal/*.go`, `styles/styles.go`). Items marked **[P]** were also confirmed by running the built binary or a probe against throwaway sandbox directories, not only by reading. Quirks carry a tag: **KEEP** (port as is), **CHANGE** (port should differ, reason given), **UNCLEAR** (needs a decision). Nothing here fixes anything.

Literal output strings are in code spans and are verbatim, including the few U+2014 dashes and `…` that the Go source contains.

Sections 1 to 9 describe the Go tool. The Rust tool keeps what is tagged KEEP and changes what is tagged CHANGE; the quirk table of section 9 also carries the rules that exist only in the Rust tool (from Q34 on: backups and `undo`, profiles, the interface's plan-first discipline, `status`, `diff`, `doctor`, `bridge`, the scan progress line, the interface pages for scan problems, history, add, init and changes, and `report`). Each such row names the code and the tests that hold it.

## 1. Commands, flags, exit codes

`main.go:5-7` only calls `cmd.Execute()`. The CLI is built with cobra; `Execute` exits 1 on any error returned by a handler (`cmd/root.go:38-42`), otherwise 0.

| Invocation | Behavior |
|---|---|
| `skill_Manag` | Root handler `runMenu` (`cmd/sync.go:19-75`): resolve config (section 2), run Setup if vault or root is empty, then the menu loop (section 8). |
| `skill_Manag --dry-run` | Non-interactive sync preview (section 5). `--dry-run` is a local flag of the root command only (`root.go:52`), so `list --dry-run` is an unknown-flag error. |
| `skill_Manag list` | Browse TUI (`cmd/list.go:34-43`). Needs `root`; `vault` optional. |
| `skill_Manag delete [name]` | Max 1 arg (`cobra.MaximumNArgs(1)`, `delete.go:48`). No arg: delete TUI. With name: CLI delete (section 6). |
| persistent flags | `--vault <dir>`, `--root <dir>` on every command (`root.go:48-49`). `delete` adds `--project <dir>` and its own `--dry-run` (`delete.go:54-55`). |
| `help`, `completion`, `-h` | Cobra built-ins, exit 0. |

There is **no `sync` or `push` subcommand** [P]: `skill_Manag sync` prints `Error: unknown command "sync" for "skill_Manag"` then `Run 'skill_Manag --help' for usage.` to stderr, exit 1. Sync is the root action (menu entry or `--dry-run`); push exists only in the TUI (`push.go:8-12`, no flags, no dry run).

Error path [P]: cobra prints `Error: <message>` to stderr, then the full `Usage:` block of the failing command (SilenceUsage is not set), exit 1. Parity tests should compare only the first line and the exit code. Help and usage texts come from `root.go:13-36`, `delete.go:21-47`, `list.go:12-24` plus cobra's default template.

Errors that exit 1 (messages verbatim): `scan root is required: use --root or configure via Setup` (`list.go:40`, `delete.go:60,87`), `vault and root are required` (`sync.go:31`), `reading vault: <os error>` (`sync.go:86`), `scanning projects: <err>` (unreachable, the walker never returns an error), `accepts at most 1 arg(s), received 2`, unknown flag/command, `delete --project` removal failure, and Bubble Tea failures such as `could not open a new TTY: open /dev/tty: no such device or address` [P] (any TUI without a terminal).

## 2. Configuration

Library: viper (YAML). `cobra.OnInitialize(initConfig)` runs before every command (`root.go:45,61-84`).

**Precedence per key (high to low):** flag, environment, config file.
- Vault: `--vault` > `SKILL_MANAG_VAULT` > pointer file `~/.config/skill_Manag/vault`. The pointer file is read only when flag and env gave nothing (`root.go:67-77`): `os.UserHomeDir()` (`$HOME`), whole file `TrimSpace`d, read errors ignored, no `~` expansion, a relative value resolves against the cwd [P]. An empty env var counts as unset [P].
- Root, mandatory, exclude lists: `--root` > `SKILL_MANAG_ROOT` > `<vault>/config.yaml` [P]. `viper.AutomaticEnv` with prefix `SKILL_MANAG` also makes `SKILL_MANAG_EXCLUDE_DIRS`, `_EXCLUDE_PATHS`, `_MANDATORY` work, whitespace-separated [P for EXCLUDE_DIRS]. README documents only the first two.
- Config file is read only if a vault is known: `viper.SetConfigFile(<vault>/config.yaml)` then `ReadInConfig`, **errors ignored** (`root.go:80-83`). Missing or malformed YAML behaves like an empty file [P]. Keys are case-insensitive (viper lowercases).

**`<vault>/config.yaml` keys:** `root` (string), `mandatory` (list of skill names), `exclude_dirs` (list of directory basenames), `exclude_paths` (list; absolute or relative to `root`). Read in `sync.go:103-108` and `push.go:9-10`. Unknown keys are tolerated and survive rewrites.

**Who requires what:** root handler needs vault and root (else Setup, `sync.go:24-33`; if still empty after Setup: `vault and root are required`, exit 1 [P] for first-run quit). `list` and `delete` need only root and never launch Setup. `delete <name> --project P` needs nothing.

**Writes (only from the TUI):**
- Pointer file: `MkdirAll(~/.config/skill_Manag, 0755)`, then `vault + "\n"` mode 0644 (`setup.go:261-268`).
- `config.yaml` rewrite (`setup.go:269-276` for Setup, `281-288` for the Push edit overlay): fresh viper instance, `ReadInConfig` (error ignored), `Set("root", root)` and `Set("mandatory", list)` (Push edit sets only `mandatory`), `WriteConfigAs`. Result [P]: all keys lowercased and sorted alphabetically, 4-space indented block sequences, **comments dropped**, unknown keys kept, empty list written as `mandatory: []`, strings YAML would read as booleans quoted (`"y"`), new file mode 0644. **A malformed existing file is silently replaced** by just `mandatory` and `root`, losing exclusions [P].
- After Setup returns, `runMenu` only calls `viper.SetConfigFile` (`sync.go:70-72`); the new file is not re-read until Push calls `ReadInConfig` (`push.go:9`). So `exclude_*` stay stale for Sync/List/Delete in that session.
- `internal/config.go` (`LoadConfig`, reads `~/.config/skill_Manag/config.yaml` keys `source`, `root`) is dead code, never called.

## 3. Walker (`internal/walker.go`)

A **project** is the parent of an `.agents` directory that has a child directory named exactly `skills` (case-sensitive: `.AGENTS` and `Skills` are ignored [P]). The unit is a **Target** `{ProjectPath, SkillName, SkillPath = <project>/.agents/skills/<name>}` (`walker.go:39-43`).

Algorithm shared by all four finders (`walker.go:112-160,180-212,222-260`): `filepath.WalkDir(root)`, depth unlimited, lexical order (byte-wise sorted names per directory, depth first).
1. Callback errors (unreadable paths) are swallowed and the walk goes on (`116-119`). A nonexistent or non-directory root yields no error and no targets [P].
2. Any directory is pruned (`SkipDir`) if its basename is in the built-in set `.git node_modules vendor dist build out target .next .nuxt .venv __pycache__ .tox .pytest_cache .cache .turbo .parcel-cache` (`10-36`), or equals an `exclude_dirs` entry exactly (`59-63`), or lies within an `exclude_paths` entry (`64-68`, `72-86`). `exclude_paths` entries: empty ignored, cleaned, relative ones joined to `root`, match when the directory equals or is below the entry (component-wise, so `sub/other` does not match `sub/other2` [P]). Entry `.` excludes everything [P]. Hidden dirs other than the listed ones are walked (`.agents`, `.claude`, ...).
3. The prune test also runs on `root` itself, so a root whose basename is e.g. `build` scans nothing [P].
4. A directory named `skills` whose parent basename is `.agents` ends the descent into it (`SkipDir`, `156`); siblings and nested projects elsewhere are still walked [P], nested `.agents/skills` inside a skill are never seen [P].
5. Symlinks are never followed: a symlinked project dir, a symlinked `.agents/skills`, or a symlinked skill directory is invisible [P] (`WalkDir` and `DirEntry.IsDir` do not resolve links). Plain files in `skills/` are ignored.
6. If reading the `skills` dir fails, the callback returns nil, so the walk then descends into it (`135-138`).

The four finders differ only at the `skills` dir:
- `FindTargetsWithOptions(root, master, opts)`: one Target per child directory whose name is a key of `master` (the opt-in rule). Order: walk order, then name order.
- `FindTargetsByNameWithOptions(root, name, opts)`: same with `master = {name}`; used by `delete <name>`.
- `FindAllSkillTargetsWithOptions(root, opts)`: every child directory, hidden ones included; used by List and the Delete TUI.
- `FindPushTargetsWithOptions(root, pushSkills, opts)`: for every project, one Target per push skill whether or not the directory exists; the `skills` dir itself may be empty. **Per-project skill order is Go map order, random** [P] (`200`).

`ReadMasterSkills(vault)` (`89-103`): every child **directory** of the vault (no symlinks, hidden dirs included), as name to path. Consequence [P]: when the vault is a git repo, `.git` is a "skill". An unreadable vault returns the OS error.

## 4. Copier (`internal/copier.go`)

`SyncSkill(master, target, dryRun)` (`24-64`), per target, in this order:
1. `src = master[target.SkillName]` (an absent key gives `""`, see Q1).
2. File list: `gitListFiles(src)` runs `git -C <src> ls-files --cached .` (`72`). On success: output `TrimSpace`d, split on `\n`, empty lines skipped, `FromSlash`. This list is **tracked files only** (index content): untracked and ignored files are excluded, files in the index but missing from disk are included. Order is git's (byte-sorted path). If git is missing or `src` is not in a work tree, fall back to `walkVaultFiles` (`87-107`): lexical walk, directories in the skip set pruned (also when `src` itself has such a name), symlinks excluded, every other non-directory included (a *file* named `dist` is copied), hidden files included.
3. `Removed` = files under the destination that are not in the list (`110-136`, informational, never displayed). Computed also in dry run.
4. `dryRun`: return with `Files` and `Removed`, touching nothing (`43-45`).
5. `os.RemoveAll(SkillPath)`, then for each file `copyFile`. **First error aborts** and is returned in `Err`; there is no rollback, the destination stays missing or partial [P].

`copyFile` (`139-165`): `os.Stat(src)` (follows symlinks), `MkdirAll(dirname(dst), 0755)` (umask applies, so usually 0755; directory modes of the vault are not mirrored, empty directories are not recreated), open dst `O_WRONLY|O_CREATE|O_TRUNC` with the source permission bits, `io.Copy`, then `Chmod(dst, srcMode.Perm())` so the umask cannot weaken it. Only the 9 permission bits are mirrored (no setuid or sticky, no owner, no mtime) [P: 0444, 0600, 0755 preserved]. Symlinks tracked in git: file symlink becomes a regular file with the target's content [P]; directory symlink leaves an empty file and fails with `copy_file_range: is a directory` [P]; broken symlink fails at `Stat` [P].

Display text for a result uses only `len(Files)`.

## 5. Sync entry paths

`skill_Manag` without `--dry-run`: menu item Sync, `tui.RunSync` (section 8.3). With `--dry-run` (`sync.go:78-101,111-165`), no TUI and no writes:
1. `ReadMasterSkills(vault)`; error: `reading vault: <err>` (exit 1). Zero skills: prints `No skills found in vault.` (stdout, exit 0).
2. `FindTargetsWithOptions`. Zero targets: `No matching skills found in any project.` (exit 0).
3. Targets are grouped by project in a Go map and **iterated in random order** [P] (`112-120`); within a project, walk order.
4. Per project: `"\n" + "● <project>"` (bold). Per target a line on stdout: `"  ~ " + name padded to 30 + "would sync (<N> files)"`, i.e. `fmt "  %s %-30s %s\n"`. On `Err`: stderr `"  ✗ <name>  <err>"`, counted as an error, no stdout line.
5. Blank line, then summary: `<S> skills would be synced across <P> projects. (dry run)`, with ` <E> error(s).` appended when `E > 0` (then styled as warning, otherwise success). **Always exit 0**, even with errors. No singular forms: `1 files`, `1 skills`, `1 projects` [P].

Real-run counterparts (`"synced"`, `"<S> skills synced across ..."`) exist in `syncAll` (`136-161`) but are unreachable: `doSync` only calls `syncAll` with `dryRun=true`.

## 6. Delete and push

`DeleteSkill(target, dryRun)` (`deleter.go:13-21`): `os.RemoveAll(SkillPath)` unless dry run. Removes only the skill directory; leaves `.agents/skills` and the project. A missing path is success. A symlink is removed, not its target.

`delete <name>` without `--project` (`delete.go:85-111`): `root` required; `FindTargetsByNameWithOptions`; none found: `"\n" + "<name> not found in any project."` (exit 0). Else for each target (walk order) print the result line, then `"\n"` and `<N> project(s) updated.` or `<N> project(s) would be updated. (dry run)` (N = number of targets). Result line (`delete.go:115-137`), success: `"  ✗ <name padded to 30> deleted from <ProjectPath>"` (the `✗` is red even on success) or `"  ~ <name padded 30> would delete from <ProjectPath>"` for dry run, stdout. Error: stderr `"  ✗ <name padded 30> <err>"`; the loop continues and the exit code stays 0 (code reading).

`delete <name> --project P` (`73-82`): builds `Target{P, name, P/.agents/skills/<name>}` directly: no scan, no existence check, no `root`, no summary line, prints one result line, returns the removal error (exit 1) if any. A nonexistent path prints `deleted from` anyway [P]. The name is not validated (Q5).

Push (TUI only, `tui/push.go:65-94`): `master = ReadMasterSkills(vault)`; push set = mandatory names that exist in `master` (unknown names silently dropped); empty set: no scan; else `FindPushTargetsWithOptions`. Each Target is run through the same `SyncSkill` with `dryRun` hard-coded false (`125`). Created if the skill directory does not exist, fully replaced (delete then copy) if it does [P]. Projects need an existing real `.agents/skills` directory (an `.agents` without `skills` is skipped [P]).

## 7. Exact output strings (CLI)

Colors apply only when stdout is a terminal; piped output has no ANSI [P] (lipgloss auto-detects; on a TTY the binary also issues terminal queries, `ESC ]11;? ESC \` and `ESC [6n` [P]). Palette (`styles/styles.go:9-14`): Success `#00FF87` bold, Warning `#FFDB58`, Error `#FF5F5F` bold, Muted `#626262`, Header bold, SkillName `#87CEEB`. Parity tests should compare piped output.

| Where | stream | text |
|---|---|---|
| dry run, no vault skills | out | `No skills found in vault.` |
| dry run, no targets | out | `No matching skills found in any project.` |
| dry run, project header | out | blank line, `● <path>` |
| dry run, skill | out | `  ~ <name %-30s> would sync (<N> files)` |
| dry run, skill error | err | `  ✗ <name>  <error>` |
| dry run, summary | out | `<S> skills would be synced across <P> projects. (dry run)` + optional ` <E> error(s).` |
| delete, not found | out | blank line, `<name> not found in any project.` |
| delete, success | out | `  ✗ <name %-30s> deleted from <project>` |
| delete, dry run | out | `  ~ <name %-30s> would delete from <project>` |
| delete, error | err | `  ✗ <name %-30s> <error>` |
| delete, summary | out | blank line, `<N> project(s) updated.` / `<N> project(s) would be updated. (dry run)` |
| Setup save failed | err | `warning: could not save config: <err>` (exit stays 0) |

Padding detail: `%-30s` pads the *styled* string, so on a TTY the escape bytes eat the padding and columns collapse, piped output is aligned [P] (Q9). Project paths print as the walker produced them: as given for a relative root (`rootB/proj`), absolute otherwise [P].

## 8. TUI

### 8.1 Frame, keys, mouse (all screens)

Every screen is its own Bubble Tea program with alt screen and `WithMouseAllMotion` (`menu.go:164`, `sync.go:286`, ...). `Setup` additionally renders to **stderr** (`setup.go:240`), all others to stdout. When a screen's program ends, control returns to the menu loop (`sync.go:41-74`), which creates a fresh menu with the cursor on Sync. Quitting the menu (`q`, `ctrl+c`) exits 0.

Layout: `View() = "\n" + AppHeader + "\n\n" + body`, so row 0 blank, row 1 header, row 2 blank, body from row 3. Header (`styles.go:43-70`): menu `   skill_Manag  ·  javedab.com`; screens ` ← skill_Manag  ·  javedab.com  ·  <screen>` with screen in `sync list delete push setup`. `javedab.com` is an OSC 8 hyperlink to `https://javedab.com`, muted underlined, `#87CEEB` on hover. Header hit boxes (`styles.go:19-37`, `tui/header.go:11-32`): link row 1, cols 19..29; back arrow row 1, col 1 (child screens only). Hover state updates on motion and press; left press on the link runs `xdg-open` / `open` / `cmd /c start` detached (`browser.go`); left press on `←` quits the screen (in Push, closes the edit overlay first).

Key bindings (`keys.go`), identical for Sync, Delete, Push, List unless noted:

| Action | Keys | Help text |
|---|---|---|
| up / down | `up` `k` / `down` `j` | `↑/k up`, `↓/j down` |
| toggle | `space` | `space toggle` |
| all | `a` | `a all` |
| confirm | `enter` | `enter confirm` (Push: `enter push`) |
| edit mandatory | `e` (Push only) | `e edit mandatory` |
| filter / sync / delete | `/` / `s` / `d` (List only) | `/ filter`, `s sync`, `d delete` |
| help | `?` | `? help`, toggles the full help |
| quit (back) | `q` `ctrl+c` `alt+left` | `q/alt+← back` |

Short help line: items joined by ` • ` (`space toggle • a all • enter confirm • ? help • q/alt+← back`; List: `/ filter • space toggle • s sync • d delete • ? help • q/alt+← back`; Push adds `e edit mandatory` before `?`). Full help is a grid with columns `{up,down}`, `{toggle,all}`, `{confirm,quit}` (Push `{confirm,edit,quit}`, List `{toggle,all}`, `{filter,sync,delete}`, `{help,quit}`).

Shared mechanics:
- Phases (`shared.go:10-16`): loading, select, syncing, confirm, done. **Loading**: only `ctrl+c` works, text `  <spinner>  Scanning projects…` (spinner `Points`, `#87CEEB`). **Done**: any key quits.
- Selection tables show 10 rows per page (`pageSize`, `shared.go:18`). Page indicator is bubbles `paginator` in dots mode, drawn on its own line after a blank line only when more than one page: `•·` (active dot styled, inactive `·` muted). `up` at row 0 of a page goes to the previous page's last row (cursor 9), `down` at the last row goes to the next page's row 0; no wraparound.
- `a`: if every item is selected, deselect all, else select all (across all pages).
- Enter with nothing selected does nothing, no message.
- Mouse: motion over a row moves the cursor to it; left press on a row moves the cursor and toggles it; wheel and other buttons do nothing; rows start at screen row 6 (Sync/Delete/Push/List without filter), 7 for List when a filter is non-empty (`list.go:113-116`), row 5 in the Push edit overlay, row 10 in Setup mandatory step (`setup.go:79-97`).
- Table geometry (Sync/Delete/Push): row 3 `<Title>   <n> / <total> selected`, row 4 `  [·] ` + `%-22s  %s` of `skill`, `projects`, row 5 `  ` + 52 × `─`, then rows `<cursor><box> <name %-22s>  <count>` where cursor is `> ` or two spaces, box `[✓]` or `[ ]` (Delete uses `[✗]`), count `<N> project` / `<N> projects`. Name padding has the same ANSI problem as Q9.
- Colors per screen (cursor, checkbox, title): Sync cursor and box Success with Header-bold title; Delete Error throughout; Push Warning throughout; List cursor and box SkillName with Header-bold title. Active paginator dot: Sync and List SkillName, Delete Error, Push Warning. Progress bar (Sync and Push only, `sync.go:263-273`, `push.go:483-487`): gradient `#626262` to `#87CEEB` (Push to `#FFDB58`), width 40, no percentage.

### 8.2 Main menu (`menu.go`)

List of five items, selected row has a left bar `│` in `#87CEEB`; tagline under the header: `Sync Claude Code skills across all your projects from a single vault.` (muted, 2 spaces indent). Keys: `up/down/j/k` (bubbles list defaults), `enter` picks, `q`/`ctrl+c` quit, footer `↑/k up • ↓/j down • q quit • ? more`. Mouse: list top at row 4, 3 rows per item (`(Y-4)/3`); motion selects, left press picks. Below the list: a divider of `w-4` × `─` and the selected item's detail text wrapped to `w-4` (default width 72 when unknown). Window resize sets list height to `height-11`.

| Item | Description | Detail text |
|---|---|---|
| Sync | `Updates only skills already installed per project — vault never force-pushes` | `Walks every project under your root and updates skills they already have installed — pulling the latest from your vault. The opt-in rule: if a project doesn't have a skill, it will inshallah never be added. Only what's already there gets refreshed.` |
| List | `Full table view — filter by name, sync or delete selected rows in place` | `A searchable table of every skill installed across all your projects. Filter by name with /, select rows with space, then sync or delete the selection directly without leaving the screen.` |
| Delete | `Nothing pre-selected — pick explicitly and confirm before anything is removed` | `Remove skills from projects. Nothing is pre-selected — you pick explicitly, then confirm before anything is removed. Supports removing from all matching projects at once.` |
| Push | `Force-installs mandatory skills to every opted-in project` | `Reads the mandatory list from your vault config and pushes those skills to every project that already has .agents/skills/ — bypassing the opt-in rule. Configure mandatory skills by adding 'mandatory: [skill-name]' to <vault>/config.yaml.` |
| Setup | `Filesystem picker for vault and root — auto-runs on first launch` | `Configure your vault (the folder holding your master skill files) and root (the folder containing all your projects) using a filesystem picker. Settings are saved to ~/.config/skill_Manag/config.yaml.` (stale, Q24) |

### 8.3 Sync screen (`sync.go`, `sync_view.go`)

Scan (`60-72`): `ReadMasterSkills`, `FindTargetsWithOptions`, group targets by skill name, groups sorted by name (byte order), all pre-selected. Zero groups, including an empty vault, goes to Done and prints `No matching skills found in any project.` (Muted); a scan error prints `✗ <err>`.
- Select view title `Select skills to sync`, text keys as 8.1.
- `enter` keeps only selected groups, phase Syncing. Each tick synchronously runs `SyncSkill(master, t, false)` for **all targets of one skill group** (UI blocks meanwhile), progress bar set to `done/total`. Label `  Syncing  <i> / <n>` where `i = doneCount+1`, so it shows the group about to be processed. When the last group finishes the phase flips straight to Done.
- Results (Done): `Results`, blank line, then per skill in processed order: `<icon> <name %-22s> <verb> <N> project(s) (<F> files)` with verb `synced to`, icon `✓` (green), `✗` red if every target failed, and `  <E> error(s)` appended in red when any failed; `N` counts successes, `F` sums `len(Files)` of successes. Under it one line per target in walk order: success `  ✓ updated  <ProjectPath>`, failure `  ✗ <ProjectPath>  <err>`. Finally blank line and `q/alt+← back`. Dry-run variants (`would sync to`, `~`, `would update`) are unreachable from the CLI.
- Keys while Syncing are not ignored (Q21).

### 8.4 List screen (`list.go`)

Scan: if `vault` is non-empty `ReadMasterSkills` (error aborts), then `FindAllSkillTargetsWithOptions` (every installed skill, even ones absent from the vault); no grouping, one row per target, in walk order. Title `Skills — <N> installed`, with an active filter `Skills — <M> matching "<filter>"` (Go `%q` quoting). Columns `skill`, `project` (`%-22s  %s`), divider 54 × `─`, project shown as `shortPath` (last two path components joined by `/`, `shared.go:21-28`). Empty result still shows the empty table; no message. Selection is a set of indices into the full list, kept when the filter changes.
- Filter mode (`/`): line `filter: <text>▌` (Warning); `esc` clears and leaves, `enter` keeps and leaves (line becomes `filter: <text>  esc to clear`, muted), `backspace` deletes a byte, `ctrl+c` quits, any key whose string is one byte is appended (so `q`, space, digits are text; pasted or non-ASCII input is dropped). Match: case-insensitive substring on skill name only. Every filter change resets cursor and page.
- Browse: `esc` clears the filter; `a` toggles only the filtered rows; `s` requires a vault else Done with `✗ vault not configured — run Setup to set a vault path`; otherwise `SyncSkill` for every selected index (including rows hidden by the filter), in Go map order; `d` deletes every selected index with `DeleteSkill` and **no confirmation**. Both end in Done and the list is not re-scanned.
- Result lines (`Results`, blank line, messages, blank line, `q/alt+← back`): sync ok `✓ <name>  synced → <shortPath>`, delete ok `✗ <name>  deleted from <shortPath>` (✗ yellow), failure `✗ <name>  <err>`. With nothing selected the Results page is empty.

### 8.5 Delete screen (`tui/delete.go`)

Same scan as List but grouped by skill name, sorted, **nothing selected**; title `Select skills to delete`. Zero groups: Done with `No skills found in any project.`. `enter` with at least one selection enters Confirm:

```
Delete <k> skill(s)?

  This will inshallah permanently remove the selected skills from all matching projects.

  y / enter  confirm   n / esc  cancel
```
In Confirm: `y` `Y` `enter` delete all targets of selected groups (`DeleteSkill(t, dryRun)`), `n` `N` `esc` go back, `ctrl+c` quits, `q` does nothing. Results (`Delete results`): per skill `<icon> <name %-22s> <verb> <N> project(s)` (+ `  <E> error(s)` in red), verb `deleted from` (icon yellow `✗`) or `would delete from` (`~`, only with `skill_Manag delete --dry-run`, which opens this screen with `dryRun` set; the confirm text does not change); no per-project lines. Skill order is the group (name) order.

### 8.6 Push screen (`tui/push.go`)

Scan (section 6), groups by skill name, all pre-selected, title `Select skills to push`; `N projects` counts targets. Zero groups: Done with `No mandatory skills configured in vault config.` when `mandatory` is empty, else `No opted-in projects found.` (also shown when no mandatory name exists in the vault). `enter` (select phase only) runs the groups like Sync with label `  Pushing  <i> / <n>`. Results `Push results`: `<icon> <name %-22s> pushed to <N> project(s) (<F> files)` (+ red `  <E> error(s)`), no per-project lines.
- Edit overlay (`e`, select phase only): title `Edit mandatory skills`, divider 52 × `─`, one row per vault skill (sorted, `.git` included [P]) `<cursor><box> <name>` with box `[✓]` for mandatory, footer `  ↑/↓ navigate  space toggle  enter save  esc/q cancel`. `up/k down/j space` as usual; `q` `alt+left` `esc` close the overlay; `ctrl+c` quits the screen; `enter` writes `mandatory` via the rewrite in section 2 (write failure: Done with `✗ <err>`), then rescans from the Loading phase.

### 8.7 Setup wizard (`tui/setup.go`)

Runs from the menu (Setup) and automatically when vault or root is empty. Phases: vault, root, mandatory, save, done (`19-25`).
1. **Vault**: text `→ Vault`, `Your skill collection — the one folder where you` / `write and maintain skills. Select a directory:`, then the bubbles filepicker (directories only, hidden hidden, starts in the existing vault, else `$HOME`).
2. **Root**: `✓ Vault  <path>`, `→ Root`, `The folder that contains all your projects.` / `skill_Manag walks it to find installed skills:`, picker starting in the existing root, else `$HOME`.
3. **Mandatory**: `✓ Vault`, `✓ Root   <path>`, `→ Mandatory`, `Skills pushed to every opted-in project:`, checklist of `readVaultSkillNames(vault)` (sorted vault directories, `.git` included [P]) with boxes pre-set from the **global** viper `mandatory` (old vault's config, `167`), empty vault shows `(no skills found in vault)`. Hint `  ↑/↓ navigate  space toggle  enter confirm`. Keys `up/k down/j space enter`; mouse as in 8.1 (hover moves, left press toggles).
4. **Save**: three lines `✓ Vault      <p>`, `✓ Root       <p>`, `✓ Mandatory  <a, b | (none)>`, divider (56 × `─`), then `  Save config?  vault pointer → ~/.config/skill_Manag/vault   root → <vault>/config.yaml   (y / n)`.
Footer on all steps: `  q  ·  alt+←  ·  ctrl+c   back`. `y`/`Y` saves, `n`/`N` skips (only in Save). `q`, `alt+left`, `ctrl+c`, and a click on `←` leave at once. Picker keys (library defaults): `j/down`, `k/up`, `g`, `G`, `J/pgdown`, `K/pgup`, `h/left/backspace/esc` parent directory, `l/right` open, `enter` select (the highlighted entry; it also descends). `enter` on a highlighted directory sets it as vault or root and advances; there is no way to select the directory the picker is currently in. The picker consumes the window height minus 5, on top of the header, so the first lines scroll off the top of the screen on the vault and root steps [P].

`RunSetup` (`238-254`) returns the **model's** vault and root even if nothing was saved (cancel, `n`), and saves (pointer then config) only after `y`; a save error is printed to stderr and the call still succeeds.

## 9. Quirks and defects

Tags: KEEP, CHANGE, UNCLEAR. IDs are referenced above.

| ID | Tag | What happens | Cite |
|---|---|---|---|
| Q1 | CHANGE | List `s` on a skill that is not in the vault passes `srcDir = ""`; `git -C "" ls-files` then lists the **process cwd's** tracked files. Dest is deleted and cwd files are copied over [P: 396 files, then an error on a tracked-but-deleted file]. Outside a git cwd it only errors with `lstat : no such file or directory` and the destination survives [P]. Guard: skip targets absent from master. | `copier.go:25,72`, `list.go:221-227` |
| Q2 | CHANGE | `git ls-files` quotes names with non-ASCII or control bytes (`"Gr\303\274\303\237e.md"`); the literal quoted string is then used as a path, `Stat` fails after `RemoveAll`, destination is wiped [P]. Use `ls-files -z`. | `copier.go:72-83` |
| Q3 | CHANGE | Delete-then-copy is not atomic: any copy error (missing tracked file, broken symlink, directory symlink, permission, disk full) leaves the destination missing or half written [P]. Stage and swap, or validate first. | `copier.go:48-61` |
| Q4 | CHANGE | A vault skill dir inside a git repo with zero tracked files (new, all untracked, or all ignored) gives an empty list with ok=true: destination is deleted, nothing copied, reported as success with `0 files` [P]. | `copier.go:29,77-82` |
| Q5 | CHANGE | `delete <name> --project P` joins an unvalidated name: `..` removes `P/.agents`, `coding/../../..` removes `P` itself [P]. Reject names with separators or dots. | `delete.go:74-78` |
| Q6 | CHANGE | Any vault child directory is a skill, so `.git` shows in Setup mandatory and the Push edit overlay [P]; as a push target it would delete and recreate nothing useful. Skip `.git` (decide on other dot-dirs). | `walker.go:96-99` |
| Q7 | CHANGE | Cancelling Setup after the vault step, or answering `n`, still hands the new vault/root to the running session, so Sync then scans the wrong directory [P: vault became `<vault>/coding`, `No matching skills found`]. | `setup.go:244-253`, `sync.go:64-72` |
| Q8 | CHANGE | Picker starts inside the existing vault/root, and `enter` picks a *child*, so keeping the current vault means navigating up first; header scrolls off [P]. | `setup.go:47-51,152-171` |
| Q9 | CHANGE | `%-22s` / `%-30s` applied to styled text: aligned when piped, collapsed on a TTY [P]. Pad by display width; piped output must stay byte-identical. | `sync.go:143`, `sync_view.go:106`, others |
| Q10 | CHANGE | Random order: project groups in `--dry-run`, per-project skills in push targets, selected rows in List `s`/`d` [P]. Use walk order (sorted). | `sync.go:112-120`, `walker.go:200`, `list.go:237,260` |
| Q11 | CHANGE | Prune list is applied to `root` itself: root named `build`, `out`, `target`, `dist`, `vendor`, `.cache`... scans nothing, silently [P]. | `walker.go:56,122` |
| Q12 | UNCLEAR | Missing or non-directory `--root` is not an error: `No matching skills found in any project.`, exit 0 [P]. Tests may rely on it. | `walker.go:116-119` |
| Q13 | CHANGE | Absolute `exclude_paths` never match when root is relative (`filepath.Rel` error ignored) [P]. Resolve both sides to absolute. | `walker.go:81-84` |
| Q14 | KEEP | Walker never follows symlinks (project, `.agents/skills`, skill dir); avoids loops and escapes [P]. | `walker.go:115-157` |
| Q15 | CHANGE | `delete <n> --project P` reports `deleted from` for a nonexistent skill, exit 0 [P]. | `delete.go:73-82` |
| Q16 | CHANGE | `delete <name>` and `--dry-run` sync exit 0 although some targets failed (summary still counts them as updated). | `delete.go:99-109`, `sync.go:157-164` |
| Q17 | KEEP | No pluralization: `1 files`, `1 skills ... 1 projects` [P]; keep for parity. | `sync.go:146,153` |
| Q18 | CHANGE | List selection survives filter changes; `s`/`d` act on rows no longer visible, and `d` has no confirmation (Delete screen has). | `list.go:200-232,259-280` |
| Q19 | CHANGE | List `s`/`d` with nothing selected opens an empty Results page. | `list.go:221-232` |
| Q20 | CHANGE | Push edit overlay is unreachable when no mandatory skill is configured or no target exists (screen is already Done, any key quits), so first-time configuration only works in Setup; message `No opted-in projects found.` is misleading when names are unknown to the vault. | `push.go:105-108,297,420-423` |
| Q21 | CHANGE | Sync `enter` is not guarded by phase: `enter` while Syncing restarts the index and duplicates results; `space`, arrows and `a` also act while Syncing (possible index panic with more than 10 groups); `q` aborts mid-run. Push guards `enter` and `e`. | `sync.go:179-237` |
| Q22 | CHANGE | Results are plain text without scrolling; long runs (more than the terminal height) scroll the top away [P]. | `sync_view.go:58-74` |
| Q23 | UNCLEAR | `Removed` (stale file list) is computed and never shown. | `copier.go:41` |
| Q24 | CHANGE | Stale text: menu Setup detail names `~/.config/skill_Manag/config.yaml`; README says Go 1.24+ (go.mod says `go 1.25.0`) and that `.gitignore` decides (it is the git index); `LoadConfig` dead; unreachable dry-run branches in the Sync TUI and non-dry `syncAll`. | `menu.go:66`, `config.go`, `sync.go:136-161` |
| Q25 | CHANGE | YAML rewrite loses comments, reorders and lowercases keys, and overwrites a malformed file with two keys [P]. | `setup.go:269-276,281-288` |
| Q26 | UNCLEAR | Symlink handling differs between the git path (dereferences file links, fails on directory links) and the fallback walk (skips all links). | `copier.go:96,139-165` |
| Q27 | UNCLEAR | Filter keeps only one-byte keys; pasted or non-ASCII text is ignored, `backspace` slices bytes. | `list.go:160-172` |
| Q28 | UNCLEAR | Setup preselects mandatory from the globally loaded config (the previously configured vault), not the newly chosen one. | `setup.go:167` |
| Q29 | UNCLEAR | Non-git fallback prunes by directory name only, so a *file* named `dist` is copied while a skill whose own name is in the prune set (e.g. `build`) yields zero files and wipes the destination (read, not probed). | `copier.go:93` |
| Q30 | UNCLEAR | Setup renders to stderr, other TUIs to stdout; redirecting stdout breaks all but Setup. | `setup.go:240` |
| Q31 | CHANGE | `--root` that is a symlink: Go's `WalkDir` finds nothing without a trailing slash and everything with one [P]. Rust resolves the root (canonical path), so both spellings find the same projects, and `exclude_paths` match the real path whether the root contains `..` or links. | `walker.go:115` |
| Q32 | CHANGE | `config.yaml` values that cannot mean anything are hard errors in Rust, even when a flag overrides the key: an empty `root:`, an empty `exclude_dirs` or `exclude_paths` entry, and an `exclude_paths` entry that does not exist on disk. Go ignores all of them silently, so a typo excluded nothing. A config file is checked whenever it is read, never only when its value wins. | `walker.go:72-86` |
| Q33 | CHANGE | A skill folder reached through a symlinked `.agents` or `.agents/skills` is never written or removed (`delete --project`, plan, apply). Go's `RemoveAll` followed the link and deleted the shared folder outside the project [P]. A symlinked skill folder itself may still be removed with `delete --project` (the link goes, nothing behind it). | `delete.go:74-78` |
| Q34 | CHANGE | Backups are on by default and have no Go counterpart. Every `sync`, `push`, `add`, `init` and `delete` that replaces or removes a skill folder first keeps the old copy in `$XDG_STATE_HOME/skillmirror/backups/<run-id>/<n>/tree/`, next to a JSON note `entry.json` (command, project, skill, what the run did). A skill the run created is noted without a tree, so `undo` removes it. If the old copy cannot be stored, that target fails and the old copy stays in place (hard error, no half state); the other targets continue. A symlinked skill folder is not stored, only the link is removed (Q33). A run that stored nothing leaves no folder; after a run that stored something the newest 30 runs are kept. `skillmirror undo [RUN]` brings a run back (`--project`, `--skill`, `--dry-run`, confirmation unless `--yes`, exit 2 without a terminal and without `--yes`, exit 4 when some entries fail) and is itself a run, so a second `undo` redoes it. `skillmirror history` lists the runs, newest first. The `--json` documents of `sync`, `push`, `add`, `init` and `delete` carry the run id in `backup` (null when nothing was kept). | new in Rust: `Crates/Core/src/backup/` |
| Q35 | CHANGE | Profiles (new in Rust, no Go counterpart): the skills of a profile are what its parents select, plus its `groups` and `skills`, minus its own `exclude`. An `exclude` only touches what its own profile selects, so the order of `extends` never changes the result and a profile can add back a skill that a parent left out. A skill named in `exclude` must exist in the vault. Each profile is worked out once, so a deep diamond of parents costs time in proportion to its size, not to the number of paths. | new in Rust: `Crates/Core/src/ops/select.rs` |
| Q36 | CHANGE | The TUI never writes on a single Enter for sync and push (Go did, Q21). It works out the plan in a job, shows a confirmation page that counts folders and files and names every file that would be removed, and applies exactly that plan, so a folder edited after the plan was made fails with `DestinationChanged` instead of being overwritten. A plan that would write nothing goes straight through. Job reports carry a job id and are heard only while the screen waits for that job; the header arrow, a second job and the first Ctrl-C are refused while a job writes (the second Ctrl-C quits). A dead keyboard ends the interface with exit 3. | `Crates/Tui/src/app/`, `jobs.rs` |
| Q37 | CHANGE | Selection and filter (Q18): a selection survives filter changes, but Enter, `s` and `d` act only on the selected rows that are visible; the header says how many selected rows the filter hides. | `Crates/Tui/src/screens/work.rs` |
| Q38 | KEEP | No `fsync`: a killed process leaves no half skill, a power loss may. Skills are git-tracked copies and the next sync restores them, and one `fsync` per file over hundreds of targets costs seconds. | `Crates/Core/src/apply/run.rs` |
| Q39 | CHANGE | A leftover is a warning when the goal was reached and a failure when it was not: an update whose old copy cannot be removed is `Updated` with a leftover warning, a delete whose remains in `.trash-*` cannot be removed is a failed row (exit 4) because the removal is incomplete. | `Crates/Core/src/apply/place.rs`, `ops/delete.rs` |
| Q40 | CHANGE | A project folder whose name is not valid UTF-8 is a normal project. The backup note keeps its exact bytes; text output and the `--json` documents show it with replacement characters, so no command fails after it already wrote. Skill folders with such names are still reported and skipped. | `Crates/Core/src/backup/pathjson.rs`, `Crates/Cli/src/output/lossy.rs` |
| Q41 | CHANGE | `skillmirror status [--all] [--json]` (new in Rust, no Go counterpart) compares every project that has a skills directory with the vault and writes nothing, not even a backup folder. Per project: skills up to date, skills that differ (files added, changed, removed), mandatory skills the project lacks, installed folders the vault has no skill for (informational, not drift), and comparisons that failed with their reason. Exit codes like `sync --check`: 4 when a comparison failed, 1 when `sync` or `push` would change something, else 0; an unknown mandatory name is a hard error (3) as for `push`. Without `--all` the projects that are fully in sync are not listed. It cannot say who changed a file, only that it differs. | new in Rust: `Crates/Core/src/ops/status.rs` |
| Q42 | CHANGE | `skillmirror diff [SKILL] [--project DIR] [--stat] [--json]` (new in Rust, no Go counterpart) shows what `sync` would do to each installed skill that the vault also has, as unified diffs (`similar`, 3 lines of context): the old side is the project copy, the new side the vault, so `-` is what the project would lose. Added, modified and removed files are shown with their lines; a mode change shows the old and new bits; a binary or unreadable file, or one above 1 MiB, is listed without lines. A removed folder is shown through its files. `--stat` lists line counts per file. It writes nothing. A `SKILL` the vault does not have is a hard error (3); exit 1 when something differs, 4 when a folder could not be compared, else 0. Colour only on a terminal. | new in Rust: `Crates/Core/src/ops/diff.rs` |
| Q43 | CHANGE | `skillmirror doctor [--json]` (new in Rust, no Go counterpart) checks, in layers and without writing: git (starts), the vault pointer (points at a folder), the old tool's pointer (note), the backup store (damaged runs, size above 1 GiB), the configuration (a broken config or a legacy `SKILL_MANAG_*` variable is a finding and ends the layers that need it), the vault (discovery, links and folders without a skill, git), every skill's `SKILL.md` header (starts with `---`, is closed, is valid YAML, `name` equals the folder, `description` present; an unquoted `: ` in a description is invalid YAML), edits git does not know about (untracked files are not copied, uncommitted edits are already copied, a tracked file missing from the disk makes the skill fail, an untracked `SKILL.md` makes sync refuse it), the mandatory list and the profiles, the scan (unreadable folders, leftovers of interrupted runs unless their process still exists) and the scan root's filesystem (NFS, CIFS/SMB, FUSE and FAT cannot swap a folder atomically). Each finding is a note, a warning or an error with the check, the subject, the message and a hint. Exit codes: 0 nothing but notes, 1 warnings, 3 errors. | new in Rust: `Crates/Core/src/ops/doctor/` |
| Q44 | CHANGE | `targets: [claude]` in the vault `config.yaml` names other agent folders that must see the same skills (known names: `claude` for `.claude/skills`; an unknown or repeated name is a config error, exit 3). `skillmirror bridge [--dry-run] [--yes] [--all] [--json]` (new in Rust, no Go counterpart) makes, in every scanned project, `<project>/.claude/skills` as a relative link to `../.agents/skills`, and only where nothing is: a real folder or file, a link that leads elsewhere (an absolute link to the same folder counts as in place) and a `.claude` that is itself a link are listed with a hint and left alone, the state is looked at again right before the link is made, and the link is never made in a project without `.agents/skills`. It asks once unless `--yes` or `--dry-run`; without a terminal it refuses (2). Exit codes: 0 done or nothing to do, 1 `--dry-run` with links to make, 4 something is in the way or a link failed. `add` and `init` make the missing links of their own project after a write that created or updated skill folders (never on `--dry-run`, never when the user declined, never when nothing was written); a blocked link is reported in their output and `--json` document (`bridges`, left out when empty) and does not change their exit code. `status` counts a missing link as drift (`missing_bridges`, exit 1) and a blocked one as a problem (4); `doctor` warns about both. This differs from the ADR line "sync recreates a missing bridge": `sync` keeps to updating installed skills (its plan, `--check` and parity with Go stay as they were), and a vanished link is one `bridge` away. | new in Rust: `Crates/Core/src/ops/bridge.rs`, `Crates/Cli/src/commands/bridge.rs` |
| Q45 | CHANGE | While `sync`, `push`, `status`, `diff`, `bridge`, `list` and `delete` scan the root, a spinner with the count of folders looked at and of projects found is drawn on stderr (`indicatif`), updated once per 256 folders, and taken away before the first line of output. It is drawn only when stderr is a terminal and `TERM` is not `dumb`, and never when the command is given `--json`; a pipe or a file gets no byte on stderr from it. A scan under 256 folders shows at most the spinner after 150 ms. The Go tool printed nothing while scanning. | new in Rust: `Crates/Cli/src/output/progress.rs`, `scan_with_progress` in `Crates/Core/src/scan/walk.rs` |
| Q46 | CHANGE | The selection pages of the interface (sync, push, delete, list) show `N scan problems (i)` in the heading when the scan or the list of installed folders could not read something (unreadable folders, leftovers of interrupted runs). `i` opens a box with the first five (path and reason) and the count of the rest; the next key or click, whatever it is, closes the box and does nothing else. The Go interface dropped these problems. | new in Rust: `Crates/Tui/src/ui/issues.rs`, `session.rs` |
| Q47 | CHANGE | The interface has a History page (menu entry after Push). It reads the backup store in a job and lists the runs, newest first, with the date, the command, the number of skill folders and of projects; a run whose notes cannot be read is listed with the reason and cannot be picked. Enter, `u` or a second click on the picked row checks the whole run against the disk in a job (`undo --dry-run`) and asks, naming up to six folders; yes undoes it in a job with progress, the results page lists every folder and the run that saved what was replaced (undoing that run is a redo). An undo in which nothing can be put right goes straight to the results page, without a question. A page left before its job answers drops the answer, the header arrow and keys do not leave a running undo, and the first ctrl+c during it only warns. The delete question no longer says "permanently": a copy is kept first. Dialog text keeps its indentation. The menu shows one line per entry when two lines each would leave no room for the long text. | new in Rust: `Crates/Tui/src/screens/history.rs`, `ui/history.rs`, `undo.rs` |
| Q48 | CHANGE | The interface has Add and Init entries (after Push). Add: a folder picker (it starts in the scan root) for the project, then the vault skills as rows (group in the second column, `already in the project` marked, `mandatory` marked, none ticked); Enter works out the plan in a job (a missing `.agents/skills` is created when it is applied), asks, and installs in a job. Init: the picker for the parent folder, a name field (one folder name; `q`, `?` and other keys are letters there), then the rows with the mandatory skills ticked; `g` makes the new project a git repository. A folder that does not suit (a link in `.agents`, a project that is not empty) is refused on the page with the same words as `add` and `init`. Nothing is made before the question is answered; init makes the folder and the repository right before the first write and takes them back when no skill was installed. Both keep a backup run (command `add` or `init`), make the links the vault config asks for (contract Q44) after a write and name them on the results page, and a blocked link is a warning there. A folder check or plan answered after the user stepped back is dropped. | new in Rust: `Crates/Tui/src/screens/place.rs`, `jobs_install.rs`, `ui/place.rs` |
| Q49 | CHANGE | The question before a sync, push, add or init has a changes page behind `v`: the files are read in a job and shown as the unified diff of the plan that the question is about (skill and project as headings, each file with its kind and line counts, hunks, `-` taken from the project and `+` coming from the vault, binary, oversize and unreadable files named without lines, at most 5000 lines), scrollable with the arrows, page keys and the wheel. `esc`, `q`, `v` or the header arrow return to the same question with the same plan, so `y` applies exactly what was shown. A delete has no plan and no changes page. Leaving while the files are read drops the answer. The command line `diff` is the same view for the installed skills (contract Q42). | new in Rust: `Crates/Tui/src/diffview.rs`, `ui/diff.rs`, `ops::diff_of_plan` |
| Q50 | CHANGE | `skillmirror report [-o FILE] [--open]` (new in Rust, no Go counterpart) writes one self-contained HTML page (default `skillmirror-report.html` in the current directory; `-o -` prints it): a matrix of skills against projects (up to date, outdated, mandatory missing, not in the vault, could not be compared, not installed; a cell links to the card or the diff), the vault as a tree of groups with the description from each `SKILL.md`, a card for every skill (group, description or why the header cannot be read, tracked files, the state in each project, and for an outdated copy a coloured unified diff of at most 400 lines per file), and what needs a look (scan problems, links in the way or missing). A filter box narrows rows, tree and cards as you type, a button switches light and dark (the system setting by default), and an anchor to a diff opens it. Projects are named below the scan root. The page holds no outside reference and its Content-Security-Policy forbids any request; every value is escaped; there is no JSON blob. The file is written next to its place and renamed over it with mode 0600 (diffs can hold private text), and an existing file is replaced only when it carries this tool's marker comment after the doctype, else the command refuses (3) with a hint. A missing directory or an unknown mandatory name is a hard error (3), nothing is written into the vault or the projects, `--open` starts `xdg-open` and ignores its failure. | new in Rust: `Crates/Web`, `ops::report_data`, `Crates/Cli/src/commands/report.rs` |
