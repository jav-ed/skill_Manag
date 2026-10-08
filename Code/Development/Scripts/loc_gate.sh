#!/usr/bin/env bash
# loc_gate.sh: fail when any given source file has more than 300 lines of code.
#
# Usage: loc_gate.sh <file>...
#
# "Lines of code" are tokei's `code` count: blank lines and comments (`//`, `///`,
# `//!`, nested `/* */`) are excluded, and so are embedded blobs. Inline `#[cfg(test)]`
# modules count, so keep unit tests in sibling files (`#[cfg(test)] mod tests;`).
#
# Exit codes: 0 every file is within the limit
#             1 at least one file is over the limit (the offenders are listed on stderr)
#             2 usage error (no files given)
#             3 tokei or jq is missing, or tokei's output could not be read
#
# The caller decides which files are checked (the just recipe `loc-gate` passes the
# Rust files). tokei is expected at version 15 (`tokei --files --output json`).
set -euo pipefail

limit=300

[ "$#" -gt 0 ] || { echo "usage: loc_gate.sh <file>..." >&2; exit 2; }

for tool in tokei jq; do
  command -v "$tool" >/dev/null 2>&1 || {
    echo "loc_gate.sh: '$tool' not found in PATH; the 300-line gate needs both tokei and jq" >&2
    exit 3
  }
done

# One line per analysed file: "<code lines>\t<path>". The JSON is an object keyed by
# language (plus a "Total" entry that has no per-file reports), each with a `reports`
# array of { name, stats: { code, ... } } (tokei src/language/mod.rs and src/stats.rs).
report=$(tokei --files --output json "$@" \
  | jq -r 'to_entries[] | select(.key != "Total") | .value.reports[] | "\(.stats.code)\t\(.name)"') \
  || { echo "loc_gate.sh: could not read tokei's JSON output" >&2; exit 3; }

# A gate must not pass because tokei silently skipped a file. tokei may or may not list
# empty files, so only the non-empty ones have to show up in the report.
analysed=$(printf '%s\n' "$report" | awk 'NF { n++ } END { print n + 0 }')
expected=0
for file in "$@"; do
  if [ -s "$file" ]; then
    expected=$((expected + 1))
  fi
done
if [ "$analysed" -lt "$expected" ]; then
  echo "loc_gate.sh: tokei analysed $analysed files but $expected non-empty files were given" >&2
  exit 3
fi

over=$(printf '%s\n' "$report" | awk -F '\t' -v limit="$limit" 'NF && $1 + 0 > limit' | sort -rn)
if [ -n "$over" ]; then
  printf 'loc_gate.sh: files over %s code lines (blanks and comments excluded):\n%s\n' "$limit" "$over" >&2
  echo "Split by responsibility; move unit tests into a sibling tests.rs." >&2
  exit 1
fi
