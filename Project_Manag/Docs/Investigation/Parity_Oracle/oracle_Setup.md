# Parity oracle: Setup

How to get from a fresh clone of the branch to a green parity run, how the oracle is built from the frozen Go commit `c7310f9`, how its 131 scenarios work, and what to keep when the Go code is deleted at cutover. The scripts live in `Code/Development/Parity/` (see its `README.md` for one-line commands); this file is the explanation behind them. The oracle is the Go tool recorded on throw-away fixtures: a `golden/` tree of stdout, stderr, exit code and a full manifest of every project tree after each step. The Rust binary is replayed through the same fixtures and classified against it.

## What is committed and what is not

| committed | not committed (rebuilt on demand, all under `Scratch/Oracle/`, which is gitignored) |
|---|---|
| `Code/Development/Parity/`: scenarios, harness, driver source (`oracle_drv.go.txt`), compare script | `src/`: detached git worktree at `c7310f9` (fixtures and Go source) |
| `divergences.tsv` and `divergences_Explained.md` in this folder | `skill_go`, `skill_go_drv`: the two Go binaries built from it |
| `parity_Report.md` here: condensed last result | `golden/`: the recorded Go outputs (11 MB, 1030 files) |
| `Project_Manag/Docs/Descr/behavior_Contract.md`: the quirk ids the table cites | `parity/out/`, `parity/report.md`, `parity/results.json`: the Rust replay and its report |

Override the work directory with the environment variable `PARITY_WORK` (default `<repo>/Scratch/Oracle`). Everything else is found from the script path, so the scripts work from any current directory.

## Requirements

Linux, bash 5, git (2.55.0 recorded), go (1.27.0 recorded, only to build the oracle), python3 (3.13 recorded), util-linux `setsid`, coreutils `timeout`. The first `go build` downloads the Go modules unless the module cache already has them. A Rust binary to test: cargo is needed only for that, never by the harness. `TMPDIR` (default `/tmp`) must not be inside a git repository, because the Go tool shells out to `git -C <vault>/<skill> ls-files` and would see the outer repo; the runner checks and refuses.

## From a fresh clone to a green parity run

1. Clone with the full history. The commit `c7310f9` must exist locally (the rewrite decision names a tag `go-oracle` on it; the clone at the time of writing had no tags, the scripts use the commit id).
2. Build the oracle (creates the worktree, builds both Go binaries, prints their hashes):

```bash
Code/Development/Parity/build_Oracle.sh
```

   The exact commands inside, run in `Scratch/Oracle/src` after `git worktree add --detach Scratch/Oracle/src c7310f9`: `go build -o ../skill_go .` for the unmodified tool; then `oracle_drv.go.txt` is copied to `oracle_drv/main.go` (it must sit inside the Go module to import `skill_Manag/internal`) and `go build -o ../skill_go_drv ./oracle_drv`. No build flags. `build_Oracle.sh --dry-run` prints these steps and changes nothing.
3. Record the golden tree and check it (about 1 minute for the recording, about 3 minutes for the whole determinism check):

```bash
Code/Development/Parity/run_Scenarios.sh
Code/Development/Parity/check_Determinism.sh
```

   Expected: `run1 vs run2` identical, `golden vs run1` identical, and the tree digest equal to the recorded one below.
4. Build the Rust binary and put a copy where the harness looks (a private target directory keeps your working build untouched; copying keeps a rebuild from changing a run in progress):

```bash
CARGO_TARGET_DIR=Scratch/Oracle/rust_target cargo build --locked
cp Scratch/Oracle/rust_target/debug/skillmirror Scratch/Oracle/skillmirror_under_test
```

5. Replay and classify (about 1.5 minutes on an idle machine with a debug binary, up to 4 on a busy one), then read `Scratch/Oracle/parity/report.md`:

```bash
Code/Development/Parity/check_Parity.sh
```

   Green means UNEXPECTED is 0. The run prints the class counts and one line per UNEXPECTED scenario.

Remove the worktree when done with `git worktree remove --force Scratch/Oracle/src`.

## Recorded reference values

