# Scripts

`src/Scripts/` is the **pipeline/infrastructure layer** of an Astro project. Everything that participates in *building or running* the site lives here: build-time data preparation, content-schema definitions, translation tables, browser-hydrated interaction code, OG image generation. Pages and components consume Scripts; Scripts may consume `Utils/` (the leaf layer); Scripts does not import back from pages.

This is the largest of the three code directories in `src/` — by file count, by surface area, and by depth of conventions. The structure splits by **purpose folder**, each with its own runtime (build vs browser vs separate worker process) and naming pattern.

## The core reason

A non-trivial Astro project produces several distinct kinds of pipeline code:

1. Helpers called inside `.astro` frontmatter at build time (route factories, lang providers, data collectors)
2. Markdown / syntax-highlighting plugins wired into root config files (`astro.config.mjs`, `ec.config.mjs`)
3. Client-side JavaScript hydrated in the browser (TOC, menus, animations)
4. Zod schemas defining the shape of every content collection
5. Translation tables for shared UI labels (footer, nav, tooltips)
6. OG image generation — usually a separate worker process triggered by a build hook

Each kind has different consumers, different runtimes, and different conventions. Mixing them into one flat folder turns the directory into rubble. The org's solution: **one purpose folder per kind**, with a known runtime and an internal sub-structure that fits its needs.

## The six purpose folders

| Subfolder | Runs when | Purpose |
|---|---|---|
| [`Astro_Frontmatter/`](./astro_Frontmatter.md) | Build time | Helpers imported by `.astro` frontmatter — routing factory, multilingual lang provider, OG image naming, per-page data collectors |
| [`Build/`](./build.md) | Build time | Assets loaded by root config files — remark/rehype plugins, syntax grammars for Expressive Code, standalone integrations (HTML minifier, meta-tag fixer) |
| [`Browser_Client/`](./browser_Client.md) | Runtime (browser) | All client-side JS — page-scoped interactions, cross-page handlers (theme, tooltip), hydrated via `<script>` tags or dynamic `import()` |
| [`Content_Schemas/`](./content_Schemas.md) | Build time | Zod schemas for every content collection, plus shared SEO metadata schema |
| [`Multi_Lang_Txts/`](./multi_Lang_Txts.md) | Build time | Translation tables keyed by lang code — UI labels, structural route labels, page copy, tooltips. Build-time validation ensures every lang is covered |
| [`OG_Images/`](./og_Images.md) | Build hook (separate process) | Manifest integration + external Bun worker for dynamic OG card generation |

Three of these (`Astro_Frontmatter/`, `Browser_Client/`, `Multi_Lang_Txts/`) split internally by page or feature. The other three stay flat or split by file-type.

## Scripts is not the only "scripts folder" — `Code/` at the repo root is different

Two folders sound similar but serve different purposes:

| Folder | What it is | When it runs |
|---|---|---|
| `src/Scripts/Build/` | Part of the Astro build pipeline — code imported by `astro.config.mjs` / `ec.config.mjs` | Every `astro build` |
| `Code/` (repo root) | Manual CLI tooling — diagram generators, doc generators, asset pre-processors | Only when invoked via `bun run <script>` |

A remark plugin goes in `src/Scripts/Build/MD_Plugins/`. A Mermaid CLI tool goes in `Code/Mermaid/`. **Never create a lowercase `scripts/` at the repo root** — the convention is capital `Code/`.

See [file_Structure.md](../Areas/file_Structure.md) for the broader `src/` overview that places `Scripts/` alongside `Utils/` and `Code/`.

## The leaf-layer rule still applies

Scripts can freely import from `Utils/` — language tables, locale detection, URL builders are all leaf-layer code that pipeline code legitimately needs. Scripts should **not** import from pages, components, or layouts. That's a smell: if a build-time helper depends on a page template, the architecture is upside-down — extract the shared logic into `Utils/` and have both call it.

## Naming

Follow the project-wide `improved_Camel_Snake` convention from the [coding skill](../../coding/languages/improved_Version.md):

