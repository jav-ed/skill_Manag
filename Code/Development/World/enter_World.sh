#!/usr/bin/env bash
# Opens a shell inside a fresh sandbox world, to try skillmirror without touching your own vault and projects.
#
# usage: enter_World.sh [lab|first-run]
#   SKILLMIRROR_BIN=path/to/skillmirror   use this build instead of the one on the PATH
#   WORLD_DIR=folder                      where the world is built (default: Scratch/World of this repository)
# The folder is replaced on every call, but only when it carries the marker build_World.sh leaves.
# Scratch/ is disposable, gitignored and skipped by scans (the doc-start convention, `exclude_dirs: Scratch`).
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
stage="${1:-lab}"
dir="${WORLD_DIR:-$repo/Scratch/World}"

die() {
  echo "enter_World: $1" >&2
  [ -z "${2:-}" ] || echo "hint: $2" >&2
  exit 2
}

bin="${SKILLMIRROR_BIN:-$(command -v skillmirror || true)}"
[ -x "$bin" ] || die "no skillmirror binary" "build one (cargo build) and set SKILLMIRROR_BIN, or put skillmirror on the PATH"
bin="$(readlink -f "$bin")"
if [ -e "$dir" ]; then
  [ -f "$dir/.skillmirror-world" ] || die "$dir exists and is not a world" "pick another WORLD_DIR, or remove it yourself"
  rm -rf "$dir"
fi

SKILLMIRROR_BIN="$bin" bash "$here/build_World.sh" "$dir" "$stage"
echo
echo "You are in a throwaway home; $bin is the skillmirror here."
echo "Try: skillmirror status | skillmirror changes | skillmirror agents status | skillmirror sync --dry-run | skillmirror"
echo "Leave with exit; the folder stays until the next call replaces it."
echo

# The world's HOME and none of yours; skillmirror comes first on the PATH so the build you named is the one that runs.
export PATH="$(dirname "$bin"):$PATH"
# shellcheck source=/dev/null
source "$dir/env.sh"
exec "${SHELL:-bash}" -i
