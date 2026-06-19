# Scripts/OG_Images

`src/Scripts/OG_Images/` holds the **OG (Open Graph) manifest builder** for the social-card pipeline. This folder produces a JSON manifest describing every page that needs a card: the title and subtitle text, the variant template, the theme tokens, and the font face paths. An external Bun renderer (not in this repo) consumes that manifest after build and produces the actual `.webp` files.

This is the usage view: file map, output format, how to wire into a new project, what does NOT belong here, naming, and an add/rename checklist. For the internals — which Astro hooks fire and in what order, how the virtual `/og_Manifest.json` route works, the manifest payload shape, font path resolution against `.astro/fonts/` — see [og_Images_Pipeline.md](./og_Images_Pipeline.md).

The split between manifest building (in-repo, Astro integration) and image rendering (external Bun tool) is deliberate. The manifest builder has full access to `astro:content`, the project's font and theme registries, and the schema-typed collection entries. The renderer is heavy (Takumi layout and raster output) and standalone, so it stays out of every project's source tree.

This is the most isolated folder in `Scripts/`. Most projects can leave it alone once configured. Editable project inputs live in `src/Data/Og_Img/`; this folder should only contain the manifest machinery and validation.

## What lives here

```
OG_Images/
└── Og_Img_Gen/
    ├── manifest_Integration.js       # Astro integration. Injects the route + moves the manifest.
    ├── output_Config_Validation.js   # validates Data/Og_Img output paths and format.
    └── Manifest/
        ├── theme_Helper_Mn.js        # resolves theme tokens into the manifest payload
        ├── worker_Mn.js              # manifest route handler. Walks collections, returns JSON.
        └── Helpers/                  # route, font, asset, template, and payload helpers
```

The `_Mn` suffix on `Manifest/*` files marks them as manifest-internal modules: helpers consumed when building the manifest, not direct entry points.

`worker_Mn.js` is misleadingly named for historical reasons. It is the **manifest route handler**, not the image renderer. It exports a `GET()` function that Astro calls during build to produce the manifest JSON.

Project-owned inputs consumed by this machinery live in `src/Data/Og_Img/`:

- `og_Image_Config.ts`: template catalogue and per-collection defaults.
- `og_Asset_Config.ts`: reusable project-root-relative image/SVG asset roles plus rare route-level hero image paths that cannot live in content frontmatter.
- `og_Icon_Config.ts`: built-in or project-owned SVG icon choices for generic split-icon pages. No default icon is allowed; a collection that selects `zet_General_Split_Icon` must have an explicit mapping here.
- `og_Output_Config.js`: output format, output folder, manifest cache path, source image root.
- `og_Theme_Contract.js`: renderer token values mirrored from the CSS theme.

Before the manifest is emitted, `Manifest/Helpers/content_Context_Mn.js` normalizes OG display copy. It strips one final ASCII full stop from `content.title` and `content.description`, because OG cards read as visual titles rather than prose sentences. It keeps question marks and other meaningful punctuation.

## Output format

Partner emits **WebP**. The constant is `og_Img_Format = "webp"` in `src/Data/Og_Img/og_Output_Config.js`; the file extension constant `og_Img_Ext = "webp"` must stay in sync because the `<meta property="og:image">` URL builder consumes it.

Crawler compatibility for WebP is project-tested. The `og_Output_Config.js` comment instructs retesting LinkedIn, X, and Mastodon after any format change, with `og_Images_Investigation.md` as the historical reference for which crawlers tolerate what. Earlier versions of the pipeline emitted JPEG for crawler safety; the move to WebP came after the platform tests cleared.

`og_Img_Format` is passed to the external renderer via `manifest.meta.img_Format`, so the renderer encodes to whatever the constant says. The same value drives the `<meta property="og:image">` URL extension on the site side.

## Importing and wiring

The integration is wired in `astro.config.mjs`:

```js
// astro.config.mjs
import { generate_Og_Manifest } from "./src/Scripts/OG_Images/Og_Img_Gen/manifest_Integration.js";

export default defineConfig({
  integrations: [generate_Og_Manifest()],
});
```

Files inside `OG_Images/` import freely from each other, and may import:

- `Data/Og_Img/` — template defaults, assets, output format/paths, theme contract.
- `Utils/Common/` — language tables, locale detection.
- `Content_Schemas/` — for typing the manifest entries against collection schemas.
- `Multi_Lang_Txts/` — if OG titles/subtitles use translation tables.

They should NOT import from `Browser_Client/` (different runtime), pages, or components. The worker process has its own dependency tree — keep it lean and isolated.

