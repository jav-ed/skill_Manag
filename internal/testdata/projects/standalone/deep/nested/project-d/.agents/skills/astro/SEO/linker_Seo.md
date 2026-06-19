# SEO

Everything an Astro project in this org needs to be discoverable and previewable: the head-tag block (title / meta / OG / Twitter / canonical / hreflang), structured data via JSON-LD, sitemap generation, and robots policy. SEO has its own folder because the surfaces touch different files (root Layout, content schemas, build-time integrations, dynamic API routes) but solve one cohesive problem — telling crawlers and social platforms what each page is.

The boundary with adjacent areas:

- **OG *image* generation** (the JPEG cards crawlers fetch) lives in [`../Scripts/og_Images.md`](../Scripts/og_Images.md). This folder covers the `<meta og:image>` *tag* — the URL the crawler reads — but not the pipeline that produces the image bytes.
- **The Zod `schema_Seo.ts`** that every content collection composes is a content-schema concern; its catalogue and import wiring live in [`../Scripts/content_Schemas.md`](../Scripts/content_Schemas.md). The actual file lives in `src/Scripts/Content_Schemas/schema_Seo.ts`. A copy-paste starter is at [`../Templates/seo_Schema.ts`](../Templates/seo_Schema.ts).
- **Per-language identity metadata** (BCP-47 codes, the `og:locale` map, the supported-langs registry) lives in [`../Data/linker_Data.md`](../Data/linker_Data.md) → Common. The SEO surfaces *consume* it; they do not define it.
- **Breadcrumb chain derivation** (URL → `{ label, href? }[]`) lives in `src/Scripts/Astro_Frontmatter/Common/Routing/breadcrumb_Chain.ts`, documented in [`../Scripts/astro_Frontmatter.md`](../Scripts/astro_Frontmatter.md) → Routing. The JSON-LD `BreadcrumbList` builder is a *consumer* of that utility, alongside the visible blog breadcrumb and the non-blog single back-link. SEO does not own chain logic — adding a new structural segment is a routing change in `route_Slugs.ts`, not an SEO change.

## Routing

- [Head tags](head_Tags.md): the block of `<head>` content the root Layout (typically `src/Layouts/Base_Layouts/Init_Layout.astro`) emits on every page — title + description, full OG set (type, url, locale, site_name, image with width/height/type/alt, locale:alternate per supported lang), Twitter card, canonical URL, hreflang + x-default, sitemap link, theme-color light+dark, favicon SVG+ICO fallback, viewport, author/referrer. Includes the `extra_Schemas` and `og_Type` prop contract the Layout exposes to pages, two patterns for hreflang (translated-slug `alternates` map vs path-prefix swap), and the order rules (charset + viewport must be first; pre-paint theme script before any rendered content). Open for any "where do I add a meta tag?" / "why is my hreflang wrong?" / "how do I get a different OG image on this page?" question.

- [JSON-LD](json_Ld.md): the structured-data subsystem — one builder per schema.org type (Article, Profile, Website, Breadcrumb, FAQPage, plus domain schemas like Person, Organization, LocalBusiness, MedicalProcedure as projects need them), the shared **identity builder** pattern (one Person/Organization function reused across every type so the author/owner stays consistent), the `og_Type` dispatcher (`jsonLd_Main.js` picks the default schema set from a single Layout prop), the `extra_Schemas` escape hatch (pages inject FAQPage from MDX frontmatter, LocalBusiness on the homepage, MedicalProcedure on service detail pages), and the `<script type="application/ld+json">` render loop. Covers the schema catalogue per page type (landing → Website+Person; blog post → Article+Breadcrumb+optional FAQ; about/CV → ProfilePage+Person; service detail → MedicalProcedure or equivalent), date normalisation, image objects with dimensions, the null-return convention for empty data, and how to add a new schema type without touching the Layout. Open when adding rich results, structured data, schema.org markup, or debugging Google's Rich Results Test.

- [Sitemap and robots](sitemap_And_Robots.md): the `@astrojs/sitemap` integration (install, `site:` option, `filter` callback, native `i18n` with `defaultLocale` and `locales`), why every project wires it (zero-cost discoverability), how to exclude internal routes (`dev/`, `proto/`, `admin/`, `live-editor/`, bare `/` deployment diagnostic) via the `filter` callback, and the two robots.txt patterns — **static** file in `public/` for simple sites, **dynamic** API route at `src/pages/robots.txt.ts` for sites with per-language Disallow rules generated from the language registry. Includes the `<link rel="sitemap">` head-tag pairing (covered in [Head tags](head_Tags.md)), the `Sitemap:` line in robots.txt, the `site:` option in `astro.config.mjs` that drives both, and a checklist for adding a new excluded route. Open for the basic install and any "which routes go in / what stays out" question.

