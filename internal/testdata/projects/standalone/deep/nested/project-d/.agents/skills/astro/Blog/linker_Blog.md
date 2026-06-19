# Blog

Everything specific to publishing long-form prose (blog posts, glossary entries, articles) in Astro projects in this org: the MDX authoring layer, the post layout's chrome (TOC variants, reading progress, scroll icons, series nav), the markdown build pipeline (Expressive Code + custom remark plugins), and the typography stack (`@tailwindcss/typography` plus per-project overrides).

Blog has its own folder because the surfaces touch many other systems (Content collections, SEO/JSON-LD, Fonts/reader-settings, Scripts/Browser_Client, Styles) but cohere around one product question: *how do you read a post?* This folder is the operating manual.

## Boundary with adjacent areas

Each adjacent area owns its piece — Blog/ explains how they cooperate:

- **Reader-settings panel** (runtime font/size/spacing on the post page) → [`../Fonts/reader_Settings.md`](../Fonts/reader_Settings.md). The Aa button, the `data-rs-*` protocol, the inline-style swap on `article.prose`. This doc points at the styling and layout integration.
- **JSON-LD on posts** (Article + BreadcrumbList + FAQPage) → [`../SEO/json_Ld.md`](../SEO/json_Ld.md). Schema-type catalogue, the `og_Type="article"` dispatcher, the `extra_Schemas` escape hatch for FAQ. This doc explains *what frontmatter feeds it*.
- **Frontmatter schemas** (the Zod shape posts must satisfy) → [`../Content/schema_Patterns.md`](../Content/schema_Patterns.md) and [`../Scripts/content_Schemas.md`](../Scripts/content_Schemas.md). The `status` enum, FAQ block with `.strict()`, embedded structured frontmatter. This doc covers the specific fields a post needs (`date`, `categories`, `subtitle`, `overview_finder`, `faq.items`).
- **Folder layout** (`src/Content/<Collection>/{lang}/NNNN_<key>.mdx`, `Post_Assets/`) → [`../Content/folder_Structure.md`](../Content/folder_Structure.md).
- **Per-language routing** (the `route_Slugs.ts` map, `build_Per_Lang_Slug_Paths` helper) → [`../Content/slug_And_Id.md`](../Content/slug_And_Id.md) and [`../Scripts/astro_Frontmatter.md`](../Scripts/astro_Frontmatter.md).
- **Browser-client scripts** for TOC observers, reader-settings, scroll-icons, work-crossfade → [`../Scripts/browser_Client.md`](../Scripts/browser_Client.md). The org convention is to put them under `Browser_Client/Blog/`; this doc explains *which scripts you need* and how they wire to the layout.
- **OG image generation** for blog cards → [`../Scripts/og_Images.md`](../Scripts/og_Images.md).

## Routing

- [MDX components](MDX_Components/linker_MDX_Components.md): the shared component-map subsystem. Routes through focused files for `mdx_Components.ts`, `Post_Img`, links/cards, structured blocks, code/tables, charts, and diagrams. Open when adding or changing any MDX-authored helper or designing a new post-only component.

- [Post covers](post_Covers.md): frontmatter cover images for blog posts. Covers use one `16:9` landscape source shared by the post page's article-column `Cover_Hero.astro` and the blog index's small thumbnail row. Open when adding cover frontmatter, generating AI cover assets, preserving the LCP loading contract, or deciding whether an index thumbnail needs a separate crop.

- [TOC and chrome](toc_And_Chrome.md): the structural shell around the article — four TOC variants (PC sticky sidebar with clerk-style SVG path animation, inline mobile, fixed mobile overlay panel, header trigger button), the IntersectionObserver active-tracking pattern, the scroll-icons pair (mobile floating pill that hides on footer-intersect, desktop sidebar version), the reading-progress bar, the series-nav end card (prev/next from `compute_Series_Nav`, View Transition `post-title` morph), the breadcrumb + draft notice + work-with-me CTA wiring inside `Post_Layout.astro`, plus the cross-references between markup (`src/Components/Posts/`), styles (`src/Styles/2_Pages/3_Posts/`), and browser scripts (`src/Scripts/Browser_Client/Blog/`). Open when adding a new chrome element, debugging an active-state observer, or designing a custom mobile reader surface.

- [Markdown pipeline](markdown_Pipeline.md): the build-time chain that processes `.md` / `.mdx` — Expressive Code for fenced code blocks (color-chips and language-badge plugins, themed via design-token CSS variables, custom Shiki grammars like Caddyfile), the two custom remark plugins (`remark_Reading_Time` writes minutes into `frontmatter.mdx_Info.reading_time`, `remark_Locale_Quotes` runs after SmartyPants to fix English curly quotes to per-locale forms `„…"` / `«…»` / `「…」`), and the wire order in `astro.config.mjs`. Diagrams sit outside this pipeline — see the two diagram docs below. Open when adding a remark/rehype plugin, configuring code blocks, or debugging why reading-time is missing.

