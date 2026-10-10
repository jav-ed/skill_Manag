#!/usr/bin/env bash
# End-to-end smoke test of a built skillmirror binary: authoring, install, drift, sync, undo and redo,
# adopt, scoped add, and the local web server over HTTP (token, cookie, refusals, plan, apply, job).
# It runs in a throwaway HOME, vault and scan root; nothing outside one temporary folder is touched.
#
# usage: check_Smoke.sh [path/to/skillmirror]     default: target/debug/skillmirror
# needs: bash, git, curl, python3 (JSON fields only)
# prints one PASS or FAIL line per check; exit status is the number of failures (0 = SMOKE OK)
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
SM="$(readlink -f "${1:-$repo/target/debug/skillmirror}")"
[ -x "$SM" ] || { echo "check_Smoke: no executable at $SM (run cargo build first)" >&2; exit 2; }

W="$(mktemp -d)"
WP=""
trap '[ -n "$WP" ] && kill "$WP" 2>/dev/null; rm -rf "$W"' EXIT
export HOME="$W/home"; mkdir -p "$HOME"
unset SKILLMIRROR_VAULT SKILLMIRROR_ROOT
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null
V="$W/vault"; R="$W/code"; mkdir -p "$R"
A="$R/p1/.agents/skills/alpha/SKILL.md"
fails=0
LAST=""

ok() { echo "PASS  $1"; }
bad() { echo "FAIL  $1"; fails=$((fails + 1)); }
step() { # name, wanted exit code, command...
  local name="$1" want="$2"; shift 2
  LAST="$("$@" 2>&1)"; local got=$?
  if [ "$got" = "$want" ]; then ok "$name (exit $got)"; else bad "$name (exit $got, wanted $want)"; echo "$LAST" | head -15; fi
}
has() { if printf '%s' "$LAST" | grep -qF -- "$1"; then ok "  output has: $1"; else bad "  output lacks: $1"; echo "$LAST" | head -12; fi; }
exists() { if [ -e "$1" ]; then ok "  exists: ${1#"$W"/}"; else bad "  missing: ${1#"$W"/}"; fi; }
absent() { if [ ! -e "$1" ]; then ok "  absent: ${1#"$W"/}"; else bad "  present: ${1#"$W"/}"; fi; }
contains() { if grep -q "$2" "$1"; then ok "  $3"; else bad "  $4"; fi; }
lacks() { if grep -q "$2" "$1"; then bad "  $4"; else ok "  $3"; fi; }
code_of() { curl -s -o "${OUT:-/dev/null}" -w '%{http_code}' "$@"; }
expect_code() { # name, accepted codes (space separated), got
  case " $2 " in *" $3 "*) ok "$1 ($3)";; *) bad "$1 (got $3, wanted $2)";; esac
}
json_field() { python3 -c 'import json,sys;print(json.load(open(sys.argv[1]))[sys.argv[2]])' "$1" "$2" 2>/dev/null; }
J=(-H 'Content-Type: application/json' -H 'X-Skillmirror: 1')

# ---- the command line -----------------------------------------------------------------------------
step "vault init" 0 "$SM" vault init "$V" --root "$R"
exists "$V/config.yaml"
step "new skill" 0 "$SM" new alpha --vault "$V" --root "$R"
exists "$V/alpha/SKILL.md"
(cd "$V" && git add -A && git commit -qm seed)
# the text every project's AGENTS.md block is filled from: seeded from the built-in rules, then edited
step "agents seed, dry run" 0 "$SM" agents seed --dry-run --vault "$V" --root "$R"; has "Would write"
absent "$V/project-files"
step "agents seed" 0 "$SM" agents seed --vault "$V" --root "$R"; has "project-files/AGENTS.md"
contains "$V/project-files/AGENTS.md" "# Coding related" "seed wrote the built-in rules" "seed did not write the built-in rules"
step "agents seed does not overwrite" 3 "$SM" agents seed --vault "$V" --root "$R"; has "already exists"
printf '# Rules\n\n1. old rule\n' > "$V/project-files/AGENTS.md"
printf 'about the vault repository itself\n' > "$V/AGENTS.md"
step "mandatory add" 0 "$SM" mandatory add alpha --vault "$V" --root "$R"
(cd "$V" && git add -A && git commit -qm mandatory)
step "doctor" 0 "$SM" doctor --vault "$V" --root "$R"

