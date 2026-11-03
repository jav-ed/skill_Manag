#!/usr/bin/env bash
# Builds the sandbox world: a throwaway HOME with a skill vault and projects in every state skillmirror tells apart.
#
# usage: build_World.sh DIR [lab|first-run]
#   lab        the vault with its history and five projects (in sync, behind, edited, new, missing, outdated, ...)
#   first-run  no vault yet, three projects that carry the same copy of one skill (the Getting started story)
# DIR must not exist or be empty. Needs bash and git. The lab also needs a skillmirror binary (SKILLMIRROR_BIN, else
# the one on PATH), because only the tool can write the AGENTS.md blocks; everything else is plain files and git, so
# the world is an independent witness of what `status`, `changes` and the others must report.
#
# The sources are in Files/ and no folder there is called .agents/skills: a scan of the code folder must never find a
# project inside this repository, because a real sync would write into it. The projects are put together here, in DIR.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
files="$here/Files"

die() {
  echo "build_World: $1" >&2
  [ -z "${2:-}" ] || echo "hint: $2" >&2
  exit 2
}

dest="${1:-}"
stage="${2:-lab}"
[ -n "$dest" ] || die "no folder given" "usage: build_World.sh DIR [lab|first-run]"
case "$stage" in lab | first-run) ;; *) die "unknown stage '$stage'" "use lab or first-run" ;; esac
command -v git >/dev/null || die "git is not installed"
if [ -e "$dest" ] && [ -n "$(ls -A "$dest" 2>/dev/null)" ]; then
  die "$dest is not empty" "pick a new folder; enter_World.sh replaces the world it made before"
fi
bin=""
if [ "$stage" = lab ]; then
  bin="${SKILLMIRROR_BIN:-$(command -v skillmirror || true)}"
  [ -x "$bin" ] || die "no skillmirror binary" "build one (cargo build) and set SKILLMIRROR_BIN, or put skillmirror on the PATH"
  bin="$(readlink -f "$bin")"
fi

mkdir -p "$dest"
dest="$(cd "$dest" && pwd)"
home="$dest/home"
root="$home/projects"
vault="$home/skills"
mkdir -p "$root" "$home/.config" "$home/.local/state"

# Everything below runs as the world, never as you: its HOME, its git identity, none of your git configuration.
export HOME="$home" XDG_CONFIG_HOME="$home/.config" XDG_STATE_HOME="$home/.local/state"
export GIT_AUTHOR_NAME=World GIT_AUTHOR_EMAIL=world@example.invalid
export GIT_COMMITTER_NAME=World GIT_COMMITTER_EMAIL=world@example.invalid
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null
unset SKILLMIRROR_VAULT SKILLMIRROR_ROOT

# put_skill PROJECT NAME SOURCE: the folder SOURCE becomes PROJECT/.agents/skills/NAME
put_skill() {
  mkdir -p "$1/.agents/skills"
  cp -a "$3" "$1/.agents/skills/$2"
}
# seal DIR MESSAGE: commit everything in DIR (a project is a git repository, as real projects are)
seal() {
  git -C "$1" add -A
  git -C "$1" commit -q -m "${2:-init}"
}
new_project() {
  mkdir -p "$root/$1"
  git -C "$root/$1" init -q -b main
}

# ---- first-run: three projects that each carry their own copy of coding --------------------------------
if [ "$stage" = first-run ]; then
  for p in atlas-api docs-site infra; do
    new_project "$p"
    put_skill "$root/$p" coding "$files/older/coding_v1"
    seal "$root/$p"
  done
fi

