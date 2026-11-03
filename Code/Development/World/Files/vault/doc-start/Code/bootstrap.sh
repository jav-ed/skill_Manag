#!/usr/bin/env bash
# Stand-in for the scaffold of the real doc-start skill: the folders and .gitignore lines a new project starts with.
set -euo pipefail
project="${1:?usage: bootstrap.sh PROJECT_DIR}"
cd "$project"
mkdir -p Project_Manag/Docs Scratch Repos
printf '# doc_Start\n\nWhat this project is, and where to read next.\n' > Project_Manag/Docs/doc_Start.md
touch .gitignore
for line in 'Repos/' 'Scratch/'; do grep -qxF "$line" .gitignore || echo "$line" >> .gitignore; done
echo "  created: Project_Manag/Docs/doc_Start.md, Scratch/, Repos/, .gitignore"
