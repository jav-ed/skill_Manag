#!/usr/bin/env bash
set -euo pipefail

asset_root="${1:-}"

if [[ -z "$asset_root" || ! -d "$asset_root" ]]; then
  echo "Usage: $0 /path/to/assets/basemaps" >&2
  exit 2
fi

require_command() {
  local command_name="$1"

  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "Missing required command: $command_name" >&2
    exit 3
  fi
}

require_command jq
require_command gzip
require_command brotli
require_command zstd

write_compressed_sidecars() {
  local file="$1"
  local gzip_tmp="${file}.gz.tmp"
  local brotli_tmp="${file}.br.tmp"
  local zstd_tmp="${file}.zst.tmp"

  gzip -9 -c "$file" > "$gzip_tmp"
  mv "$gzip_tmp" "${file}.gz"

  brotli -f -q 11 -o "$brotli_tmp" "$file"
  mv "$brotli_tmp" "${file}.br"

  zstd -q -19 -f -o "$zstd_tmp" "$file"
  mv "$zstd_tmp" "${file}.zst"
}

minify_json_file() {
  local file="$1"
  local tmp

  tmp="$(mktemp "${file}.tmp.XXXXXX")"
  jq -c . "$file" > "$tmp"
  printf '\n' >> "$tmp"
  mv "$tmp" "$file"
}

json_count=0
pbf_count=0

while IFS= read -r -d '' file; do
  minify_json_file "$file"
  write_compressed_sidecars "$file"
  json_count=$((json_count + 1))
done < <(find "$asset_root" -type f -name '*.json' -print0)

while IFS= read -r -d '' file; do
  write_compressed_sidecars "$file"
  pbf_count=$((pbf_count + 1))
done < <(find "$asset_root" -type f -name '*.pbf' -print0)

echo "Compressed JSON files: $json_count"
echo "Compressed PBF files: $pbf_count"
echo "Skipped PNG/image assets intentionally."
