# Map Style JSON Handoff

Summary: the map visual style is approved and frozen as minified MapLibre style JSON. Please serve these files from the map server. The frontend should not switch to the served style URL until the URLs and CORS behavior are verified.

## Generated Files

```text
Scratch/G12_Handoff/map_Styles/<tenant>.public.json
Scratch/G12_Handoff/map_Styles/<tenant>.public.json.gz
Scratch/G12_Handoff/map_Styles/<tenant>.public.json.br
Scratch/G12_Handoff/map_Styles/<tenant>.public.json.zst
Scratch/G12_Handoff/map_Styles/<tenant>.local-dev.json
Scratch/G12_Handoff/map_Styles/<tenant>.local-dev.json.gz
Scratch/G12_Handoff/map_Styles/<tenant>.local-dev.json.br
Scratch/G12_Handoff/map_Styles/<tenant>.local-dev.json.zst
```

Generated from:

```text
<path-to-local-style-builder>
```

Generator:

```text
bun Code/Maps/export_Map_Style.ts
```

The exporter writes minified JSON and gzip/Brotli/Zstandard sidecars. Serve the sidecars with the normal Caddy precompressed-static setup used for public static assets.

## Public Style

Suggested public URL:

```text
https://maps.example.com/styles/<tenant>.json
```

Allowed public origin:

```text
https://<deployed-site-origin>
```

Do not allow localhost or 127.0.0.1 on the public tenant.

## Private Local-Dev Style

Suggested private URL:

```text
http://PRIVATE-MAP-IP/styles/local-dev/<tenant>.json
```

Allowed private local origins:

```text
http://localhost:<any port>
http://127.0.0.1:<any port>
```

## Expected Public Runtime URLs

```text
https://maps.example.com/t/<tenant>/<tileset>/{z}/{x}/{y}
https://maps.example.com/assets/basemaps/fonts/{fontstack}/{range}.pbf
https://maps.example.com/assets/basemaps/sprites/v4/light
```

## Verification Needed

- Public style URL returns `200` for the deployed site origin.
- Public style URL rejects localhost and 127.0.0.1 origins.
- Private style URL works from local dev over the private network.
- Browser Network panel shows no third-party map tile, glyph, sprite, or style requests.
