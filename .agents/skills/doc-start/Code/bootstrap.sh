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

echo "doc-start bootstrap in $(pwd)"

# Required folders under Project_Manag/Docs/
for area in Architecture Decisions Descr Research; do
  create_dir "Project_Manag/Docs/$area"
done

# Live_Working folder
create_dir "Project_Manag/Live_Working"

# doc_Start.md at repo root
create_file "doc_Start.md" "$(cat <<'EOF'
# doc_Start

A short summary of the repo: what it is, what it does, and the broad shape (kind of system, primary stack, what is in scope and what is out of scope). Replace this paragraph with the actual summary.

Entry point(s): the key files an agent would need to know to orient in the code without reading anything (e.g. `src/main.js`). One line per file.

## Docs

- [Architecture](Project_Manag/Docs/Architecture/linker_Architecture.md): structure, systems, technical layout, integration boundaries
- [Decisions](Project_Manag/Docs/Decisions/linker_Decisions.md): important decisions, tradeoffs, ADR-like notes
- [Descr](Project_Manag/Docs/Descr/linker_Descr.md): what the repo or product does, domain model, conceptual descriptions
- [Research](Project_Manag/Docs/Research/linker_Research.md): research, comparisons, external analysis
EOF
)"

# Linker stubs in each required area
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

# open_Issues.md in Live_Working
create_file "Project_Manag/Live_Working/open_Issues.md" "$(cat <<'EOF'
# Open issues

Active items, in-progress work, and things to track.
EOF
)"

echo "done."
