#!/usr/bin/env bash
# Bootstrap a new repo with the canonical doc-start structure.
# Safe to re-run: existing files and folders are never overwritten.
#
# Usage:
#   bootstrap.sh            # run in current directory
#   bootstrap.sh <path>     # run in given directory

set -euo pipefail

ROOT="${1:-.}"
cd "$ROOT"

create_dir() {
  local dir="$1"
  if [[ -d "$dir" ]]; then
    echo "  exists:  $dir/"
  else
    mkdir -p "$dir"
    echo "  created: $dir/"
  fi
}

create_file() {
  local path="$1"
  local content="$2"
  if [[ -f "$path" ]]; then
    echo "  exists:  $path"
  else
    printf '%s' "$content" > "$path"
    echo "  created: $path"
  fi
}

ensure_gitignore_line() {
  local line="$1"
  touch .gitignore
  if grep -Fxq "$line" .gitignore; then
    echo "  exists:  .gitignore line '$line'"
  else
    printf '%s\n' "$line" >> .gitignore
    echo "  added:   .gitignore line '$line'"
  fi
}

echo "doc-start bootstrap in $(pwd)"

# Required folders under Project_Manag/Docs/
for area in Architecture Decisions Descr Research Setup; do
  create_dir "Project_Manag/Docs/$area"
done

# Live_Working folder
create_dir "Project_Manag/Live_Working"

# doc_Start.md under Project_Manag/Docs/
create_file "Project_Manag/Docs/doc_Start.md" "$(cat <<'EOF'
# doc_Start

*This `doc_Start.md` is the docs entry point, structured so an agent can quickly decide what to read and what to skip. It opens with a short summary of the repo and key entry-point files, then routes to each topic area through labeled links. Open a linker only when the task calls for it; the labels are written to make that decision possible without clicking.*

A short summary of the repo: what it is, what it does, and the broad shape (kind of system, primary stack, what is in scope and what is out of scope). Replace this paragraph with the actual summary.

Entry point(s): the key files an agent would need to know to orient in the code without reading anything (e.g. `src/main.js`). One line per file.

## Docs

- [Architecture](Architecture/linker_Architecture.md): structure, systems, technical layout, integration boundaries
- [Decisions](Decisions/linker_Decisions.md): important decisions, tradeoffs, ADR-like notes
- [Descr](Descr/linker_Descr.md): what the repo or product does, domain model, conceptual descriptions
- [Research](Research/linker_Research.md): research, comparisons, external analysis
- [Setup](Setup/linker_Setup.md): local setup, environment bootstrap, install steps, and the `/Repos/` clone manifest
EOF
)"

# Linker stubs in each required area (Setup gets a specialised stub below)
for area in Architecture Decisions Descr Research; do
  linker_path="Project_Manag/Docs/$area/linker_$area.md"
  create_file "$linker_path" "$(cat <<EOF
# linker_$area

A short summary of this area: what it covers, what kinds of tasks or questions belong here, and how it fits with adjacent areas. Replace this paragraph with the actual summary.

## Docs

- [Short label that explains why to click](path/to/doc.md): description that gives the reader enough to navigate
EOF
)"
done

# Specialised linker_Setup.md — already references repos_List.md as a sibling
create_file "Project_Manag/Docs/Setup/linker_Setup.md" "$(cat <<'EOF'
# linker_Setup

Local setup, environment bootstrap, install steps, and the manifest of third-party repos that get cloned into `/Repos/` for source-code or documentation reference. Anything an engineer (or agent) needs to bring this project up on a fresh machine belongs here.

## Docs

- [repos_List](repos_List.md): manifest of third-party repos cloned into `/Repos/` (gitignored at repo root). Lists what should be present and gives the `git clone --depth 1` command for each, so the folder can be repopulated on a fresh machine.
EOF
)"

# open_Issues.md in Live_Working
create_file "Project_Manag/Live_Working/open_Issues.md" "$(cat <<'EOF'
# Open issues

Active items, in-progress work, and things to track.
EOF
)"

# Repos folder at repo root (fully gitignored) + manifest under Project_Manag/Docs/Setup/
create_dir "Repos"
create_file "Project_Manag/Docs/Setup/repos_List.md" "$(cat <<'EOF'
# repos_List

Third-party repos cloned locally for source-code or documentation reference. The clones live in `/Repos/` at the repo root, which is fully gitignored. This file lives under `Project_Manag/Docs/Setup/` so it stays under git, and is the source of truth for what should be present in `/Repos/`. To repopulate on a fresh machine, run the clone commands below.

Convention: every clone uses `git clone --depth 1` and lands inside `/Repos/`. See `.agents/skills/doc-start/References/repos_Convention.md` (if available) for the full rules.

## Repos

- (No repos cloned yet. Add entries in the format below as repos are added.)

<!--
- **<repo-name>**: one-line reason this is here (source code reference / documentation / both).
  - URL: https://github.com/<owner>/<repo>
  - Clone: `git clone --depth 1 https://github.com/<owner>/<repo> Repos/<repo-name>`
-->
EOF
)"
ensure_gitignore_line "Repos/"

# Scratch folder at repo root (fully gitignored) + three standard subfolders.
# Created unconditionally; if a project does not use them, the empty folders
# cost nothing.
create_dir "Scratch"
create_dir "Scratch/Screenshots"
create_dir "Scratch/Design"
create_dir "Scratch/Audit"
ensure_gitignore_line "Scratch/"

echo "done."
