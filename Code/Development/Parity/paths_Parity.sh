#!/usr/bin/env bash
# paths_Parity.sh - sourced by every script in this folder; decides where things live.
#
#   PARITY_CODE   this folder (the scripts)
#   PARITY_REPO   the repository root (three levels up), found from the script path, not from the cwd
#   PARITY_WORK   the work directory with everything that is NOT committed: the rebuilt Go binaries,
#                 the detached worktree at the frozen commit, golden/, the Rust binary under test and
#                 the parity run output. Default: <repo>/Scratch/Oracle (Scratch/ is gitignored);
#                 override with the PARITY_WORK environment variable.
#   PARITY_DIVERGENCES   the tracked divergence table (single copy, read by compare.py)
#
# Idempotent: sourcing it twice, or from a script that was itself sourced, changes nothing.
PARITY_CODE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
PARITY_REPO=$(cd "$PARITY_CODE/../../.." && pwd)
PARITY_WORK=${PARITY_WORK:-$PARITY_REPO/Scratch/Oracle}
PARITY_DIVERGENCES=${PARITY_DIVERGENCES:-$PARITY_REPO/Project_Manag/Docs/Investigation/Parity_Oracle/divergences.tsv}
# The commit the Go oracle is built from (the last Go-only commit of the repository).
PARITY_FROZEN_COMMIT=${PARITY_FROZEN_COMMIT:-c7310f9}
