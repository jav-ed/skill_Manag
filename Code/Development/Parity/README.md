# Parity harness

Replays the frozen Go tool (commit `c7310f9`) and the Rust `skillmirror` binary through the same 131 fixture scenarios and compares the resulting project trees, exit codes and dry-run sets. Everything generated goes to a work directory outside version control (`Scratch/Oracle` by default); nothing here touches the real vault, the real scan root or the real config directory, and none of it runs cargo. Why it works this way, what the result classes mean and how to add a scenario: `Project_Manag/Docs/Investigation/Parity_Oracle/oracle_Setup.md`.

Needs: bash 5, git, python3, `setsid` (util-linux), `timeout` (coreutils); go only for `build_Oracle.sh`. The commands below are written from the repository root; the scripts find the repository from their own location, so any current directory works.

## Commands

```bash
Code/Development/Parity/build_Oracle.sh                 # worktree at c7310f9 + skill_go + skill_go_drv (--dry-run shows the steps)
Code/Development/Parity/run_Scenarios.sh                # record golden/ from the Go oracle (about 1 minute)
Code/Development/Parity/run_Scenarios.sh -l             # list the 131 scenario names
Code/Development/Parity/run_Scenarios.sh -o /tmp/x cfg_env_only drv_sync_noop   # a subset into another directory
Code/Development/Parity/check_Determinism.sh            # two fresh runs, diff against each other and golden/ (about 3 minutes)
Code/Development/Parity/check_Determinism.sh --digest-only   # digest of golden/, compare with oracle_Setup.md
Code/Development/Parity/check_Parity.sh [path/to/skillmirror]   # replay against Rust, classify, write report.md
Code/Development/Parity/run_Parity.sh cfg_env_only      # replay only (optionally a subset), no comparison
python3 Code/Development/Parity/compare.py              # compare an existing replay with golden/
```

Rust binary under test: `PARITY_RUST_BIN`, the first argument of `check_Parity.sh`, or `Scratch/Oracle/skillmirror_under_test`. Copy a build there instead of running cargo's `target/` in place, so a rebuild cannot change a run in progress.

## Environment variables

| variable | default | meaning |
|---|---|---|
| `PARITY_WORK` | `<repo>/Scratch/Oracle` | work directory: `src/` (worktree), `skill_go`, `skill_go_drv`, `golden/`, `skillmirror_under_test`, `parity/` |
| `PARITY_RUST_BIN` | `$PARITY_WORK/skillmirror_under_test` | binary to replay |
| `PARITY_OUT` | `$PARITY_WORK/parity/out` | where the Rust replay is written |
| `PARITY_DIVERGENCES` | `Project_Manag/Docs/Investigation/Parity_Oracle/divergences.tsv` | table of expected differences |
| `PARITY_FROZEN_COMMIT` | `c7310f9` | commit the Go oracle is built from |
| `TMPDIR` | `/tmp` | fixtures are built here; must not be inside a git repository |
| `ORACLE_OUT`, `ORACLE_GO_BIN`, `ORACLE_DRV_BIN`, `ORACLE_KEEP_UNSORTED`, `ORACLE_RECORD_PRE` | see `run_Scenarios.sh` | overrides used by the parity runner and the determinism check |

## Files

| file | job |
|---|---|
| `paths_Parity.sh` | sourced by the scripts: repo root, work directory, table path |
| `build_Oracle.sh` | builds the two Go binaries from the frozen commit |
| `oracle_drv.go.txt` | source of the TTY-free driver for sync apply and push (copied into the worktree by `build_Oracle.sh`; `.go.txt` keeps the Go toolchain away from it) |
| `run_Scenarios.sh` | the 131 scenarios (`sc_<group>_<name>` functions) and the runner |
| `oracle_tools.py` | tree manifest and output normalization helpers |
| `check_Determinism.sh` | repeatability check and golden digest |
| `translate.sh` | maps Go calls and `SKILL_MANAG_*` variables to the `skillmirror` grammar |
| `run_Parity.sh`, `compare.py`, `check_Parity.sh` | replay, classification, both in one |

The golden tree and the replay output are not committed (11 MB, derived from the scenarios and the frozen commit). Keep commit `c7310f9` reachable (the Go sources were deleted from the branch on 2026-10-08; the harness builds the oracle from that commit in a git worktree, never from the working tree) and keep this folder.
