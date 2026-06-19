# Maps

Astro map surfaces in this org should use a self-hosted vector-map stack by default: **MapLibre GL JS** renders in the browser, a map server serves vector tiles, glyphs, sprites, and the hosted style JSON once the visual look is frozen. The Astro repo owns the component wiring and keeps project-specific style builder code in `Code/Maps/` only for future exports.

The common production goal is simple: the visitor's browser should not contact third-party tile, glyph, sprite, style, font, or map APIs. OpenStreetMap attribution still needs to be visible, but map runtime requests should go to the controlled map host.

## Runtime Model

The map stack has five separate pieces. Keep them separate when debugging:

```text
OpenStreetMap / Natural Earth data
→ generated vector tiles, usually PMTiles or MBTiles
→ tile server, for example Martin behind Caddy
→ MapLibre GL JS in the browser
→ MapLibre style JSON: sources, layers, glyphs, sprites, colors, labels
```

MapLibre style JSON is the visual recipe. It can be loaded as a hosted `.json` file, or generated locally in TypeScript and passed to `new maplibregl.Map({ style })`. During visual iteration, a local TypeScript style generator is acceptable. Once the visual look is stable, freeze it into JSON and have the map server serve it.

## Default Contract

- Local development uses a private map tenant reachable over the private network only.
- Public production uses a public tenant that only allows the deployed site origin.
- Localhost and `127.0.0.1` must not be allowed on public tenants.
- The public and private styles should have the same layers and visual logic; only tile/glyph/sprite/style URLs differ.
- Do not replace a self-hosted vector source with OpenStreetMap raster tiles, Google Maps, Mapbox, MapTiler, OpenFreeMap hosted styles, or Protomaps GitHub assets unless the user explicitly accepts the legal/privacy/product tradeoff.
- Keep visible OpenStreetMap attribution linked to `https://www.openstreetmap.org/copyright`.

## Common Workflow

For visual style changes:

1. Iterate locally in the Astro repo, usually in a TypeScript style generator.
2. Run the local dev server against private map-server endpoints.
3. Verify the browser network log has no third-party map runtime requests.
4. Freeze the approved style into public and private JSON files plus precompressed sidecars.
5. Hand the generated bundle to the map-server operator.
6. Only switch the frontend to a served style URL after the style URL, CORS, glyphs, sprites, and tiles are verified.

For tile, glyph, or sprite changes, do not edit the style first. Inspect which server asset is missing or wrong, produce or mirror the correct asset, then ask the map-server operator to serve it under the same URL contract.

## Leaves

- [Architecture](./architecture.md): the boundaries between Astro, MapLibre, style JSON, tile schema, glyphs, sprites, CORS, and the map server. Read when deciding where a map concern belongs or when a request is mixing renderer, data, and server responsibilities.
- [Style workflow](./style_Workflow.md): how to modify a map's visual style locally, verify it against private endpoints, freeze it into JSON, and hand it to the map-server operator. Read when the user says the visual look needs adjustment or is final.
- [Precompression handoff](./precompression_Handoff.md): ownership rules for precompressed map assets. Read before deciding whether the Astro repo should export an asset, or whether G12 should compress it from the real server asset root.

## Templates

- [Style export template](./Templates/Style_Export/linker_Style_Export.md): copyable TypeScript exporter plus install notes. Use when a project needs a `Code/Maps/export_<Project>_Map_Style.ts` script that writes minified public/private style JSON plus `.gz`, `.br`, and `.zst` sidecars into ignored `Scratch/` output.
- [Server precompression template](./Templates/Server_Precompression/linker_Server_Precompression.md): copyable server-side script for compressing existing static map assets in place. Use on G12 for sprite JSON and glyph PBF sidecars so no sidecar is created without a matching base file.
- [Style handoff note](./Templates/style_Handoff.md): copyable Markdown note for sending frozen style JSONs to the map-server operator.