| item | value |
|---|---|
| toolchain | go1.27.0 linux/amd64, git 2.55.0, bash 5.3, python 3.13.7 |
| `skill_go` sha256 | `4fc7c4fa1fc4d28cb5c04351c092d31dcec7a0083fd7b27d5e3b117717b8f314` |
| `skill_go_drv` sha256 | `bee7b0bbf34b929d99c932030adc6696630efa531996619c9c19ad922e1db716` |
| VCS stamp (`go version -m skill_go`) | `vcs.revision=c7310f93887f2c06a04049cf87b48eda18edb807`, `vcs.modified=false`; the driver binary shows `+dirty` (its extra directory is untracked, on purpose) |
| golden tree digest (`check_Determinism.sh --digest-only`) | `6e6ee2d4c8b1033040c79cc522f71810cff3a77041571e42f7a5d0bc4e04b431` |
| golden size | 131 scenarios, 153 steps, 1030 files, 11 MB |
| Rust binary of the last run | `skillmirror 0.1.0`, sha256 `704bad01ce78a25e...`, sources hash `128acde33cdb8586` |

How far the hashes carry. Both Go hashes were reproduced by a second build in the same checkout path with the same toolchain. Go embeds absolute source paths (no `-trimpath`) and the VCS stamp, so another checkout path gives other bytes with the same behaviour; there the check is `go version -m Scratch/Oracle/skill_go`, which must show the revision above and `vcs.modified=false`. `build_Oracle.sh` removes `oracle_drv/` before building `skill_go` so a rebuild keeps the clean stamp. The golden digest is the cross-machine check: it covers every recorded byte after normalization and does not depend on paths. It also depends on the git version (the Go tool prints git's own error text in some scenarios), so a different digest on another git is first a reason to compare `git --version`, then to diff against a golden tree from the original machine.

## How the 131 scenarios are defined

Each scenario is a bash function `sc_<group>_<name>` in `Code/Development/Parity/run_Scenarios.sh`; `run_Scenarios.sh -l` lists them. A scenario calls `desc "one line"` and then one or more `run <label> <go|drv> args...` steps. `go` is the unmodified binary, `drv` is the driver. Groups: `cli_` 8 (help texts, unknown flag or subcommand, too many arguments), `tty_` 4 (flows that need a terminal and fail without one), `cfg_` 29 (config resolution: flag over env over pointer file over vault `config.yaml`, with decoy vault and root to make the winner visible), `scan_` 25 (exclusions, built-in skip folders, symlinks, unreadable folders, file-count semantics), `del_` 23 (delete by name and by project, hostile names, read-only and symlinked targets), `drv_` 42 (sync apply and push: edits, adds, removals, symlinks, modes, odd file names, read-only folders, mandatory list).

Why a driver exists. Only `--dry-run` on the root command, `delete <name> [--project P] [--dry-run]`, `--help` and the error paths run without a terminal in the real Go binary. Everything that writes in sync or push, plus `list`, interactive delete, the menu and Setup, runs inside a Bubble Tea program and fails without a terminal. No terminal is faked. `skill_go_drv` (source `oracle_drv.go.txt`) calls the same `internal` functions as the TUI update loops, in the same order, with every item selected (the TUI default). It is a stand-in, not the shipped binary. Its CLI is the contract a Rust equivalent must meet: `sync|push [--vault V] [--root R] [--dry-run]`, one tab-separated line per target (`ok|ERR`, project, skill, `files=N [list]`, `stale=N [list]`, `err=...`) and a `SUMMARY ok=N err=N dry_run=bool` line.

Fixture per scenario, rebuilt from scratch in a fresh temp directory: `vault/` is the Go repository's `internal/testdata/vault` (read from the worktree) with `git init` and one commit and its `config.yaml` `root:` repointed (the checked-in value names a real scan root; never run the Go tool on a raw copy of the fixtures); `projects/` is `internal/testdata/projects` (four projects below `org/` and `standalone/`, plus one without skills) with an empty `.git` folder next to every `.agents`; `home/` is `HOME` and `XDG_CONFIG_HOME` with the vault pointer file. Recorded per step in `golden/<scenario>/NN_<label>/`: `cmd`, `stdout`, `stderr`, `exit_code`, `projects.manifest`; per scenario `DESCRIPTION`, `rest.manifest` (vault files, home, anything outside `projects/`), sometimes `observations.txt`; `golden/INDEX.tsv` has one line per scenario. Manifest lines, sorted by byte order: `f <mode> <sha256> <relpath>`, `d <mode> - <relpath>/`, `l <mode> -> <target> <relpath>`.

## The scripts

| file | job |
|---|---|
| `paths_Parity.sh` | sourced by all: finds the repo root from the script path, sets `PARITY_WORK`, the divergence table path and the frozen commit |
| `build_Oracle.sh` | worktree at the frozen commit, builds `skill_go` and `skill_go_drv` |
| `run_Scenarios.sh` | defines and runs the scenarios; writes `golden/` or `-o DIR`; usable as a library (`oracle_main`) with two hooks |
| `oracle_tools.py` | `manifest` (sorted tree listing with hashes, `--tmp` substitutes the temp dir inside hashed content) and `normalize` (temp dir to `<TMP>`, sorts dry-run project blocks) |
| `check_Determinism.sh` | two fresh runs, diff against each other and against `golden/`, block-order statistics, tree digest; `--digest-only` prints the digest of `golden/` |
| `translate.sh` | sourced by the parity runner: maps each Go call and `SKILL_MANAG_*` variable to the `skillmirror` grammar, copies the vault pointer file, asks for a plan call |
| `run_Parity.sh` | replays all scenarios (or a named subset) against a `skillmirror` binary into `parity/out/` in the golden layout, plus `plan.json`, `pre.manifest` per step |
| `compare.py` | compares `golden/` with `parity/out/`, classifies, writes `report.md` and `results.json` |
| `check_Parity.sh` | `run_Parity.sh` then `compare.py` |

## Pointing the harness at a fresh Rust binary

`check_Parity.sh [path/to/skillmirror]` or the variable `PARITY_RUST_BIN`; the default is `Scratch/Oracle/skillmirror_under_test`. Optional `Scratch/Oracle/rust_source_sha256.txt` (one line, any stable hash of `Crates/`, `Cargo.toml`, `Cargo.lock`) is copied into the report header so a report names the sources it measured. `run_Parity.sh` sources the scenario file and `translate.sh`, runs each Go call as the Rust call below, and runs a plan call (`... --dry-run --json --all`) before the real call to get the (project, skill, file-count) set.

| Go call | Rust call | plan call |
|---|---|---|
| root `--dry-run [flags]` | `sync --dry-run [flags]` | `sync --dry-run --json --all [flags]` |
| driver `sync [flags]` | `sync --yes [flags]` | same as above |
| driver `push [flags]` | `push --yes [flags]` | `push --dry-run --json --all [flags]` |
| `delete ARGS` | `delete ARGS --yes` | none |
| `list`, `--help`, unknown flag or subcommand, flags only (menu) | unchanged | none |

`SKILL_MANAG_VAULT` and `SKILL_MANAG_ROOT` become `SKILLMIRROR_VAULT` and `SKILLMIRROR_ROOT`; any other `SKILL_MANAG_*` variable is passed through unchanged on purpose (the Rust tool treats it as a hard error and a scenario shows that). The Go pointer file (`$HOME/.config/skill_Manag/vault`) is copied byte for byte to `$XDG_CONFIG_HOME/skillmirror/vault` before every call, or removed when the scenario removed the Go one.

## Reading the result classes

Per step three checks are computed: `tree` (projects manifest after the step, exact), `exit` (Rust code in the set the Go code maps to: Go 0 without error line is 0; Go 0 with an error line is 3 or 4; Go 1 with a cobra usage error is 2; Go 1 otherwise is 3; Go 124 is 124) and `set` (dry-run or driver set against the plan JSON; not applicable when either side printed none). Per scenario one more, `rest`.

- MATCH: every check of every step agrees.
- EXPECTED-DIVERGENCE: some check differs and a row of `divergences.tsv` explains every differing check. The `covers` column lists check kinds; `exit=3` or `exit=2|4` demands that Rust exits with exactly that code, `tree=pre` demands that Rust left the tree as it was before the step. So a divergence in the wrong direction is not accepted.
- UNEXPECTED: a differing check without a row, or a row whose constraint fails, or a scenario that was not run. The report shows both commands, the heads of both outputs, the tree diff and the set diff. Decide per case: a Rust bug (fix Rust), a translation problem (fix `translate.sh`), a Go quirk the contract missed (add it to `behavior_Contract.md` and add a row), or an intended change (add a row with the contract id and the narrowest `covers`).
- SKIPPED: a step that cannot be translated (none today; the Rust CLI needs no terminal).

A row that no longer differs shows under "Entries that matched anyway" in the report: delete it. Optional files in `Scratch/Oracle/parity/`: `observations.md` (copied into the report), `diagnoses.tsv` (scenario, kind, text, shown under UNEXPECTED), `results_previous.json` (adds a "changes since the previous round" section; copy `results.json` there before a new round).

## Adding a scenario

1. Add `sc_<group>_<name>() { desc "..."; ...; }` to `run_Scenarios.sh`. Available in the body: `$V` vault, `$P` projects root, `$H` home, `$FX` fixture dir, `$PA $PB $PC $PD` the four projects, `${FLAGS[@]}` (`--vault $V --root $P`), `sk <project> <skill>`, `set_config` (stdin becomes the vault `config.yaml`), `make_decoy`, `vgit` and `vault_commit` (edit the vault history), `ENV_EXTRA=(NAME=value)` and `RUN_CWD=dir` for the next step, `prelude_sync` or `qrun` for set-up steps that are not recorded, `skip_if_root` for permission tests.
2. Use only Go behaviour that exists at `c7310f9`; the scenario records whatever Go does, including failures.
3. Record twice (`run_Scenarios.sh -o <dir> <name>`) and compare the two trees; fix anything that differs (see determinism rules).
4. Re-record the whole `golden/` (1 minute), run `check_Determinism.sh`, and write the new digest and counts into this file and `parity_Report.md`.
5. Run `check_Parity.sh`; add a row to `divergences.tsv` if Rust differs on purpose. If the scenario needs a new Go grammar, add its case to `translate.sh`.

## Determinism rules

The golden tree is byte-identical across runs, although every run builds its fixtures in another random temp directory. This holds because of these rules; a new scenario must keep them.

- Environment isolation: every call runs under `env -i` with `PATH`, `HOME` and `XDG_CONFIG_HOME` pointing into the fixture, `LANG=C.UTF-8`, `TERM=dumb`, `NO_COLOR=1`, `GIT_CONFIG_NOSYSTEM=1`, `GIT_CONFIG_GLOBAL=/dev/null` (so a user's git config such as `core.quotepath` cannot change `git ls-files`), stdin from `/dev/null`, no controlling terminal (`setsid -w`), `timeout --kill-after=2 30` (a hang is recorded as exit 124), `umask 022`, fixture modes normalized to 644 and 755. The real vault, scan root and config directory are never reachable.
- Paths: the fixture directory is replaced by `<TMP>` in stdout, stderr, `cmd`, symlink targets and inside hashed file contents (pointer file, `config.yaml`). Never write a temp path into a recorded file in another way.
- Ordering: manifests sort by byte order; the driver sorts its own report; `git ls-files` is byte-sorted; the Go walk is lexical. The one random order is the Go dry-run's per-project `● <path>` blocks (map iteration; between two runs 18 and 21 of the 153 steps differed raw in the two checks made, the count itself is random). `oracle_tools.py normalize --sort-blocks` sorts the blocks by header and keeps a leading blank line in place. Nothing else is reordered.
- Time: no timestamp is printed by the tools and no mtime is recorded. A scenario that needs time (`drv_sync_noop` sets every mtime to a fixed date and counts rewrites) writes the result to `observations.txt` as a number, not as a date.
- Randomness on the Rust side: `.stage-<pid>-<timestamp>-<n>` and `.trash-<pid>-<timestamp>-<n>` names appear in stdout of three failure scenarios; they are not compared.

## Regenerating golden, and why it is not committed

`Code/Development/Parity/run_Scenarios.sh` rewrites `Scratch/Oracle/golden/` (it refuses to delete a directory that is not an oracle output tree). The tree is 11 MB in 1030 small files that change whenever a scenario is added, would bloat every clone and review, and is fully derived from the committed scenarios plus the frozen commit. The digest above is the committed proof that a rebuild is the same. Go is frozen, so golden only changes when a scenario is added or the harness is fixed; then the new digest goes into this file in the same commit.

## What must survive the Go removal at cutover

The cutover commit deletes the Go sources from the working tree. The oracle needs only history, not the live tree: the scripts read the Go code and the fixtures from the worktree of `c7310f9`, never from the live tree.

- Keep commit `c7310f9` reachable forever: tag it (`go-oracle` per the rewrite decision), merge with `--no-ff`, never squash or prune it. Without it `build_Oracle.sh` and every fixture are gone.
- Keep `Code/Development/Parity/`, in particular `oracle_drv.go.txt` (the only copy of the driver), `run_Scenarios.sh`, `translate.sh`, `compare.py` and `paths_Parity.sh`. The `.go.txt` ending keeps the Go toolchain from compiling it in the live tree; after cutover there is no Go module there at all.
- Keep `divergences.tsv`, `divergences_Explained.md`, `parity_Report.md` and `Project_Manag/Docs/Descr/behavior_Contract.md` (the ids in the table).
- Keep the `/Scratch/` entry in `.gitignore`, the rebuilt Go toolchain only on machines that run the oracle, and the recorded versions above; the CI job for parity (if any) installs go only to build the oracle.
- After cutover the oracle is a regression pin for the behaviour that was kept, not a target: when a Rust change alters a MATCH scenario on purpose, move it to the table with a contract id instead of editing golden.

## Parity for the new features

Backup store and `undo`, `status`, `diff` and `doctor` have no Go counterpart, so the oracle cannot judge them and must not be stretched to: a scenario without golden is UNEXPECTED by construction. At the time of writing the backup store is half built and `status`, `diff`, `doctor` are not started (see [the backlog](../../../Live_Working/Rust_Rewrite_Handoff/next_Steps.md)); the checks below are proposals for when they exist. They are a separate Rust-only layer in `Crates/Cli/tests/` beside the oracle job, built on the same fixture shape (vault, projects, home) and the same manifest idea so both kinds of test read alike. Three rules keep the two layers apart:

- Existing scenarios keep judging what both tools do. Backups are always on in Rust, so every `sync --yes`, `push --yes` and `delete --yes` also writes a backup run and prints one extra line. Neither touches a compared artifact: the store lives under the fixture's `HOME` (`$HOME/.local/state/skillmirror`), which the `rest` check ignores, and output text is not compared. If a future change does leak into a compared artifact, it is an EXPECTED-DIVERGENCE row with a contract id, not a golden edit.
- A new command gets Rust tests named after the invariant, not after a Go scenario.
- One manifest routine serves both layers: the Rust test kit should produce the `oracle_tools.py manifest` line format, so "tree before equals tree after" reads the same in both.

What the Rust-only verification should look like:

- `undo` (round trips on the oracle fixtures): manifest, `sync --yes`, `undo`, manifest: equal to the first. The same for `delete`, for `push` (a created skill is removed again) and for two applies in a row. `undo` is itself a run, so a second `undo` redoes: the tree equals the state after the sync. Filters by project and skill touch only those targets. Thirty runs are kept: the thirty-first prunes the oldest and warns on a prune failure. Failure cases: an empty store (clear error, no change), an unwritable state directory (hard error, no half state), a read-only target (no backup entry appears, the old copy stays complete), a symlinked skill folder (only the link goes, nothing is stored).
- `status`: it must agree with the engine that is already compared against Go. On every fixture where `sync --dry-run --json --all` is compared, the non-clean (project, skill) pairs in `status --json` equal that set; after a real `sync --yes` every row is clean and `--check` exits 0; a local edit in one project makes exactly that skill "would change"; a stale copy of a skill removed from the vault (`drv_sync_vault_skill_removed` shape) shows as installed but not in the vault; a missing mandatory skill shows as such. It never changes the tree.
- `diff`: for every file the dry run plans, the unified diff names it and shows the change (changed, added, removed, mode-only); empty after `sync --yes`; `--stat` totals equal the plan counts. It never changes the tree.
- `doctor`: one seeded fault per finding on an otherwise clean fixture, each reported once with its hint: a `SKILL.md` that breaks the lint rules, uncommitted or unstaged edits in a skill folder (the `drv_sync_vault_file_changed_uncommitted` shape), a dot-folder skipped by discovery (Q6), clashing names, an invalid `config.yaml`, a leftover `.stage-*` folder. A clean fixture reports nothing. It never changes the tree.
- All four: `--json` output pinned with `insta` snapshots that are read before they are accepted; exit codes asserted; the "never changes the tree" check run for every read-only command on every fixture.

## Calls made when this was written

- The driver is stored as `oracle_drv.go.txt`, not `main.go`, so `go build ./...` in the live tree never compiles it.
- `build.sh` became `build_Oracle.sh` with `--dry-run` and `--help`; `paths_Parity.sh` is new; the destructive-delete guard in `run_Scenarios.sh` and the tree digest in `check_Determinism.sh` are new. Scenarios, normalization and comparison logic are unchanged: a full recording from the new location matched the old golden digest, and the full parity replay reproduced 87 MATCH, 44 EXPECTED-DIVERGENCE, 0 UNEXPECTED.
- `divergences.tsv` has one copy, here; `compare.py` reads it from this folder. `observations.md`, `diagnoses.tsv` and the round-1 files stay in the work directory (round 1 is summarized in `parity_Report.md`); `results_round1.json` became the generic `results_previous.json`.
- `README.md` in `Code/Development/Parity/` is upper case because the task asked for that name; every other file starts lower case.
- The hash of the Go binaries is a record, the golden digest is the check (see "How far the hashes carry").
