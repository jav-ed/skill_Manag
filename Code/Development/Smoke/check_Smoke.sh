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
