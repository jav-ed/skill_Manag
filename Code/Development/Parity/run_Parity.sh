#!/usr/bin/env bash
# run_Parity.sh - replay every Go oracle scenario against a skillmirror binary and write the results in
# the golden layout (same fixtures, same normalization, same manifests).
#
#   run_Parity.sh [-h|--help] [scenario ...]
#
# Env: PARITY_RUST_BIN  the binary under test (default: $PARITY_WORK/skillmirror_under_test; copy a
#                       build there first, so a rebuild during a run cannot change the run)
#      PARITY_OUT       output tree (default: $PARITY_WORK/parity/out)
#      PARITY_WORK      work directory (default: <repo>/Scratch/Oracle, see paths_Parity.sh)
#      TMPDIR           fixtures are built here; must not be inside a git repo
# Extra files per step compared to golden: plan.json (skillmirror ... --dry-run --json --all, run
# BEFORE the real call), plan_exit_code, plan_cmd, pre.manifest (projects tree before the call),
# orig_cmd/SKIPPED for untranslatable steps. See translate.sh for the grammar mapping.
set -u
# shellcheck source=paths_Parity.sh
source "$(dirname "${BASH_SOURCE[0]}")/paths_Parity.sh"
case ${1:-} in -h | --help) sed -n '2,14p' "${BASH_SOURCE[0]}"; exit 0 ;; esac

export PARITY_RUST_BIN=${PARITY_RUST_BIN:-$PARITY_WORK/skillmirror_under_test}
[ -x "$PARITY_RUST_BIN" ] || { echo "run_Parity: missing $PARITY_RUST_BIN (set PARITY_RUST_BIN or copy a build there)" >&2; exit 1; }
export ORACLE_RECORD_PRE=1
export ORACLE_GO_BIN=$PARITY_RUST_BIN ORACLE_DRV_BIN=$PARITY_RUST_BIN
export ORACLE_OUT=${PARITY_OUT:-$PARITY_WORK/parity/out}
# the scenario file also needs the frozen worktree (fixtures), not the Go binaries
[ -d "$PARITY_WORK/src/internal/testdata/vault" ] ||
	{ echo "run_Parity: no fixtures at $PARITY_WORK/src (run build_Oracle.sh first)" >&2; exit 1; }

# shellcheck source=run_Scenarios.sh
source "$PARITY_CODE/run_Scenarios.sh"
# shellcheck source=translate.sh
source "$PARITY_CODE/translate.sh" || { echo "run_Parity: cannot load translate.sh" >&2; exit 1; }

oracle_main -o "$ORACLE_OUT" "$@"
"$PARITY_RUST_BIN" --version >"$ORACLE_OUT/RUST_VERSION" 2>&1
sha256sum "$PARITY_RUST_BIN" | cut -d' ' -f1 >"$ORACLE_OUT/RUST_BIN_SHA256"
# optional: hash of the sources the binary was built from (see README.md)
[ -f "$PARITY_WORK/rust_source_sha256.txt" ] && cp "$PARITY_WORK/rust_source_sha256.txt" "$ORACLE_OUT/RUST_SOURCE_SHA256"
exit 0