- **Folders:** `PascalCase` — `Astro_Frontmatter/`, `Browser_Client/`, `OG_Images/`.
- **Files:** `snake_Case.ext`. Verb-first lowercase for action utilities (`routing_Factory.ts`, `compute_Series_Nav.ts`); noun-first uppercase for feature modules (`Blog_Linker.js`).
- **No numeric prefixes on code folders or files** — content collections use numerics (they're sort/identity keys); code files are identified by name. Jav_Web has a few historical exceptions like `schema_Entries_5xx.ts` that encode the collection's ID range — fine for that purpose but not a pattern to copy by default.

English-only folder names, even when the customer-facing route or page is in another language. The customer experience is handled by the i18n layer (slugs, translation tables); the code layer stays in English so any developer can navigate it.

## Where does a new pipeline file go?

| What you are building | Where it goes |
|---|---|
| A helper called from `.astro` frontmatter (route building, data collection) | `Scripts/Astro_Frontmatter/` |
| A remark/rehype plugin | `Scripts/Build/MD_Plugins/` |
| A custom syntax grammar for Expressive Code | `Scripts/Build/Grammars/` |
| A standalone build integration (minifier, post-processor) | `Scripts/Build/` (top level) |
| A client-side interactive feature on one page | `Scripts/Browser_Client/<EnglishPageName>/` |
| A cross-page client-side feature (theme, tooltip, lazy loader) | `Scripts/Browser_Client/Common/` |
| A Zod schema for a content collection | `Scripts/Content_Schemas/` |
| Shared UI label strings | `Scripts/Multi_Lang_Txts/Shared/` |
| Structural route labels for breadcrumbs or route-aware chrome | `Scripts/Multi_Lang_Txts/Shared/route_Txt.ts` |
| Page-specific copy or labels | `Scripts/Multi_Lang_Txts/Pages/` or `Sections/` |
| OG image generation glue | `Scripts/OG_Images/Og_Img_Gen/` |
| A manual CLI tool (not wired into `astro build`) | `Code/<Tool>/` at the repo root, NOT in `src/Scripts/` |
| A pure helper called from anywhere (data, formatter, locale logic) | `Utils/Common/` — see [Utils](../Utils/linker_Utils.md) |

## Deep-dives

- [Astro_Frontmatter](./astro_Frontmatter.md): build-time helpers imported by `.astro` frontmatter, the Common/ + per-page page-split pattern, routing factories.
- [Build](./build.md): remark/rehype plugins, syntax grammars, root-config integrations.
- [Browser_Client](./browser_Client.md): hydrated client-side JS organized by English page name, Common/ for cross-page concerns.
- [Content_Schemas](./content_Schemas.md): Zod schemas per content collection, shared SEO schema, content.config.ts wiring.
- [Multi_Lang_Txts](./multi_Lang_Txts.md): translation tables, page/section split, assert_Translation validation.
- [OG_Images (usage)](./og_Images.md): file map, WebP output, wiring into `astro.config.mjs`, what does NOT belong here, add/rename checklist.
- [OG_Images pipeline (internals)](./og_Images_Pipeline.md): hook order, virtual-route injection, manifest payload shape, font path resolution against `.astro/fonts/`. Open only when modifying the integration or worker.

## Add or rename — checklist

1. Decide which purpose folder fits — pick by runtime and consumer (frontmatter? browser? build plugin? schema? translation? OG?).
2. Inside page-splitting folders (`Astro_Frontmatter/`, `Browser_Client/`, `Multi_Lang_Txts/`), pick the English page subfolder.
3. Name the file `snake_Case.ext` — noun-first uppercase, verb-first lowercase. No numeric prefix.
4. Import via bare `src/Scripts/...` path from `.astro` frontmatter; use Vite aliases (`@components/`) for non-Scripts imports.
5. If the new code imports from a page or component, stop — it belongs somewhere else. Either lift the shared logic into `Utils/`, or rewrite so Scripts is the producer and pages are the consumer.
6. On rename, grep across `src/`, `astro.config.mjs`, `ec.config.mjs`, `package.json`, and `Project_Manag/Docs/` — `refac-cli` does not reliably update bare `src/` paths or `.astro` frontmatter.
