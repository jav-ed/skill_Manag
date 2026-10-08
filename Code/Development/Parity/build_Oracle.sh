#!/usr/bin/env bash
# build_Oracle.sh - rebuild the two Go oracle binaries from the frozen commit.
#
#   build_Oracle.sh [--dry-run] [-h|--help]
#
# Creates (if missing) a detached git worktree of the frozen commit at $PARITY_WORK/src and builds
#   $PARITY_WORK/skill_go      the unmodified Go tool           (go build .)
#   $PARITY_WORK/skill_go_drv  the TTY-free driver              (go build ./oracle_drv)
# The driver source is oracle_drv.go.txt in this folder; it is copied into the worktree as
# oracle_drv/main.go because it imports skill_Manag/internal and must live inside that Go module.
# (It is kept as .go.txt here so that `go build ./...` in the live tree never picks it up.)
#
# The worktree is cleaned of oracle_drv/ before skill_go is built, so skill_go carries a clean VCS stamp
# (vcs.modified=false) and rebuilds to the same bytes; skill_go_drv is stamped "+dirty" by design.
#
# --dry-run prints every command and changes nothing (no worktree, no build).
# Needs: git, go (1.27 for the recorded hashes, see oracle_Setup.md). Nothing else.
set -euo pipefail
# shellcheck source=paths_Parity.sh
source "$(dirname "${BASH_SOURCE[0]}")/paths_Parity.sh"

dry=0
case ${1:-} in
-h | --help) sed -n '2,17p' "${BASH_SOURCE[0]}"; exit 0 ;;
--dry-run) dry=1 ;;
"") ;;
*) echo "build_Oracle: unknown argument $1" >&2; exit 2 ;;
esac

src=$PARITY_WORK/src
run() {
	if [ $dry = 1 ]; then printf '+'; printf ' %q' "$@"; printf '\n'; else "$@"; fi
}

if [ $dry = 0 ]; then
	command -v go >/dev/null || { echo "build_Oracle: go is required" >&2; exit 1; }
	command -v git >/dev/null || { echo "build_Oracle: git is required" >&2; exit 1; }
	git -C "$PARITY_REPO" rev-parse --verify --quiet "$PARITY_FROZEN_COMMIT^{commit}" >/dev/null ||
		{ echo "build_Oracle: commit $PARITY_FROZEN_COMMIT is not in this clone (fetch the full history)" >&2; exit 1; }
fi

if [ ! -e "$src/.git" ]; then
	run mkdir -p "$PARITY_WORK"
	run git -C "$PARITY_REPO" worktree add --detach "$src" "$PARITY_FROZEN_COMMIT"
fi
if [ $dry = 0 ] && [ "$(git -C "$src" rev-parse --short=7 HEAD)" != "$(git -C "$PARITY_REPO" rev-parse --short=7 "$PARITY_FROZEN_COMMIT")" ]; then
	echo "build_Oracle: worktree $src is not at $PARITY_FROZEN_COMMIT" >&2
	exit 1
fi

if [ $dry = 1 ]; then
	echo "+ rm -rf $src/oracle_drv"
	echo "+ (cd $src && go build -o $PARITY_WORK/skill_go .)"
	echo "+ mkdir -p $src/oracle_drv && cp $PARITY_CODE/oracle_drv.go.txt $src/oracle_drv/main.go"
	echo "+ (cd $src && go build -o $PARITY_WORK/skill_go_drv ./oracle_drv)"
	exit 0
fi

rm -rf "$src/oracle_drv"
(cd "$src" && go build -o "$PARITY_WORK/skill_go" .)
mkdir -p "$src/oracle_drv"
cp "$PARITY_CODE/oracle_drv.go.txt" "$src/oracle_drv/main.go"
(cd "$src" && go build -o "$PARITY_WORK/skill_go_drv" ./oracle_drv)

echo "built: $PARITY_WORK/skill_go  $PARITY_WORK/skill_go_drv"
(cd "$PARITY_WORK" && sha256sum skill_go skill_go_drv)
