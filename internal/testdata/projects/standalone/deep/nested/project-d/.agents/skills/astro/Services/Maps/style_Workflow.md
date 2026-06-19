# Map Style Workflow

Use this workflow when changing the visual map style or freezing an approved local style into hosted JSON.

## Change A Visual Style

1. Locate how the project provides the MapLibre style:
   - local TypeScript builder returning a `StyleSpecification`
   - static JSON under the repo
   - hosted style URL served by the map server
2. Keep style changes local while iterating. Do not ask the map-server operator to deploy every palette tweak.
3. Start the Astro dev server and use the private map tenant.
4. Check the map at the real page surface, not only in an isolated fixture.
5. Verify:
   - map is nonblank
   - labels render, which proves glyphs are reachable
   - map icons or shields render if the style uses sprites
   - controls are readable in the site's theme
   - marker and reset controls remain visible at relevant zooms
   - browser network requests go only to the intended private map host for map runtime assets

Keep style changes scoped. If the user asks for "better map style," adjust the style layers and palette first. Do not regenerate tiles or change the tile schema unless the requested data is missing from the tiles.

## Freeze An Approved Style

Once the visual look is approved:

1. If the project does not already have an exporter, copy [the style export template](./Templates/Style_Export/linker_Style_Export.md) into the project tooling layer:

```text
Code/Maps/export_<Project>_Map_Style.ts
```

Then keep the style builder in `Code/Maps/`, adjust the local style-builder import, tenant names, public/private tile URLs, glyph URL, sprite URL, and attribution text. Add a package script such as `maps:export-style` so the bundle can be regenerated without remembering the full path.

2. Export the current local style into two minified JSON files plus precompressed sidecars:
   - public style: public HTTPS tile/glyph/sprite URLs
   - private local-dev style: private network tile/glyph/sprite URLs
   - sidecars: `.json.gz`, `.json.br`, and `.json.zst`
3. Validate both JSON files:
   - `version` is `8`
   - `sources` point to the correct tenant
   - `glyphs` points to the map server
   - `sprite` points to the map server, without `.json` or `.png`
   - layer count is plausible and identical between public/private files
4. Keep the exported `.json` files minified. Use `jq` or editor formatting for debugging; do not hand off pretty-printed JSON as the served artifact.
5. Search the exported JSONs for unwanted hosts:

```bash
rg -n 'tile\.openstreetmap|protomaps\.github|openfreemap|maptiler|mapbox\.com|google' Scratch/G12_Handoff/map_Styles
```

6. Copy the style bundle and a handoff note to the map-server operator. Do not export sprite or glyph sidecars from the frontend repo; if static server assets need sidecars, the map-server operator should follow [Precompression handoff](./precompression_Handoff.md) and compress the real asset root.
7. Ask for served URLs, for example:

```text
https://maps.example.com/styles/<tenant>.json
http://PRIVATE-IP/styles/local-dev/<tenant>.json
```

8. Verify with `curl` and browser DevTools before switching the frontend to the served style URL.

## Switch Frontend To Hosted JSON

Only switch the frontend after the served style URL works. Once switched, the local style builder remains in `Code/Maps/` as the source for future exports; it should not be imported by browser runtime code.

The frontend can then pass a URL directly:

```ts
new maplibregl.Map({
  container,
  style: style_Url,
});
```

If attribution text needs to stay language-specific, fetch the hosted JSON in the browser, patch only the source `attribution`, and pass the patched style object to MapLibre. The visual style still comes from the server; the frontend only localizes the legal/UI label.

Keep an env or config override so preview deployments can test a replacement style without code edits. Continue to enforce the public/private split:

```text
development style URL -> private map server
production style URL  -> public map server
```

## When Not To Use This Workflow

- Missing buildings, missing U-Bahn data, missing POIs, or wrong road attributes may be tile-schema or tile-generation issues. Inspect tile metadata before changing style.
- Broken text labels are usually glyph URL, fontstack, or CORS issues.
- Broken station icons or shields are usually sprite URL, sprite JSON/PNG, or `icon-image` layer issues.
- A public `403` from localhost is expected when the public tenant correctly rejects local origins.
