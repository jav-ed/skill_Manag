# OG Images pipeline — deep dive

How the in-repo manifest builder actually executes during `astro build`: which Astro hooks fire and in what order, how the virtual `/og_Manifest.json` route gets injected and walked, what the manifest payload looks like on the way to the external renderer, and how font file paths are resolved against `.astro/fonts/`. Use this when modifying `manifest_Integration.js` / `worker_Mn.js`, adding a new field to the manifest payload, debugging a build that emits an empty or malformed manifest, or porting the pipeline to a new project. For the author-facing usage (file map, output format policy, wiring into `astro.config.mjs`, what NOT to put here, rename checklist) see [og_Images.md](./og_Images.md).

## How the pipeline works

1. **`astro:config:setup` fires.** `manifest_Integration.js` calls `injectRoute({ pattern: "/og_Manifest.json", entrypoint: ".../worker_Mn.js" })`, adding a virtual route to the build. The route does not exist as a file in `src/pages/`; the integration synthesises it at config time.
2. **`astro build` runs.** Astro renders every page including the injected virtual route. `worker_Mn.js`'s `GET()` handler walks every content collection, resolves each entry's OG spec (resolved template + theme + asset roles + icon choices + font paths + per-collection defaults from `Data/Og_Img/og_Image_Config.ts`), and returns the manifest JSON. Astro writes it to `dist/og_Manifest.json`.
3. **`astro:build:done` fires.** The integration moves `dist/og_Manifest.json` to `Cache/Og_Gen/og_Manifest.json`. The path comes from `Data/Og_Img/og_Output_Config.js` (`rel_Path_Json`). The move is a rename, not a copy — `dist/og_Manifest.json` no longer exists after this step.
4. **External Bun renderer runs.** Out-of-repo. Reads `Cache/Og_Gen/og_Manifest.json`, renders one `.webp` per task, writes to `dist/Og_Gen/`. Invocation is project-CI's responsibility; this repo does not ship the renderer.
5. **Crawlers fetch them.** The deployed site's `<meta property="og:image">` tags point at `Og_Gen/<page>.webp`.

The split between manifest building (Astro integration, full access to `astro:content`) and image rendering (heavy Takumi layout and raster output in an external Bun tool) is deliberate. The manifest builder needs project schemas, font registries, theme tokens; the renderer needs none of that — it just consumes JSON and writes bytes.

## Fonts in OG images

The manifest builder reads font roles from `src/Data/Common/font_Config.js`, the same registry the rendered site uses. There is no separate font ingestion for OG cards.

Two derived exports drive the link:

- `og_Font_Roles`: filtered subset of configured roles where both `og_Style` and `og_Subset` are set. Each role in this list becomes a font face the external renderer can call by name.
- `primary_Font`: the `primary` role. OG card templates use this face for body text by default; `worker_Mn.js` writes `meta.font_Display = primary_Font.family_Name` into the manifest payload.

A role enters OG generation by having both fields populated:

```js
primary: {
  family_Name: "Platypi",
  weights:     ["100 900"],
  // ...
  og_Style:    "normal",      // or "italic"
  og_Subset:   "latin",       // or "latin-ext", "cyrillic", etc.
}
```

A role without `og_Style` and `og_Subset` is registered with Astro Fonts API and emitted to the page, but `og_Font_Roles` skips it. That is the right design for a face only needed at runtime (a script-specific font like Quran, an icon font, a dev-only switcher candidate).

`worker_Mn.js` resolves the actual font files by globbing `.astro/fonts/` for `{cssVariable}-{weight}-{style}-{subset}-{contenthash}.woff2` (see `find_Astro_Cached_Font` in the file). The cache is populated by Astro Fonts API during the build and shared with the rendered HTML. Local fonts registered via `fontProviders.local()` also land in `.astro/fonts/` after build, so the same lookup works regardless of provider. The resolved paths are written into the manifest payload as `fonts.<role>.variants[].src` entries, and the external renderer reads files directly from those paths.

For the project-setup view of how font roles flow into every consumer (runtime, OG, Mermaid), see [`../Fonts/bootstrap.md`](../Fonts/bootstrap.md).

## Manifest payload — what the renderer consumes

The JSON written to `Cache/Og_Gen/og_Manifest.json` has two top-level keys:

- `meta` — globally-derived values the renderer needs once: `img_Format` (from `og_Img_Format` in `Data/Og_Img/og_Output_Config.js`), `font_Display` (the `primary_Font.family_Name`), font file paths under `fonts.<role>.variants[]`, and theme tokens resolved by `theme_Helper_Mn.js`.
- `tasks` — one entry per page that wants a card. Each task carries the resolved template name, per-page text (title + subtitle, resolved via `Multi_Lang_Txts` + collection frontmatter), the output filename (under `dist/Og_Gen/`), and any per-collection defaults pulled from `Data/Og_Img/og_Image_Config.ts`.

For `zet_General_Split_Icon`, the manifest must contain exactly one icon source: `variant_Data.general_Icon_Kind` for built-ins or `asset_Roles.general_Icon` for a project-owned SVG. `og_Icon_Config.ts` is the project-owned source of that choice. The builder and external renderer both hard-fail when the icon is missing or ambiguous.

Any change to this shape is a contract change with the external renderer. The renderer's expected schema lives outside this repo; coordinate before adding or removing fields.
