# Scripts/Build

`src/Scripts/Build/` holds the pieces that plug **into root config files** — `astro.config.mjs` and `ec.config.mjs` for Expressive Code. Remark/rehype plugins that transform Markdown, TextMate grammars for syntax highlighting, standalone build-time integrations (HTML minifiers, meta-tag fixers). All of it runs during `astro build` (or `astro dev`) but is set up by config rather than imported from pages.

This is the most "Astro-internal" folder. A new contributor should rarely need to add a file here — most pipeline work is data collection (`Astro_Frontmatter/`) or schemas (`Content_Schemas/`). `Build/` is reserved for genuine integrations with the build tooling itself.

## What lives here

Three kinds of artifacts, with two clear subfolders plus top-level standalone files:

| Location | What goes here |
|---|---|
| `Build/MD_Plugins/` | Remark/rehype plugins — reading-time annotation, locale-aware quotes, frontmatter extraction. Wired into `astro.config.mjs` via `markdown.remarkPlugins` / `rehypePlugins`. Diagrams are not handled here; the org uses static SVG rendering via the `Code/Mermaid/` CLI instead. |
| `Build/Grammars/` | TextMate `.tmLanguage.json` syntax grammars loaded into Expressive Code / Shiki via `ec.config.mjs`. Custom languages, dialect overrides. |
| `Build/` (top level) | Standalone integrations that are neither a Markdown plugin nor a grammar — HTML minifier wrapping `@swc/html`, meta-tag requoter, similar one-off post-processors. |

If a future project introduces a different category of build artifact (e.g. PostCSS plugins as files, custom Vite plugins), give it its own subfolder under `Build/` named after the category.

## Internal structure

```
Build/
├── MD_Plugins/
│   ├── reading_Time.js
│   ├── locale_Quotes.js
│   └── frontmatter_Extract.js
├── Grammars/
│   ├── astro.tmLanguage.json
│   └── …
├── minify_Html.js              # standalone, wired into astro:build:done
└── requote_Meta_Tags.js        # standalone, wired into astro:build:done
```

## HTML minify and meta requote order

`minify_Html.js` runs first during `astro:build:done` and wraps `@swc/html`.
It preserves explicit `<head>` and `<body>` tags with
`tagOmission: "keep-head-and-body"` because social crawlers need the document
head to locate OG metadata. SWC may still remove quotes from valid HTML
attribute values such as `property=og:image` or
`content=https://example.com/card.webp`.

`requote_Meta_Tags.js` runs after minification. Its scope is deliberately
narrow: for each built HTML file, it inspects only the `<head>...</head>` block,
then only `<meta ...>` tags inside that head. It re-quotes unquoted attributes
for crawler-sensitive metadata and skips anything already inside single or
double quotes. Do not widen this pass to the body; metadata that crawlers parse
belongs in the document head, and body-wide post-processing can corrupt valid
content such as meta-refresh values.

## Naming

- Folders: `PascalCase` named after the *kind* of artifact (`MD_Plugins/`, `Grammars/`).
- Files: `snake_Case.ext`. Verb-first lowercase for action plugins (`minify_Html.js`, `requote_Meta_Tags.js`). Noun-first for data-shaped files (TextMate grammar JSON files keep their canonical lowercase names, since the language ecosystem expects `astro.tmLanguage.json` etc.).
- Top-of-file comment explaining WHY the file exists and which config it's wired into. Remark plugins are easy to lose track of when stripped from context — a header comment naming the entry config saves later spelunking.

## What does NOT belong here

| Pattern | Where it goes |
|---|---|
| A helper called from `.astro` frontmatter | `Scripts/Astro_Frontmatter/` |
| A Zod content schema | `Scripts/Content_Schemas/` |
| Code that runs in the browser | `Scripts/Browser_Client/` |
| OG image generation | `Scripts/OG_Images/` (it spawns its own external worker) |
| A manual CLI tool not wired into `astro build` | `Code/<Tool>/` at the repo root |

The dividing line: **does the code participate in `astro build`?** If yes, and it's a plugin/integration rather than a frontmatter helper or schema, it goes here. If no, it goes in repo-root `Code/`.

## Importing

`Build/` files are imported by root config files, not by pages or components. Typical wiring:

```js
// astro.config.mjs
import { reading_Time } from "./src/Scripts/Build/MD_Plugins/reading_Time.js";
import { minify_Html } from "./src/Scripts/Build/minify_Html.js";

export default defineConfig({
  markdown: { remarkPlugins: [reading_Time] },
  integrations: [minify_Html()],
});
```

```js
// ec.config.mjs
import astroGrammar from "./src/Scripts/Build/Grammars/astro.tmLanguage.json" assert { type: "json" };
```

Internally, `Build/` files may import from `Utils/` (e.g. a locale plugin reads `SUPPORTED_LANGS` from `Utils/Common/`). They should not import from `Astro_Frontmatter/` or `Browser_Client/` — different layers.

## Why "Build" and not "Plugins"

The folder is named after *when* the code runs (build time), not *what kind* of artifact it is (plugins). Several non-plugin things live here — grammars, integrations, post-processors. Naming by runtime keeps the folder coherent as new build-time artifacts arrive.

## Add or rename — checklist

1. Pick the subfolder by artifact kind — `MD_Plugins/`, `Grammars/`, or a new one if a new kind is genuinely needed. Standalone integrations sit at the top level of `Build/`.
2. Name `snake_Case.ext`, verb-first for action plugins.
3. Add a top-of-file comment naming WHICH config file imports it and WHY (without this, plugins become orphans in code review).
4. Wire into the relevant config file — `astro.config.mjs` for remark/rehype/integrations, `ec.config.mjs` for grammars.
5. On rename, grep `astro.config.mjs`, `ec.config.mjs`, and `package.json` for the file path.
