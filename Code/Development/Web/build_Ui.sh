#!/usr/bin/env bash
# Builds the web interface in Ui/ and puts the result where the Rust crate embeds it:
# Crates/Web/assets/ui/. The built files are committed, so a plain `cargo build` or `cargo install` needs no
# node; this script is for people who change Ui/.
#
# usage: build_Ui.sh            install the locked packages, type-check, build, copy
#        build_Ui.sh --check    build into a temporary folder and fail when the committed files differ
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
ui="$repo/Ui"
target="$repo/Crates/Web/assets/ui"

command -v node >/dev/null || { echo "build_Ui: node 22.12 or newer is required" >&2; exit 1; }
cd "$ui"
npm ci --no-audit --no-fund
npm run --silent check
if [ "${1:-}" = "--check" ]; then
  out="$(mktemp -d)"
  trap 'rm -rf "$out"' EXIT
  npx astro build --outDir "$out/ui" >/dev/null
  if diff -r "$out/ui" "$target" >/dev/null; then
    echo "build_Ui: the committed interface matches Ui/"
  else
    echo "build_Ui: Crates/Web/assets/ui differs from a fresh build of Ui/; run Code/Development/Web/build_Ui.sh" >&2
    diff -rq "$out/ui" "$target" >&2 || true
    exit 1
  fi
  exit 0
fi
npm run --silent build
rm -rf "$target"
cp -r "$ui/dist" "$target"
echo "build_Ui: $(find "$target" -type f | wc -l) files in $target"
