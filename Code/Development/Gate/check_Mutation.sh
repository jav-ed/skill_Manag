#!/usr/bin/env bash
# Mutation check for one guard: change a line, run the tests that should notice, put the file back.
# A guard is proven only when at least one test FAILS under the mutation. A green run means a survivor:
# write the missing test, then run the mutation again.
#
# usage: check_Mutation.sh FILE SED_EXPR CARGO_TEST_ARGS...
# e.g.   check_Mutation.sh Crates/Web/src/server/guard.rs 's/"sec-fetch-site"/"sec-fetch-sitx"/' -p skillmirror-web --features server
# The file must be compiled by the tests you name (the web server needs --features server), or nothing is tested.
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$here/../../.."
file="${1:?file}"; expr="${2:?sed expression}"; shift 2
backup="$(mktemp)"
trap 'cp "$backup" "$file"; rm -f "$backup"' EXIT
cp "$file" "$backup"
sed -i "$expr" "$file"
if cmp -s "$file" "$backup"; then echo "!! the expression changed nothing in $file: $expr"; exit 2; fi
echo "--- $file :: $expr"
cargo test "$@" 2>&1 | grep -E "^test .*FAILED|test result: (ok|FAILED)|error(\[|:)" | head -4