# ---- lab: the vault, three commits deep, so that a project can hold an older version of coding ----------
if [ "$stage" = lab ]; then
  git init -q -b main "$vault"
  cp -a "$files/vault/." "$vault/"
  config="$(<"$vault/config.yaml")"
  printf '%s\n' "${config//__ROOT__/$root}" >"$vault/config.yaml"
  rm -rf "$vault/coding"
  mkdir "$vault/coding"
  cp "$files/older/coding_v1/SKILL.md" "$vault/coding/"
  cp "$files/older/agents_v1.md" "$vault/project-files/AGENTS.md"
  seal "$vault" "seed: the first versions"
  cp "$files/older/coding_v2/SKILL.md" "$vault/coding/SKILL.md"
  seal "$vault" "coding: 250 lines"
  rm -rf "$vault/coding"
  cp -a "$files/vault/coding" "$vault/coding"
  seal "$vault" "coding: naming reference"

  # the pointer file is how the tool finds the vault
  mkdir -p "$XDG_CONFIG_HOME/skillmirror"
  (umask 077 && printf '%s\n' "$vault" >"$XDG_CONFIG_HOME/skillmirror/vault")

  # atlas-api: everything current, the .claude link in place
  new_project atlas-api
  for s in coding doc-start file-tree-optimization; do put_skill "$root/atlas-api" "$s" "$files/vault/$s"; done
  mkdir -p "$root/atlas-api/.claude"
  ln -s ../.agents/skills "$root/atlas-api/.claude/skills"
  # a fixture project below testdata: config.yaml skips testdata folders, so it must stay out of every list
  put_skill "$root/atlas-api/testdata/sample" coding "$files/older/coding_v1"

  # docs-site: coding is the version before the newest (behind), a skill only this project has, one mandatory skill missing
  new_project docs-site
  put_skill "$root/docs-site" coding "$files/older/coding_v2"
  put_skill "$root/docs-site" astro "$files/vault/Web_Skills/astro"
  put_skill "$root/docs-site" custom "$files/local/custom"
  put_skill "$root/docs-site" doc-start "$files/vault/doc-start"

  # infra: coding edited here (a line the vault never had), a skill only this project has
  new_project infra
  for s in coding doc-start file-tree-optimization; do put_skill "$root/infra" "$s" "$files/vault/$s"; done
  printf -- '- Prefer small functions.\n' >>"$root/infra/.agents/skills/coding/SKILL.md"
  put_skill "$root/infra" runbooks "$files/local/runbooks"

  # lumen-app: all current, an AGENTS.md block written from the first text (it falls behind below)
  new_project lumen-app
  for s in coding doc-start file-tree-optimization; do put_skill "$root/lumen-app" "$s" "$files/vault/$s"; done
  put_skill "$root/lumen-app" astro "$files/vault/Web_Skills/astro"
  "$bin" agents add --project "$root/lumen-app" --yes >/dev/null

  # orbit: an empty skills folder and an AGENTS.md of its own, without a block
  new_project orbit
  mkdir -p "$root/orbit/.agents/skills"
  cp "$files/local/orbit_AGENTS.md" "$root/orbit/AGENTS.md"

  # the rules grow by one (lumen-app is now behind); atlas-api gets the new text, infra gets it and then an edit inside the block
  cp "$files/vault/project-files/AGENTS.md" "$vault/project-files/AGENTS.md"
  seal "$vault" "agents text: tests rule"
  "$bin" agents add --project "$root/atlas-api" --yes >/dev/null
  "$bin" agents add --project "$root/infra" --yes >/dev/null
  sed -i 's/^3\. No soft fallbacks/3. No fallbacks at all/' "$root/infra/AGENTS.md"

  for p in atlas-api docs-site infra lumen-app orbit; do seal "$root/$p"; done
  # the three agents runs above are the builder's, not yours: the world starts with an empty history
  rm -rf "$XDG_STATE_HOME/skillmirror"
fi

# ---- the shell files: how to be inside the world, and a prompt that says so -----------------------------
printf 'PROMPT="[world] %%~ %%%% "\n' >"$home/.zshrc"
printf 'PS1="[world] \\w \\$ "\n' >"$home/.bashrc"
{
  echo "# Source this file to work inside the world: the shell then has the world's HOME and nothing of yours."
  printf 'export HOME=%q XDG_CONFIG_HOME=%q XDG_STATE_HOME=%q\n' "$home" "$home/.config" "$home/.local/state"
  echo "export GIT_AUTHOR_NAME=World GIT_AUTHOR_EMAIL=world@example.invalid GIT_COMMITTER_NAME=World GIT_COMMITTER_EMAIL=world@example.invalid"
  echo "export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null"
  echo "unset SKILLMIRROR_VAULT SKILLMIRROR_ROOT"
  echo 'cd "$HOME"'
} >"$dest/env.sh"
echo "$stage" >"$dest/.skillmirror-world"

echo "World ($stage) built in $dest"
echo "  HOME      $home"
if [ "$stage" = lab ]; then echo "  vault     $vault"; fi
echo "  projects  $root"
echo "  enter     source $dest/env.sh"
