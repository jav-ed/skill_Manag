#!/usr/bin/env bash
# Builds a throwaway world, writes the report with the given skillmirror binary and drives it in a real
# browser (Chromium through Playwright): filter, "no match" line, theme button, diff anchor, console errors,
# and screenshots. Nothing outside a temporary folder is touched.
#
# usage: check_Report.sh [path/to/skillmirror] [screenshot-dir]
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
binary="${1:-$repo/target/debug/skillmirror}"
shots="${2:-}"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

mkdir -p "$work/vault/coding/ref" "$work/vault/web/astro" "$work/vault/tmux" "$work/home"
printf -- '---\nname: coding\ndescription: Write code that other people can read <and> change\n---\nv2\n' > "$work/vault/coding/SKILL.md"
echo "style v2" > "$work/vault/coding/ref/style.md"
printf -- '---\nname: astro\ndescription: Astro sites\n---\nv2\nline2\n' > "$work/vault/web/astro/SKILL.md"
printf -- '---\nname: tmux\ndescription: Terminals\n---\nv2\n' > "$work/vault/tmux/SKILL.md"
printf 'root: %s/projects\nmandatory: [tmux]\n' "$work" > "$work/vault/config.yaml"
for p in one two three four; do mkdir -p "$work/projects/$p/.agents/skills"; done
cp -r "$work/vault/coding" "$work/projects/one/.agents/skills/"
cp -r "$work/vault/coding" "$work/projects/two/.agents/skills/"
sed -i 's/v2/v1/' "$work/projects/two/.agents/skills/coding/SKILL.md"
rm "$work/projects/two/.agents/skills/coding/ref/style.md"
cp -r "$work/vault/web/astro" "$work/projects/one/.agents/skills/"
mkdir -p "$work/projects/three/.agents/skills/astro" "$work/projects/four/.agents/skills/mine"
echo legacy > "$work/projects/three/.agents/skills/astro/SKILL.md"
echo hi > "$work/projects/four/.agents/skills/mine/SKILL.md"
git -C "$work/vault" init -q
git -C "$work/vault" add -A
git -C "$work/vault" -c user.name=t -c user.email=t@t commit -qm snapshot

HOME="$work/home" XDG_CONFIG_HOME="$work/home/.config" XDG_STATE_HOME="$work/home/.local/state" \
  XDG_CACHE_HOME="$work/home/.cache" "$binary" --vault "$work/vault" --root "$work/projects" \
  report -o "$work/report.html"

export REPORT_HTML="$work/report.html" REPORT_SHOTS="$shots"
node "$here/drive_Report.js"
