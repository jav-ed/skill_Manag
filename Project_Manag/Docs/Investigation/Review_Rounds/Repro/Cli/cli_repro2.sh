#!/usr/bin/env bash
# Review round 2, series K (command line). Findings: M6 (K1), L9 (K2), L10 (K7, read only).
# Uses an isolated HOME and temp dirs only; never touches a real vault or project.
# Usage: BIN=path/to/skillmirror bash cli_repro2.sh
set -u
BIN=${BIN:?set BIN to the skillmirror binary, e.g. target/debug/skillmirror}
T=$(mktemp -d)
export HOME=$T/home XDG_CONFIG_HOME=$T/home/.config XDG_STATE_HOME=$T/home/.state XDG_CACHE_HOME=$T/home/.cache
mkdir -p "$HOME"
for v in $(env | grep -o '^GIT_[A-Z_]*'); do unset "$v"; done
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null
g() { git -C "$1" -c user.name=t -c user.email=t@t -c commit.gpgsign=false "${@:2}"; }

V=$T/vault; R=$T/projects
mkdir -p "$V/good" "$V/bad" "$R"
echo good > "$V/good/SKILL.md"
echo bad  > "$V/bad/ref.md"
git init -q "$V"
printf 'root: %s\nmandatory: [good, bad]\n' "$R" > "$V/config.yaml"
g "$V" add -A; g "$V" commit -q -m init
echo "SKILL.md of bad stays untracked" > "$V/bad/SKILL.md"      # R: SKILL.md exists on disk, git does not track it

run() { echo; echo "\$ skillmirror $*"; "$BIN" --vault "$V" --root "$R" "$@"; echo "[exit $?]"; }

echo "=== K1: init when every skill fails to plan (only 'bad')"
printf 'root: %s\nmandatory: [bad]\n' "$R" > "$V/config.yaml"
run init "$T/new1" --yes
echo "new1 exists after the failed init: $([ -d "$T/new1" ] && echo YES || echo no)"; ls -A "$T/new1" 2>/dev/null
run init "$T/new1b" --git --yes
echo "new1b exists: $([ -d "$T/new1b" ] && echo YES || echo no); has .git: $([ -d "$T/new1b/.git" ] && echo YES || echo no)"
run init "$T/new1b" --git --yes   # second try after the failure

echo; echo "=== K2: init --git with GIT_DIR in the environment"
printf 'root: %s\nmandatory: [good]\n' "$R" > "$V/config.yaml"
mkdir -p "$T/elsewhere"
GIT_DIR=$T/elsewhere/other.git "$BIN" --vault "$V" --root "$R" init "$T/new2" --git --yes
echo "[exit $?]"
echo "new2/.git exists: $([ -e "$T/new2/.git" ] && echo YES || echo no); GIT_DIR target created: $([ -d "$T/elsewhere/other.git" ] && echo YES || echo no)"

echo; echo "=== K3: dry runs write nothing"
mkdir -p "$T/proj3"
run add good --project "$T/proj3" --dry-run
echo "proj3/.agents exists: $([ -e "$T/proj3/.agents" ] && echo YES || echo no)"
run init "$T/new3" --dry-run
echo "new3 exists: $([ -e "$T/new3" ] && echo YES || echo no)"

echo; echo "=== K4: add twice (the second run is up to date)"
run add good --project "$T/proj3" --yes
run add good --project "$T/proj3" --yes

echo; echo "=== K5: add where .agents/skills is a link"
mkdir -p "$T/shared/skills" "$T/proj5"
ln -s "$T/shared" "$T/proj5/.agents"
run add good --project "$T/proj5" --yes

echo; echo "=== K6: group, profile and unknown names"
mkdir -p "$V/web/seo"; echo s > "$V/web/seo/SKILL.md"; g "$V" add -A; g "$V" commit -q -m seo
run skills
run skills --group web
run skills --group nope
run add --group web --project "$T/proj3" --dry-run
run add --profile nope --project "$T/proj3" --dry-run
run add --project "$T/proj3" --dry-run

echo; echo "=== K7: --json with prompts is not valid JSON on stdout (needs a tty, not run here)"
echo "(reading only: pipeline.rs confirm_write prints the rows before asking, also with --json)"

rm -rf "$T"
