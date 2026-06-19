# What To Precompress And Provide

Use this when deciding which map assets belong in a frontend handoff and which ones must be produced on the map server. The core rule is:

```text
Frontend-owned style JSON can be exported by the Astro repo.
Server-owned static assets should be compressed on the server from the real asset root.
```

Precompression means creating sidecar files next to an existing original:

```text
file.ext
file.ext.gz
file.ext.br
file.ext.zst
```

Never create a sidecar for a file that does not exist in the server asset root. A compressed glyph such as `0-255.pbf.br` without the matching `0-255.pbf` is a misleading asset state.

## Ownership Boundary

| Asset | Owner | Minify | Precompress | Correct handoff |
|---|---|---:|---:|---|
| MapLibre style JSON | Frontend / Astro repo | yes | yes | Export `.json`, `.json.gz`, `.json.br`, `.json.zst` from the local style builder. |
| Sprite JSON | Map server | yes | yes | Compress on the server from the real sprite JSON files. |
| Sprite PNG | Map server | no | no | Keep original PNG files; do not HTTP-precompress. |
| Glyph PBF | Map server | no | yes, if static | Compress only existing `.pbf` files in the real asset root. |
| Static vector tile files | Map server | no | optional | Only if tiles are files on disk; compress existing base files only. |
| Martin dynamic vector-tile routes | Map server | no | no sidecars | Use correct caching and dynamic compression if appropriate. |
| Catalog / TileJSON JSON | Map server | usually no | dynamic compression is enough | No frontend handoff unless these are served as static files. |

## MapLibre Glyphs Are Not Webfonts

MapLibre style JSON uses:

```json
"glyphs": "https://maps.example.com/assets/basemaps/fonts/{fontstack}/{range}.pbf"
```

Those `.pbf` files are glyph ranges used by the map renderer. `@fontsource` packages provide WOFF2 files for normal page typography; they do not replace the MapLibre glyph endpoint unless a separate glyph-generation pipeline is introduced.

So for map labels:

```text
keep .pbf
optionally add .pbf.gz
optionally add .pbf.br
optionally add .pbf.zst
```

Do not hand the map server WOFF2 files as a substitute for MapLibre glyph PBF files.

## Frontend Handoff

The Astro repo should usually provide only the style bundle:

```text
Scratch/G12_Handoff/map_Styles/
  <tenant>.public.json
  <tenant>.public.json.gz
  <tenant>.public.json.br
  <tenant>.public.json.zst
  <tenant>.local-dev.json
  <tenant>.local-dev.json.gz
  <tenant>.local-dev.json.br
  <tenant>.local-dev.json.zst
```

This is valid because the style is authored in the frontend repo during visual iteration.

## Server Static Assets

The map server should create sidecars for static sprites and glyphs from its actual directory, for example:

```text
/var/lib/maps/assets/basemaps
```

Use [Server precompression template](./Templates/Server_Precompression/linker_Server_Precompression.md) as the starting point. It only walks existing `*.json` and `*.pbf` base files, so it cannot produce extra sidecars for missing glyph ranges.

The server operator can then configure Caddy with:

```caddyfile
file_server {
	precompressed br zstd gzip
}
```

## Verification

Before copying a frontend style bundle to the server:

```bash
jq -e . Scratch/G12_Handoff/map_Styles/*.json
find Scratch/G12_Handoff/map_Styles -type f -name '*.gz' -print0 | xargs -0 -r gzip -t
find Scratch/G12_Handoff/map_Styles -type f -name '*.br' -print0 | xargs -0 -r brotli --test
find Scratch/G12_Handoff/map_Styles -type f -name '*.zst' -print0 | xargs -0 -r zstd -t
```

After running server-side static asset compression:

```bash
find /var/lib/maps/assets/basemaps -type f -name '*.gz' | wc -l
find /var/lib/maps/assets/basemaps -type f -name '*.br' | wc -l
find /var/lib/maps/assets/basemaps -type f -name '*.zst' | wc -l
find /var/lib/maps/assets/basemaps -type f -name '*.gz' -print0 | xargs -0 -r gzip -t
find /var/lib/maps/assets/basemaps -type f -name '*.br' -print0 | xargs -0 -r brotli --test
find /var/lib/maps/assets/basemaps -type f -name '*.zst' -print0 | xargs -0 -r zstd -t
```

If the server lacks the `brotli` CLI, install it before generating `.br` files. Caddy does not need the Brotli CLI to serve existing `.br` sidecars, but the server does need a Brotli tool to create them locally.

## Do Not Overdo It

Precompression helps most for static files Caddy serves directly. It is not worth complicating dynamic Martin tile routes. If a map asset is generated dynamically or proxied from a service, prefer correct caching headers and dynamic compression over generating fake sidecars.
