#!/usr/bin/env bash
# check_Parity.sh - replay all scenarios against the Rust binary, then compare with golden/.
#
#   check_Parity.sh [-h|--help] [path/to/skillmirror]
#
# Without an argument the binary is $PARITY_WORK/skillmirror_under_test: copy a fresh build there
# first. The binary is deliberately not run from cargo's target/ in place, so a rebuild during the
# run cannot change the result. Writes $PARITY_WORK/parity/report.md and results.json (exit 0 even
# when scenarios are UNEXPECTED: read the counts printed at the end).
set -u
# shellcheck source=paths_Parity.sh
source "$(dirname "${BASH_SOURCE[0]}")/paths_Parity.sh"
case ${1:-} in -h | --help) sed -n '2,10p' "${BASH_SOURCE[0]}"; exit 0 ;; esac
[ $# -gt 0 ] && export PARITY_RUST_BIN=$1
"$PARITY_CODE/run_Parity.sh" || exit 1
python3 "$PARITY_CODE/compare.py"
echo "report: $PARITY_WORK/parity/report.md"