step "init project, dry run" 0 "$SM" init "$R/p1" --dry-run --vault "$V" --root "$R"
absent "$R/p1"
step "init project" 0 "$SM" init "$R/p1" -y --vault "$V" --root "$R"
exists "$A"
step "status, in sync" 0 "$SM" status --vault "$V" --root "$R"

echo "extra line" >> "$V/alpha/SKILL.md"; (cd "$V" && git add -A && git commit -qm edit)
step "status, drift" 1 "$SM" status --vault "$V" --root "$R"
step "diff names the change" 1 "$SM" diff --vault "$V" --root "$R"; has "extra line"
step "sync, dry run" 0 "$SM" sync --dry-run --vault "$V" --root "$R"
lacks "$A" "extra line" "the dry run wrote nothing" "the dry run wrote"
step "sync" 0 "$SM" sync -y --vault "$V" --root "$R"
contains "$A" "extra line" "sync copied the change" "sync did not copy the change"
step "status, in sync again" 0 "$SM" status --vault "$V" --root "$R"

step "history" 0 "$SM" history --vault "$V" --root "$R"
step "undo" 0 "$SM" undo -y --vault "$V" --root "$R"
lacks "$A" "extra line" "undo brought the old file back" "undo left the change"
step "redo (a second undo)" 0 "$SM" undo -y --vault "$V" --root "$R"
contains "$A" "extra line" "redo brought the change back" "redo did not bring the change back"

mkdir -p "$R/p1/.agents/skills/handmade"
printf -- '---\nname: handmade\ndescription: made by hand\n---\nbody\n' > "$R/p1/.agents/skills/handmade/SKILL.md"
step "adopt" 0 "$SM" adopt handmade --from "$R/p1" --vault "$V" --root "$R"
exists "$V/handmade/SKILL.md"
step "info" 0 "$SM" info handmade --vault "$V" --root "$R"; has "handmade"
step "info, unknown skill" 3 "$SM" info nope --vault "$V" --root "$R"
step "init second project" 0 "$SM" init "$R/p2" -y --vault "$V" --root "$R"
(cd "$V" && git add -A && git commit -qm adopt)
step "add handmade to the second project" 0 "$SM" add handmade -y --project "$R/p2" --vault "$V" --root "$R"
exists "$R/p2/.agents/skills/handmade/SKILL.md"
step "status as JSON" 0 "$SM" status --json --vault "$V" --root "$R"; has '"projects"'

# ---- AGENTS.md: written by init, then checked, updated and undone; the project's own text beside the block
P="$R/p1/AGENTS.md"
exists "$P"
contains "$P" "skillmirror:autogenerated begin" "init wrote the begin marker" "init wrote no begin marker"
contains "$P" "1. old rule" "init wrote the vault text" "init did not write the vault text"
lacks "$P" "about the vault repository" "the AGENTS.md at the vault root was not used" "the AGENTS.md at the vault root was used"
contains "$P" "change only what comes below it" "the block ends with the note about what may be changed" "the block has no closing note"
step "agents status, every project current" 0 "$SM" agents status --vault "$V" --root "$R"; has "2 current"
rm "$R/p2/AGENTS.md"
step "agents status, one file missing" 1 "$SM" agents status --project "$R/p2" --vault "$V" --root "$R"; has "missing"
step "agents add" 0 "$SM" agents add -y --project "$R/p2" --vault "$V" --root "$R"
exists "$R/p2/AGENTS.md"
echo "my own notes" >> "$P"
step "agents status, still current beside your notes" 0 "$SM" agents status --project "$R/p1" --vault "$V" --root "$R"
printf '# Rules\n\n1. new rule\n' > "$V/project-files/AGENTS.md"
LAST="$("$SM" status --vault "$V" --root "$R" 2>&1)"; has "AGENTS.md blocks: 2 out of date"
step "agents status, outdated" 1 "$SM" agents status --project "$R/p1" --vault "$V" --root "$R"; has "outdated"
step "agents sync, dry run" 0 "$SM" agents sync --dry-run --diff --vault "$V" --root "$R"; has "+1. new rule"
lacks "$P" "1. new rule" "the dry run wrote nothing" "the dry run wrote"
step "agents sync" 0 "$SM" agents sync -y --vault "$V" --root "$R"
contains "$P" "1. new rule" "sync wrote the new text" "sync did not write the new text"
contains "$P" "my own notes" "your own text beside the block survived" "your own text was lost"
lacks "$P" "1. old rule" "the old text is gone from the block" "the old text is still there"
step "undo the agents sync" 0 "$SM" undo -y --vault "$V" --root "$R"
contains "$P" "1. old rule" "undo brought the old block back" "undo did not bring the old block back"
contains "$P" "my own notes" "undo kept your own text" "undo lost your own text"
step "init without the file" 0 "$SM" init "$R/p4" -y --no-agents-md --vault "$V" --root "$R"
absent "$R/p4/AGENTS.md"
printf '   \n' > "$V/project-files/AGENTS.md"
step "init refuses an unusable vault text and makes nothing" 3 "$SM" init "$R/p5" -y --vault "$V" --root "$R"; has "is empty"
absent "$R/p5"
step "doctor reports the unusable text" 3 "$SM" doctor --vault "$V" --root "$R"; has "agents-text"
rm -f "$V/project-files/AGENTS.md"

