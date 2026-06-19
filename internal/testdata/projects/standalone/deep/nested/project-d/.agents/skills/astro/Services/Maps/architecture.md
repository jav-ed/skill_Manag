# Map Architecture

Map work fails when the layers are blurred together. A frontend style bug, a tile-schema mismatch, a missing sprite, and a CORS denial can all look like "the map is broken." Start by identifying which layer owns the problem.

## Ownership Boundaries

| Layer | Owner | Typical files or endpoint | What it controls |
|---|---|---|---|
| Astro component | Frontend repo | `src/Components/.../*.astro` | DOM slot, dimensions, lazy loading, labels, data attributes |
| Browser boot code | Frontend repo | `src/Scripts/Browser_Client/...` | MapLibre import, controls, markers, reduced-motion behavior |
| Style builder or JSON | Frontend during iteration, map server when frozen | local `*.ts` builder or hosted `/styles/*.json` | sources, layers, colors, labels, icons, 2.5D buildings |
| Tiles | Map server | `/t/<tenant>/<tileset>/{z}/{x}/{y}` | geographic features and tile schema |
| Glyphs | Map server | `/assets/.../fonts/{fontstack}/{range}.pbf` | text rendering for labels |
| Sprites | Map server | `/assets/.../sprites/<name>` | map-internal icons and shields |
| CORS / tenant policy | Map server | server config | which website origins can load each tenant |

UI icons outside the map, such as reset buttons or footer controls, are not sprite assets. Use the Astro project's icon policy for those. Map-internal POI icons, station symbols, and shields come from the MapLibre sprite.

## Tile Schema

The style can only reference layers and properties that exist in the vector tiles. For example:

```json
{
  "source-layer": "roads",
  "filter": ["==", ["get", "kind"], "rail"]
}
```

This works only when the vector tiles contain a source layer called `roads`, a property called `kind`, and the relevant values. Protomaps-style, OpenMapTiles-style, and custom tiles can use different layer and property names.

Before adapting a public style from elsewhere, inspect the map server's TileJSON or vector layer metadata. If the schema differs, the style may render incomplete or blank.

## Style Location

There are two valid style-location patterns:

```text
Local style object:
Astro bundle imports/builds a StyleSpecification and passes it to MapLibre.

Hosted style JSON:
Astro passes a style URL, and MapLibre fetches the JSON from the map server.
```

Use a local TypeScript style builder while the visual design is still changing. It gives fast iteration and type checking. Once the visual look is stable, export the style to JSON and have the map server serve it. That makes the map stack easier to share across sites and reduces frontend bundle responsibility.

## Public And Private Tenants

Use separate public and private tenant behavior:

```text
private local-dev tenant:
  reachable over private network
  allows localhost / 127.0.0.1 origins for development

public production tenant:
  reachable over HTTPS
  allows only the deployed website origin
  rejects localhost / 127.0.0.1
```

The two tenants should serve equivalent tiles, glyphs, sprites, and style logic. The URL host and CORS policy differ; the visual result should not.

## External Dependency Check

Before calling a map surface production-ready, verify the browser network log and built output. There should be no runtime requests to:

```text
tile.openstreetmap.org
protomaps.github.io
tiles.openfreemap.org
api.maptiler.com
api.mapbox.com
maps.googleapis.com
fonts.googleapis.com
fonts.gstatic.com
```

Internal library strings such as `mapbox` inside MapLibre shader code are not network requests. Judge by requested URLs, not string matches alone.
