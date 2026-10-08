#!/usr/bin/env bash
# check_Determinism.sh - run every scenario twice into separate trees (each run builds its
# fixtures in a different random temp dir) and diff the runs and the golden tree.
#
#   check_Determinism.sh [-h|--help] [--digest-only]
#     writes $PARITY_WORK/determinism_report.txt (the work directory, see paths_Parity.sh)
#     --digest-only  print only the digest of $PARITY_WORK/golden and exit
#
# Expectation: the normalized outputs of both runs are byte-identical to each other and to
# golden/. stdout.unsorted (path-normalized only, project blocks in Go's own print order)
# is kept in the temp runs so the report can show exactly which scenarios Go prints in a
# random order.
set -u
# shellcheck source=paths_Parity.sh
source "$(dirname "${BASH_SOURCE[0]}")/paths_Parity.sh"
REPORT=$PARITY_WORK/determinism_report.txt
GOLDEN=$PARITY_WORK/golden

# digest of a whole output tree: sha256 over the sorted per-file sha256 lines (paths relative)
tree_digest() { (cd "$1" && find . -type f ! -name '*.unsorted' -print0 | LC_ALL=C sort -z | xargs -0 sha256sum | sha256sum | cut -d' ' -f1); }

case ${1:-} in
-h | --help) sed -n '2,14p' "${BASH_SOURCE[0]}"; exit 0 ;;
--digest-only) [ -d "$GOLDEN" ] || { echo "check_Determinism: no $GOLDEN" >&2; exit 1; }; tree_digest "$GOLDEN"; exit 0 ;;
esac

RUNS=$(mktemp -d "${TMPDIR:-/tmp}/oracle_det.XXXXXX")
trap 'chmod -R u+rwX "$RUNS" 2>/dev/null; rm -rf -- "$RUNS"' EXIT

for n in 1 2; do
	ORACLE_KEEP_UNSORTED=1 "$PARITY_CODE/run_Scenarios.sh" -o "$RUNS/run$n" >/dev/null || exit 1
done

{
	echo "determinism check: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
	echo "scenarios per run: $(wc -l <"$RUNS/run1/INDEX.tsv")"
	echo "files per run:     $(find "$RUNS/run1" -type f ! -name '*.unsorted' | wc -l)"
	echo "tree digest run1:  $(tree_digest "$RUNS/run1")"
	[ -d "$GOLDEN" ] && echo "tree digest golden: $(tree_digest "$GOLDEN")"
	echo

	echo "== 1. run1 vs run2, normalized files (expect no differences)"
	if diff -r -x '*.unsorted' "$RUNS/run1" "$RUNS/run2" >"$RUNS/d12.txt"; then
		echo "identical"
	else
		echo "DIFFERENT:"
		head -40 "$RUNS/d12.txt"
	fi
	echo

	if [ -d "$GOLDEN" ]; then
		echo "== 2. golden/ vs run1, normalized files (expect no differences)"
		if diff -r "$GOLDEN" "$RUNS/run1" -x '*.unsorted' >"$RUNS/dg.txt"; then
			echo "identical"
		else
			echo "DIFFERENT:"
			head -40 "$RUNS/dg.txt"
		fi
		echo
	fi

	echo "== 3. stdout.unsorted (only temp paths normalized): scenarios where run1 and run2 differ"
	echo "   -> Go prints per-project blocks in random map order; golden stdout sorts the blocks"
	total=0 differ=0
	while IFS= read -r f; do
		total=$((total + 1))
		rel=${f#"$RUNS/run1/"}
		if ! cmp -s "$f" "$RUNS/run2/$rel"; then
			differ=$((differ + 1))
			echo "   differs: ${rel%/stdout.unsorted}"
		fi
	done < <(find "$RUNS/run1" -name stdout.unsorted | LC_ALL=C sort)
	echo "   $differ of $total step outputs differed in raw print order across the two runs"
	echo
	echo "== 4. block sort check: sorted stdout equals sorted-from-unsorted of the other run"
	bad=0
	while IFS= read -r f; do
		rel=${f#"$RUNS/run1/"}
		d=${rel%/stdout.unsorted}
		# a differing raw order must collapse to the identical sorted file
		cmp -s "$RUNS/run1/$d/stdout" "$RUNS/run2/$d/stdout" || bad=$((bad + 1))
	done < <(find "$RUNS/run1" -name stdout.unsorted | LC_ALL=C sort)
	echo "   sorted stdout mismatches: $bad"
} | tee "$REPORT"