## Fonts in OG images

OG cards reuse the site's font registry (`src/Data/Common/font_Config.js`) — there is no separate font ingestion. A role enters OG generation when both `og_Style` and `og_Subset` are set on its `font_Config` entry; roles without those fields are emitted to the page but skipped by the OG pipeline (the right design for a face only needed at runtime). For the resolution mechanics (how the builder globs `.astro/fonts/` and writes paths into the manifest payload) see [og_Images_Pipeline § Fonts in OG images](./og_Images_Pipeline.md). For the project-setup view of how font roles flow into every consumer (runtime, OG, Mermaid), see [`../Fonts/bootstrap.md`](../Fonts/bootstrap.md).

## What does NOT belong here

| Pattern | Where it goes |
|---|---|
| The `<meta property="og:image">` tag in HTML | A page or layout `.astro` file (in Partner: `Layouts/Base_Layouts/Init_Layout.astro`) |
| Crawler-specific HTML tweaks (e.g. fallback `<img>` tags) | A layout file, not the generator |
| Static social cards (pre-rendered, checked into `public/`) | `public/og/`. Bypassing the pipeline is fine for one-off images. |
| Font registration | `src/Data/Common/font_Config.js`. The worker reads files from `.astro/fonts/` (Astro's cache); never add bare `.woff2` paths to the worker. |
| Local font files | `src/Assets/Fonts/<Family>/`. Registered via `fontProviders.local()` in `astro.config.mjs`. Astro copies them into `.astro/fonts/` at build, and the worker picks them up from there. |
| Template defaults, output format/paths, reusable OG assets, theme tokens | `src/Data/Og_Img/`. Scripts validates and consumes them; Data owns them. |

## Naming

- The `_Mn` suffix on `Manifest/*` files is the convention indicating "manifest-internal helper". Use it for every helper in that folder.
- Other files follow improved_Camel_Snake: verb-first lowercase for actions (`manifest_Integration.js`), noun-first uppercase for feature modules.
- `worker_Mn.js` is the manifest route handler, not the image renderer. The image renderer is an external Bun tool that does not live in this repo. The handler name will inshallah be renamed to reflect its actual role once the external renderer's interface stabilises.

## Add or rename — checklist

1. **Touching the manifest shape**: edit the `GET()` handler in `worker_Mn.js` and the relevant helper in `Manifest/Helpers/`. The external renderer's expected schema is the contract; any payload change has to be coordinated with the renderer. For the current payload shape see [og_Images_Pipeline § Manifest payload](./og_Images_Pipeline.md).
2. **Changing the output format** (WebP → JPEG, etc.): update both `og_Img_Format` and `og_Img_Ext` in `src/Data/Og_Img/og_Output_Config.js`. Then retest LinkedIn, X, Mastodon, and any other crawlers in scope; document the result in `og_Images_Investigation.md`.
3. **Renaming a file in `Manifest/`**: grep `manifest_Integration.js` (the entrypoint path is hard-coded in the `injectRoute` call). The external renderer does not import from this folder, so renames here are repo-internal.
4. **Adding a new OG card template**: register the template in `src/Data/Og_Img/og_Image_Config.ts` (`og_Template_Names`) and set defaults in `collection_Template_Defaults`. Per-page exceptions use `seo.og_Img.template_Override`; the external renderer dispatches on the resolved template name.
5. **Using `zet_General_Split_Icon`**: also set the collection's icon in `src/Data/Og_Img/og_Icon_Config.ts`. Use a built-in name there, or register a `kind: 'svg'` asset in `og_Asset_Config.ts` and point to its asset id. Missing icons must hard-fail.
6. **Using legal image templates**: do not add project legal images by default. The external OG repo ships bundled neutral legal images and will use them when no legal image roles are present. Only register legal image assets in `og_Asset_Config.ts` when this project intentionally overrides the complete legal image set.
7. **Using image templates on pages with body-only images**: set both `seo.og_Img.template_Override` and `seo.og_Img.image` in the MDX frontmatter. Do not parse MDX body imports and do not let image paths select templates automatically. If an image path is present with a text-only resolved template, the manifest builder should fail so the author removes the unused field or chooses the image layout deliberately.
8. **Verifying after changes**: run `astro build` and inspect `Cache/Og_Gen/og_Manifest.json` (the moved manifest, not `dist/og_Manifest.json` which is the pre-move location). Then run the external renderer and inspect `dist/Og_Gen/` for the produced `.webp` files.