- [Diagrams](diagrams.md) — author view: how to add or update a static SVG diagram in a post. Where `.mmd` source files live (co-located under `Post_Assets/<post-folder>/`), the `bun run mermaid:render` CLI, the optional YAML frontmatter knobs (`flowchart.nodeSpacing`, `rankSpacing`, `componentSpacing`, `padding`), the `<Mermaid_Diagram alt=… caption=…>` slot-wrapper embed pattern, what happens automatically (theme adaptation, font inheritance, Reader Settings insulation), and common pitfalls (build-first ordering, geometry re-render after a font swap, plain-text-only labels). Open when adding a diagram, updating one, or chasing a render error from the CLI.

- [Diagrams pipeline](diagrams_Pipeline.md) — deep-dive: how the `.mmd` → committed `.svg` → page render chain works internally. The three-runtime split (Bun CLI for render, Astro build for inlining, browser for paint), the file map (`Code/Mermaid/`, `Code/shared/fontMetrics.js`, `Mermaid_Diagram.astro`), the eight post-processing rewrites (root-style strip, natural width, theme-token block, edge-label padding, node widths, subgraph headers, font-family swap, `@import` strip), how `fontMetrics.measureText` uses fontkit against `.astro/fonts/`, browser-time behavior (CSS-var resolution + Reader Settings double-insulation), the build-first dependency, font-swap maintenance rules, and the historical reasons we don't ship a runtime mermaid library. Open when modifying the pipeline, adapting the pattern to a new project, or debugging an unexpected SVG output.

- [Charts](charts.md) — author view: how to add a data-driven chart (bar, line, scatter, pie) to a post via `<E_Chart>`. The two authoring patterns (inline options for tiny datasets, imported `.json` from `Post_Assets/<post-folder>/`), the JSON-serialisable constraint (no functions in `options`), the props table (`options`, `height`, `id`, `caption`), what works automatically (theme adaptation, font inheritance from `.prose`, Reader-Settings size + font reactivity, `astro:after-swap` re-init), the monochromatic default palette and how to override per-series with hex, external-label positioning that triggers the 90px reserved-margin behaviour, and common pitfalls (blank container on invalid options, font/theme not updating when the pre-paint script is missing, charts outside `article.prose` losing Reader-Settings reactivity). Open when adding a chart, updating one, or chasing an authoring error.

- [Charts pipeline](charts_Pipeline.md) — deep-dive: how `<E_Chart>` actually works under the hood. The two-runtime split (Astro frontmatter stringifies `options` into a `data-echarts-options` attribute, browser script parses it and paints via the ECharts SVG renderer), the five-dependency contract (font registry → `.prose` CSS chain → theme colour tokens → `data-theme` pre-paint script → Reader-Settings `rs-*` keys and `data-rs-*` selectors) with the failure mode for each, the `cssVarToHex` canvas trick that converts `oklch` / `color-mix` to hex (and the subtle risk if a browser ever stops accepting them as `fillStyle`), the four live-update hooks (`resize`, `MutationObserver` on `data-theme`, document `click` on `[data-rs-*]`, `astro:after-swap`), the hardcoded knobs (`CHART_BASE = 12`, `SIZE_SCALE`, `FONT_SCALE.mono = 0.9`, `EXTERNAL_LABEL_PX = 90`, monochromatic palette default), how author overrides layer onto the patched series + grid + tooltip defaults, a "what breaks if X is renamed" table covering every contract surface, an eight-step porting checklist for new projects, and the reasoning for keeping the chart browser-side rather than externalising it like the OG / Mermaid pipelines. Open when modifying `E_Chart.astro`, debugging a chart that paints with the wrong colours or font, or adapting the pattern to a new project.

- [Typography](typography.md): the `@tailwindcss/typography` plugin as the base prose layer, the `.prose` class applied to `<article>` with the responsive size ladder (`prose-base md:prose-lg xl:prose-xl`), the canonical override pattern (`prose_Style.css` in `Pages/3_Posts/`, scoped under `[data-theme] & .prose.prose { … }` so dark/light theme tokens flow), the `--tw-prose-*` variable surface (`--tw-prose-body`, `--tw-prose-headings`, `--tw-prose-links`, `--tw-prose-bold`, `--tw-prose-code`, `--tw-prose-quotes`) wired to project theme tokens like `var(--foreground)` and `color-mix(in oklch, var(--foreground) 85%, transparent)`, the body-size `clamp(1rem, 2.5vw, 1.125rem)` pattern, heading scale (h1 36px/600 weight, h2–h4 medium 500 with color-only differentiation from body), and the inline-code styling carve-out for single-backtick spans (Expressive Code only handles fenced blocks). Open when prose sizes look wrong, when adding theme tokens prose should follow, or when overriding the heading scale.

## When to open which

