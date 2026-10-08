#!/usr/bin/env bash
# The whole local gate in one command; stops at the first failing step and says which one.
# It is what CI runs plus the steps CI does not have yet (features, the second toolchain's clippy).
#
# usage: check_Gate.sh
# env:   GATE_SECOND_TOOLCHAIN   a second rustc to run fmt and clippy on (default 1.99.0, the one CI uses;
#                                the lints differ between versions). Skipped, and said so, when not installed.
# needs: cargo, just, tokei, jq (loc-gate), cargo-deny
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$here/../../.."
second="${GATE_SECOND_TOOLCHAIN:-1.99.0}"
step() { echo "== $*"; }
quiet_clippy() { # toolchain-flag...
  "$@" clippy --workspace --all-targets --locked -- -D warnings 2>&1 | tail -1
  test "${PIPESTATUS[0]}" -eq 0
}

step "fmt";    cargo fmt --all -- --check
step "clippy"; quiet_clippy cargo
if rustup toolchain list 2>/dev/null | grep -q "^$second"; then
  step "fmt $second";    cargo "+$second" fmt --all -- --check
  step "clippy $second"; quiet_clippy cargo "+$second"
else
  echo "== second toolchain $second is not installed: its fmt and clippy are SKIPPED (rustup toolchain install $second)"
fi
step "loc-gate";   just loc-gate
step "features";   just check-features
step "check-deps"; just check-deps
step "deny";       just deny 2>&1 | sed 's/\x1b\[[0-9;]*m//g' | tail -1; test "${PIPESTATUS[0]}" -eq 0
step "tests"
out="$(cargo test --workspace --locked --no-fail-fast 2>&1)" || {
  echo "$out" | grep -E '^test .*FAILED|panicked'
  echo "TESTS FAILED"
  exit 1
}
echo "$out" | grep -E '^test result' | awk '{p+=$4; f+=$6} END {print "passed:", p, "failed:", f}'
echo "GATE OK"
