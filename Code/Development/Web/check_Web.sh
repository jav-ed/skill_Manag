#!/usr/bin/env bash
# Builds a throwaway world, starts `skillmirror web --allow-write` on it and drives the pages in a real
# browser (Chromium through Playwright): the first-visit link, the cookie, filter, plan, apply, history,
# undo, a name that is markup, and the refusals. Nothing outside a temporary folder is touched.
#
# usage: check_Web.sh [path/to/skillmirror] [screenshot-dir]
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
binary="${1:-$repo/target/debug/skillmirror}"
shots="${2:-}"
work="$(mktemp -d)"
server_pid=""
cleanup() {
  if [ -n "$server_pid" ]; then kill "$server_pid" 2>/dev/null || true; fi
  rm -rf "$work"
}
trap cleanup EXIT

mkdir -p "$work/vault/coding" "$work/vault/web/astro" "$work/vault/tmux" "$work/home"
printf -- '---\nname: coding\ndescription: Write code that other people can read <and> change\n---\nv2\n' > "$work/vault/coding/SKILL.md"
printf -- '---\nname: astro\ndescription: Astro sites\n---\nv2\nline2\n' > "$work/vault/web/astro/SKILL.md"
echo "ref v2" > "$work/vault/web/astro/ref.md"
printf -- '---\nname: tmux\ndescription: Terminals\n---\nv2\n' > "$work/vault/tmux/SKILL.md"
printf 'root: %s/projects\nmandatory: [tmux]\n' "$work" > "$work/vault/config.yaml"
for p in one two three; do mkdir -p "$work/projects/$p/.agents/skills"; done
mkdir -p "$work/projects/<img src=x onerror=alert(1)>/.agents/skills/mine"
echo hi > "$work/projects/<img src=x onerror=alert(1)>/.agents/skills/mine/SKILL.md"
cp -r "$work/vault/coding" "$work/projects/one/.agents/skills/"
cp -r "$work/vault/coding" "$work/projects/two/.agents/skills/"
sed -i 's/v2/v1/' "$work/projects/one/.agents/skills/coding/SKILL.md" "$work/projects/two/.agents/skills/coding/SKILL.md"
cp -r "$work/vault/web/astro" "$work/projects/one/.agents/skills/"
sed -i 's/v2/v1/' "$work/projects/one/.agents/skills/astro/SKILL.md"
git -C "$work/vault" init -q
git -C "$work/vault" add -A
git -C "$work/vault" -c user.name=t -c user.email=t@t commit -qm snapshot

export HOME="$work/home" XDG_CONFIG_HOME="$work/home/.config" XDG_STATE_HOME="$work/home/.local/state" \
  XDG_CACHE_HOME="$work/home/.cache"
"$binary" --vault "$work/vault" --root "$work/projects" web --allow-write > "$work/server.out" 2> "$work/server.err" &
server_pid=$!
link=""
for _ in $(seq 1 100); do
  link="$(sed -n 's/^Open: //p' "$work/server.out" | head -1)"
  [ -n "$link" ] && break
  sleep 0.1
done
[ -n "$link" ] || { echo "the server printed no link:"; cat "$work/server.out" "$work/server.err"; exit 1; }

export WEB_LINK="$link" WEB_WORK="$work" WEB_SHOTS="$shots"
node "${WEB_DRIVER:-$here/drive_Web.js}"