# ---- the web server over HTTP ---------------------------------------------------------------------
LOG="$W/web.log"
"$SM" web --allow-write --vault "$V" --root "$R" --idle-timeout 5 >"$LOG" 2>&1 &
WP=$!
for _ in $(seq 1 50); do grep -q 'http://127.0.0.1' "$LOG" && break; sleep 0.1; done
URL="$(grep -o 'http://127.0.0.1[^ ]*' "$LOG" | head -1)"
if [ -z "$URL" ]; then bad "the web server printed no link"; cat "$LOG"; exit "$fails"; fi
BASE="${URL%%\?*}"; BASE="${BASE%/}"; JAR="$W/jar"
expect_code "the link opens the interface" 200 "$(OUT="$W/index.html" code_of -c "$JAR" -L "$URL")"
if grep -qi '<script' "$W/index.html"; then ok "  the page loads its script"; else bad "  the page has no script"; fi
expect_code "no cookie, no data" "401 403" "$(code_of "$BASE/api/overview")"
expect_code "the cookie opens the data" 200 "$(OUT="$W/overview.json" code_of -b "$JAR" "$BASE/api/overview")"
expect_code "a wrong Host header is refused" "400 403 421" "$(code_of -b "$JAR" -H 'Host: evil.example' "$BASE/api/overview")"
expect_code "a change without the JSON header is refused" "400 403 415" "$(code_of -b "$JAR" -X POST -d '{}' "$BASE/api/plan")"

echo "web edit" >> "$V/alpha/SKILL.md"; (cd "$V" && git add -A && git commit -qm webedit)
curl -s -b "$JAR" -X POST "${J[@]}" -d '{}' "$BASE/api/rescan" >/dev/null
curl -s -b "$JAR" -X POST "${J[@]}" -d '{"kind":"sync","skills":["alpha"]}' "$BASE/api/plan" >"$W/plan.json"
PLAN="$(json_field "$W/plan.json" plan)"; UPD="$(json_field "$W/plan.json" update)"
if [ -n "$PLAN" ] && [ "${UPD:-0}" -ge 1 ]; then ok "the plan lists $UPD update(s)"; else bad "the plan: $(head -c 300 "$W/plan.json")"; fi
lacks "$A" "web edit" "planning wrote nothing" "planning wrote to the project"
curl -s -b "$JAR" -X POST "${J[@]}" -d "{\"plan\":\"$PLAN\"}" "$BASE/api/apply" >"$W/apply.json"
JOB="$(json_field "$W/apply.json" job)"; STATE=""
for _ in $(seq 1 50); do
  curl -s -b "$JAR" "$BASE/api/job/$JOB" >"$W/job.json"
  STATE="$(json_field "$W/job.json" state)"
  [ "$STATE" = done ] && break; sleep 0.1
done
if [ "$STATE" = done ]; then ok "the job finished"; else bad "the job ended as '$STATE': $(head -c 300 "$W/job.json")"; fi
contains "$A" "web edit" "the apply changed the project" "the apply did not change the project"
expect_code "a plan runs once" "400 404 409" "$(code_of -b "$JAR" -X POST "${J[@]}" -d "{\"plan\":\"$PLAN\"}" "$BASE/api/apply")"

echo
if [ "$fails" = 0 ]; then echo "SMOKE OK"; else echo "SMOKE FAILED: $fails check(s)"; fi
exit "$fails"