- "How do I add a custom component authors can use in MDX?" → [MDX component map](MDX_Components/component_Map.md)
- "How does FAQ frontmatter become a FAQPage JSON-LD (and where does the matching visible block go)?" → [Structured blocks](MDX_Components/structured_Blocks.md) + [`../SEO/json_Ld.md`](../SEO/json_Ld.md)
- "I need a new MDX helper used by only one post category" → [Component map § per-category extras](MDX_Components/component_Map.md)
- "TOC isn't highlighting the right heading on scroll" → [TOC and chrome § active observer](toc_And_Chrome.md)
- "Mobile scroll-to-top pill stays visible over the footer" → [TOC and chrome § scroll icons](toc_And_Chrome.md)
- "Adding a new series — what wires up prev/next?" → [TOC and chrome § series nav](toc_And_Chrome.md)
- "Reading-time field is missing from a post" → [Markdown pipeline § remark plugins](markdown_Pipeline.md)
- "What ratio should a blog cover or blog index thumbnail use?" → [Post covers](post_Covers.md)
- "Adding or updating a diagram in a post" → [Diagrams](diagrams.md)
- "Diagram text overflows boxes after a font swap" → [Diagrams pipeline § font-swap maintenance](diagrams_Pipeline.md)
- "Modifying the diagram post-process or adapting the pipeline to a new project" → [Diagrams pipeline](diagrams_Pipeline.md)
- "Adding a chart to a post / what props does `<E_Chart>` take" → [Charts](charts.md)
- "Chart blank or paints wrong colours after a token rename" → [Charts pipeline § what breaks if X is renamed](charts_Pipeline.md)
- "Porting `<E_Chart>` to a new project" → [Charts pipeline § porting checklist](charts_Pipeline.md)
- "Code blocks look ugly / want a new language theme" → [Markdown pipeline § Expressive Code](markdown_Pipeline.md)
- "Body text is too big / heading too small / inline code looks heavy" → [Typography](typography.md)
- "Dark theme prose has the wrong color" → [Typography § `--tw-prose-*` variables](typography.md)
- "Reader-settings Aa panel" → [`../Fonts/reader_Settings.md`](../Fonts/reader_Settings.md)
- "Article JSON-LD / Breadcrumb JSON-LD" → [`../SEO/json_Ld.md`](../SEO/json_Ld.md)
- "Where does a `.mdx` file live and what's its filename pattern?" → [`../Content/folder_Structure.md`](../Content/folder_Structure.md)

## Bootstrapping a blog on a new Astro project

When standing up a blog on a project that doesn't have one yet, work in this order — each step depends on the previous:

1. **Content collection.** Define `src/Content/blog/{en,de,…}/` with the standard pattern (path-based ID, per-language `slug:`) per [`../Content/add_New.md`](../Content/add_New.md). Schema in `src/Scripts/Content_Schemas/schema_Blog.ts` composing `seo_Schema`.
2. **Page template.** Create `src/pages/[lang]/blog/[slug].astro` with `getStaticPaths` reading the collection (inline two-step for flat URLs, `build_Per_Lang_Slug_Paths` for URLs with localised middle segments — see [`../Content/slug_And_Id.md`](../Content/slug_And_Id.md)). Render via a `Post_Layout.astro` you'll build next.
3. **Post layout shell.** `src/Layouts/Posts/Post_Layout.astro` wraps the base `Init_Layout.astro` (SEO head block) and renders `<article class="prose prose-base md:prose-lg xl:prose-xl"><slot name="main_text"/></article>`. Pass `og_Type="article"` to the base layout for Article JSON-LD via the dispatcher in [`../SEO/json_Ld.md`](../SEO/json_Ld.md).
4. **Typography base.** Add `@plugin "@tailwindcss/typography"` to the project's base CSS, then `src/Styles/2_Pages/3_Posts/prose_Style.css` overriding the `--tw-prose-*` variables to the project's theme tokens. Entry-chain it via `posts_Main.css` per [Typography](typography.md).
5. **MDX component map.** Create `src/Components/Mdx/mdx_Components.ts` with the universal map. Hand it to `<Content components={mdx_Components} />` in the page template. Start small (headings, `Inline_Link`, `Table_Wrapper`, `Post_Img`) and add components as the project authors need them — see [MDX components](MDX_Components/linker_MDX_Components.md).
6. **Markdown pipeline.** Wire Expressive Code via `astro-expressive-code` in `astro.config.mjs` (one-line integration; `ec.config.mjs` for plugins and themes). Add `remark_Reading_Time` if posts should show reading time — see [Markdown pipeline](markdown_Pipeline.md).
7. **Chrome (incremental).** Add the TOC first (most projects need at least one variant), then series-nav, scroll-icons, reading-progress, work-with-me CTA, draft notice — in whatever order the project actually needs them. See [TOC and chrome](toc_And_Chrome.md). Each one is independent; ship the minimum and grow.
8. **Reader-settings (optional polish).** Wire the Aa panel per [`../Fonts/reader_Settings.md`](../Fonts/reader_Settings.md). Project-dependent; not every blog needs it.

Steps 1–6 are required; the post is readable after step 6. Steps 7–8 are polish — they make the post a *good* read but don't change whether it ships.

## Source-of-truth references

Jav_Web has detailed internal blog docs at `Project_Manag/Docs/Architecture/Blog/` covering the full implementation. The astro skill teaches the patterns; Jav's docs are the worked example with file-by-file detail. When implementing in a new project, lean on Jav's docs for "exactly which CSS selector did they use here?" specifics, and on this folder for "should we be doing it this way at all?" decisions.
