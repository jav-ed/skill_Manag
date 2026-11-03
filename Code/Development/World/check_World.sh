#!/usr/bin/env bash
# Checks that a skillmirror binary reads the sandbox world the way the world was built to be read: every project is in
# the state its name promises, the first-run story of the docs holds, and a sync, its undo and a promote behave as
# the docs say. The world is plain files and git (build_World.sh), so it is a witness that does not share the tool's bugs.
#
# usage: check_World.sh [path/to/skillmirror]     default: target/debug/skillmirror
# needs: bash, git, python3 (JSON fields only)
# prints one PASS or FAIL line per check; exit status is the number of failures (0 = WORLD OK)
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
SM="$(readlink -f "${1:-$repo/target/debug/skillmirror}")"
[ -x "$SM" ] || { echo "check_World: no executable at $SM (run cargo build first)" >&2; exit 2; }

W="$(mktemp -d)"
trap 'rm -rf "$W"' EXIT
export NO_COLOR=1
fails=0
LAST=""

ok() { echo "PASS  $1"; }
bad() { echo "FAIL  $1"; fails=$((fails + 1)); }
# in_world DIR COMMAND...: run a command with the HOME of a built world and nothing of yours
in_world() { local dir="$1"; shift; bash -c 'source "$1/env.sh" && shift && exec "$@"' _ "$dir" "$@"; }
step() { # name, wanted exit code, command...
  local name="$1" want="$2"; shift 2
  LAST="$("$@" 2>&1)"; local got=$?
  if [ "$got" = "$want" ]; then ok "$name (exit $got)"; else bad "$name (exit $got, wanted $want)"; echo "$LAST" | head -15; fi
}
has() { if printf '%s' "$LAST" | grep -qF -- "$1"; then ok "  output has: $1"; else bad "  output lacks: $1"; echo "$LAST" | head -12; fi; }
hasnt() { if printf '%s' "$LAST" | grep -qF -- "$1"; then bad "  output has: $1"; else ok "  output lacks: $1"; fi; }
file_has() { if grep -qF -- "$2" "$1"; then ok "  $3"; else bad "  $4"; fi; }
file_hasnt() { if grep -qF -- "$2" "$1"; then bad "  $4"; else ok "  $3"; fi; }
want() { # name, JSON path in LAST (dots), expected value
  local got
  got="$(printf '%s' "$LAST" | python3 -c 'import json,sys
d = json.load(sys.stdin)
for k in sys.argv[1].split("."): d = d[k]
print(d)' "$2" 2>/dev/null)"
  if [ "$got" = "$3" ]; then ok "  $1: $2 = $3"; else bad "  $1: $2 = '$got', wanted $3"; fi
}

# ---- the lab world ----------------------------------------------------------------------------------
L="$W/lab"
step "build the lab world" 0 env SKILLMIRROR_BIN="$SM" bash "$here/build_World.sh" "$L" lab
R="$L/home/projects"; V="$L/home/skills"
step "the world starts with no history" 0 in_world "$L" "$SM" history; has "No backups yet"

step "config: the exclusion is read" 0 in_world "$L" "$SM" config show; has "exclude_dirs  testdata"; has "targets       claude"
step "list: five projects built, four with skills, the fixture below testdata skipped" 0 in_world "$L" "$SM" list
has "15 skills installed in 4 projects"; hasnt "testdata"; has "custom"; has "(not in vault)"
step "skills: groups and nesting" 0 in_world "$L" "$SM" skills; has "Web_Skills/"; has "seo/"; has "schema"; has "5 skills in 2 groups"

step "status --json: counts" 1 in_world "$L" "$SM" status --json
want "projects" summary.projects 5; want "in sync" summary.in_sync 1; want "drift" summary.drift 4
want "outdated" summary.outdated_skills 2; want "mandatory missing" summary.missing_mandatory 4
want "bridges missing" summary.missing_bridges 4; want "not in the vault" summary.not_in_vault 2
step "status: what differs, per project" 1 in_world "$L" "$SM" status
has "outdated (1 file added, 1 file changed)"; has "outdated (1 file changed)"; has "mandatory, not installed"
has "not in the vault, sync leaves it alone"; has "bridge to .agents/skills not made"
step "status: the healthy project" 0 in_world "$L" "$SM" status --project "$R/atlas-api"; has "1 project: 1 in sync"

step "changes: docs-site is behind and has a new skill" 0 in_world "$L" bash -c "cd '$R/docs-site' && '$SM' changes"
has "0 edited, 1 behind, 1 new, 2 same"
step "changes: infra edited coding" 1 in_world "$L" bash -c "cd '$R/infra' && '$SM' changes"
has "1 edited, 0 behind, 1 new, 2 same"
step "changes: atlas-api has nothing" 0 in_world "$L" bash -c "cd '$R/atlas-api' && '$SM' changes"
has "0 edited, 0 behind, 0 new, 3 same"

step "agents status: one project in every state" 1 in_world "$L" "$SM" agents status --all
has "atlas-api current"; has "docs-site missing"; has "infra edited"; has "lumen-app outdated"; has "orbit no block"
step "doctor: only the missing bridges" 1 in_world "$L" "$SM" doctor; has "0 errors, 4 warnings"

step "diff: the vault added a file docs-site lacks" 1 in_world "$L" "$SM" diff coding --project "$R/docs-site" --stat
has "references/naming.md | added"
step "sync: plans two updates and adds nothing" 0 in_world "$L" "$SM" sync --dry-run; has "2 to update"; hasnt " to add"

# ---- a sync replaces what a project edited, undo brings it back, promote takes it into the vault --------
step "sync" 0 in_world "$L" "$SM" sync --yes; has "Backup: run"
file_hasnt "$R/infra/.agents/skills/coding/SKILL.md" "Prefer small functions" "the edit made in infra is replaced" "the edit made in infra survived the sync"
step "status after the sync: only the mandatory skills and bridges are left" 1 in_world "$L" "$SM" status; has "0 skill folders outdated"
step "undo" 0 in_world "$L" "$SM" undo --yes
file_has "$R/infra/.agents/skills/coding/SKILL.md" "Prefer small functions" "undo brought the edit back" "undo did not bring the edit back"
step "promote" 0 in_world "$L" bash -c "cd '$R/infra' && '$SM' promote coding --yes"; has "Staged in git, not committed"
file_has "$V/coding/SKILL.md" "Prefer small functions" "the vault has the edit" "the vault lacks the edit"
if git -C "$V" diff --cached --name-only | grep -qx 'coding/SKILL.md'; then ok "  the edit is staged, not committed"; else bad "  the edit is not staged"; fi

# ---- the first-run world: the story of Getting started ------------------------------------------------
F="$W/first"
step "build the first-run world" 0 bash "$here/build_World.sh" "$F" first-run
step "first run: no vault yet" 3 in_world "$F" "$SM" status; has "no vault configured"
step "first run: vault init" 0 in_world "$F" "$SM" vault init "$F/home/skills" --root "$F/home/projects"
step "first run: adopt" 0 in_world "$F" "$SM" adopt coding --from "$F/home/projects/atlas-api"
step "first run: three projects, one master copy" 0 in_world "$F" "$SM" status; has "3 projects: 3 in sync"

step "a folder that is not empty is refused" 2 bash "$here/build_World.sh" "$F" lab; has "is not empty"

echo
if [ "$fails" = 0 ]; then echo "WORLD OK"; else echo "WORLD FAILED: $fails check(s)"; fi
exit "$fails"