- [Sitemap customization](sitemap_Customization.md): the custom `serialize` block layered on top of the basics — adds two things the native `i18n` option cannot. **Per-file `<lastmod>` from git timestamps** via the three-tier resolver in `Astro_Config/Last_Mod/` (slug index from frontmatter → per-folder filename match → page-file fallback for routes rendered by `src/pages/*.astro`), with the load-bearing IIFEs (`slug_Index.ts`, `git_Times.ts`), profiling via `LASTMOD_PROFILE=1`, and the hard-fail philosophy for missing files. **hreflang alternates for translated slugs** via `get_Sister_Urls(url)` — handles `/de/impressum/` ↔ `/en/legal-notice/` ↔ `/es/aviso-legal/` triples where Astro's path-suffix matching fails. Pairs with [Language modes](../Areas/Language_Modes/linker_Language_Modes.md) for the bare-root split (multi-lang root diagnostic vs single-lang public landing). Open when the build crashes with `[lastmod] … content missing`, when hreflang siblings are wrong for translated slugs, or when documenting/refactoring the `Last_Mod/` subsystem.

## When to open which

- "How do I add a meta tag / OG property / Twitter card?" → [Head tags](head_Tags.md)
- "How do I give one page a different OG image?" → [Head tags](head_Tags.md), `image` prop / per-page override
- "Hreflang is pointing at the wrong slug" → [Head tags](head_Tags.md), translated-slug `alternates` map vs path-prefix swap
- "I need to add Article / FAQ / LocalBusiness structured data" → [JSON-LD](json_Ld.md)
- "Google's Rich Results Test is flagging my schema" → [JSON-LD](json_Ld.md), per-type sections + validation
- "I want a new page type to get its own JSON-LD by default" → [JSON-LD](json_Ld.md), `og_Type` dispatcher
- "How do I exclude /admin/ from the sitemap?" → [Sitemap and robots](sitemap_And_Robots.md), `filter` callback
- "Build crashes with `[lastmod] … content missing`" → [Sitemap customization](sitemap_Customization.md), § "Per-file `lastmod` from git" + § "Hard-fail philosophy"
- "Hreflang siblings missing or wrong for translated slugs (`/de/impressum/` ↔ `/en/legal-notice/`)" → [Sitemap customization](sitemap_Customization.md), § "hreflang for translated slugs"
- "How does bare `/` differ between multi-lang and single-lang projects?" → [Language modes](../Areas/Language_Modes/linker_Language_Modes.md), then [Sitemap customization](sitemap_Customization.md)
- "Project ships in one language — how does the template adapt?" → [Single-lang mode](../Areas/Language_Modes/single_Lang_Mode.md)
- "I want robots.txt to react to the language list" → [Sitemap and robots](sitemap_And_Robots.md), dynamic API route pattern
- "Where does the Zod schema for frontmatter SEO fields live?" → [`../Scripts/content_Schemas.md`](../Scripts/content_Schemas.md), template at [`../Templates/seo_Schema.ts`](../Templates/seo_Schema.ts)
- "How is the OG image *file* generated?" → [`../Scripts/og_Images.md`](../Scripts/og_Images.md)

## Bootstrapping a new project's SEO layer

When standing up SEO on a new Astro project in this org, work in this order — each step depends on the previous:

1. **Set `site:` in `astro.config.mjs`** to the production origin (no trailing slash). Drives canonical URLs, sitemap entries, and the `site` arg passed to `robots.txt.ts`.
2. **Add the head-tag block to the root Layout** per [Head tags](head_Tags.md). Title + description + OG + Twitter + canonical + hreflang + theme-color + favicon + viewport.
3. **Drop in `schema_Seo.ts`** from [`../Templates/seo_Schema.ts`](../Templates/seo_Schema.ts) and compose it into every content-collection schema (see [Content_Schemas](../Scripts/content_Schemas.md)).
4. **Install `@astrojs/sitemap`** and wire the integration per [Sitemap and robots](sitemap_And_Robots.md).
5. **Pick a robots.txt pattern** (static or dynamic) per [Sitemap and robots](sitemap_And_Robots.md). Default to static; reach for dynamic only when per-language Disallow rules are needed.
6. **Add JSON-LD** per [JSON-LD](json_Ld.md). Start with the identity builder (Person or Organization), then the page-type schemas the project actually uses. Don't pre-build schemas the project has no page for.

Skip nothing in steps 1–5. Step 6 is incremental — add schemas as page types appear.
