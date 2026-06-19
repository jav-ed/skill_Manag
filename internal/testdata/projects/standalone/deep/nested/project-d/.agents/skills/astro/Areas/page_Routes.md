# Page Routes

`src/pages/` is Astro's file-system router: every `.astro` file becomes a URL, every bracketed segment is a dynamic param. This doc covers the **page-file side** of routing — how to name files, when to use brackets, and which `getStaticPaths` helper to reach for. The content side (per-language `slug:` in MDX, the `route_Slugs` registry, the `translation_key` concept) lives in [Slugs and IDs](../Content/slug_And_Id.md); read both when designing a new route.

## The `[lang]/` prefix

In the multi-lang version, every real public route lives under `src/pages/[lang]/`. The `[lang]` bracket receives the language code (`de`, `en`, `es`, …) from `getStaticPaths`, fed by `langs_Config.supported_Langs`.

Single-lang mode is a separate mode, not just "multi-lang with one entry." In that version, `src/pages/index.astro` renders the real landing page at `/`, while most non-root routes may still be emitted under `/<main_Lang>/` until a project chooses to adopt clean single-lang URLs. See [Language modes](./Language_Modes/linker_Language_Modes.md) before changing root, sitemap, hreflang, or language-switcher behavior.

Two pages live at the root above `[lang]/`:

| File | URL it generates | Why it must stay an index |
|---|---|---|
| `src/pages/index.astro` | `/` | Astro requires `index.astro` for directory roots. Multi-lang: deployment diagnostic only, because server config must redirect `/` to `/<main_Lang>/`. Single-lang: actual landing page. |
| `src/pages/[lang]/index.astro` | `/{lang}/` | Same framework rule — the lang-root URL has nothing after it. Renders the localized landing page. |

These two `index.astro` files are the **only** ones that survive in a Partner-style repo. Every other route file uses a bracketed self-naming form.

## English-named files, per-language URL leaves

The visible URL segment (e.g. `/de/kontakt/`, `/en/contact/`, `/es/contacto/`) is **not** the file name. The file name is a stable English concept (`[contact].astro`), and the URL leaf is read from each MDX entry's `mdx_Info.slug` by `getStaticPaths`.

```
src/pages/[lang]/[contact].astro       ← file (English-named bracket)
src/Content/001_Kontakt/de/kontakt.mdx ← slug: kontakt
src/Content/001_Kontakt/en/kontakt.mdx ← slug: contact
src/Content/001_Kontakt/es/kontakt.mdx ← slug: contacto
```

URL → `/de/kontakt/`, `/en/contact/`, `/es/contacto/`. The bracket `[contact]` is the param name — `Astro.params.contact` holds whichever localized slug fired the route.

**Never** name a page file after a German URL leaf (`kontakt.astro`, `praxistour.astro`). That nails the URL to one language and makes localization impossible without renaming the file. Same goes for English leaves baked into the file name — when the URL needs to translate, the file shouldn't have to move.

The structural-segment registry `src/Scripts/Astro_Frontmatter/Common/Routing/route_Slugs.ts` is the other half of this story for segments that appear inside the URL (like `/blog/`, `/team/`, `/glossar/`). See [Slugs and IDs](../Content/slug_And_Id.md).

## List page next to item pages

For a content section with both a list page (`/de/blog/`) and item pages (`/de/blog/<post>/`), use a **bracket file** for the list and a **bracket folder** for items, both sharing the same bracket name:

```
src/pages/[lang]/[blog].astro              ← list page (URL: /de/blog/)
src/pages/[lang]/[blog]/[post_Slug].astro  ← item pages (URL: /de/blog/<slug>/)
```

Astro disambiguates by URL depth: bare `/de/blog/` hits the file, `/de/blog/<anything>/` hits the folder route. This works for `[blog]`, `[team]`, `[glossary]`, `[services]`, and any future section.

**Don't** use the equivalent `[blog]/index.astro` form for the list page. The framework accepts it, but every list page named `index.astro` is one more identical-looking tab in your editor, and the file tree no longer self-describes what each page is. `[blog].astro` says "the blog page" at a glance.

The only exception is the two framework-mandatory `index.astro` files at the root and lang-root, listed in the table above.

## Dispatcher over fan-out

When several collections share the same page template (legal pages, static editorial pages), use **one** bracket file with a merged `getStaticPaths` that returns paths for each collection, and thread the `collection_Key` through `Astro.props`. Don't create N near-identical bracket files.

Worked example — `src/pages/[lang]/[legal_page].astro` dispatches across `legal_Notice`, `privacy_Policy`, and `accessibility`:

```ts
type Legal_Collection = "legal_Notice" | "privacy_Policy" | "accessibility";

export async function getStaticPaths() {
  const collections: Legal_Collection[] = ["legal_Notice", "privacy_Policy", "accessibility"];
  const grouped = await Promise.all(
    collections.map(async (collection_Key) => {
      const paths = await build_Per_Lang_Slug_Paths(collection_Key, "legal_page");
      return paths.map((p) => ({ ...p, props: { ...p.props, collection_Key } }));
    }),
  );
  return grouped.flat();
}

const { entry, collection_Key } = Astro.props;
await attach_Sister_Info(entry, collection_Key, []);
```

`src/pages/[lang]/[static_page].astro` follows the same dispatcher pattern for the `static_Pages` collection (which holds several entries per language: `first-visit`, `practice-philosophy`, …).

## `getStaticPaths` helpers — which one when

All in `src/Scripts/Astro_Frontmatter/Common/Routing/per_Lang_Slug.ts`.

| Helper | Use it when | Worked example |
|---|---|---|
| `build_Per_Lang_Index_Paths({ <param>: "<route_key>" })` | The route has a **localized structural segment** but no per-entry leaf. One path per supported language. | `[blog].astro` → `build_Per_Lang_Index_Paths({ blog: "blog" })` |
| `build_Per_Lang_Slug_Paths(collection_Key, leaf_Param, structural_Params?)` | The route's leaf is a **per-entry slug** read from each MDX's `mdx_Info.slug`. One path per (lang, entry). | `[contact].astro` → `build_Per_Lang_Slug_Paths("kontakt", "contact")` |
| `create_Index_Paths()` (legacy) | Top-level pages with **no** per-language slug (older pattern; new routes use the bracket-file + slug form instead). | Avoid for new work; migrate when you touch a caller. |

For href resolution from outside a page (footer, nav, in-page CTAs), use:

| Helper | Use it when |
|---|---|
| `resolve_Top_Level_Page_Path({ collection_Key, lang })` | Single-entry-per-lang collection: `kontakt`, `praxistour`, `legal_Notice`, etc. Returns the localized URL for that one entry. |
| `resolve_Top_Level_Entry_Path({ collection_Key, translation_Key, lang })` | Multi-entry collection (e.g. `static_Pages`) where you need a specific entry's URL by its stable filename. |
| `build_Path({ lang, segments, leaf? })` | URLs that go through `route_Slugs`-backed structural segments (`/blog/`, `/team/`, `/glossary/`). |

Never hardcode a URL leaf in a `href`. The Footer is the reference implementation — read [Footer.astro](../../../../src/Components/Common/Footer.astro) to see all three helpers in action.

## Sitemap resolver — honors both layouts

The sitemap `<lastmod>` step resolves each URL back to a source file via `src/Scripts/Astro_Frontmatter/Astro_Config/Last_Mod/page_File.ts`. That resolver checks **both** the bracket-file form (`[lang]/[<route_key>].astro`) and the bracket+index form (`[lang]/[<route_key>]/index.astro`), in that order. So a project can sit on either layout (or migrate between them) without the sitemap breaking.

If you're moving routes around and the sitemap step throws `[lastmod] Main language content missing` for a URL that clearly has a page, the resolver is missing a tier — extend `resolve_Page_File` rather than reverting the file move.

## Adding a new route — checklist

1. Decide the route shape — top-level (`/de/kontakt/`) or section + item (`/de/blog/<post>/`).
2. If top-level: create one MDX entry per language under the matching collection (`001_Kontakt/`, `002_Praxistour/`, …). Add `mdx_Info.slug: <per-lang-leaf>` to each entry using the kebab-case `url_Slug` validator from `schema_Url_Slug.ts`.
3. If the URL has a localized structural segment: register the segment in `route_Slugs.ts` with one cell per language.
4. Create the page file at `src/pages/[lang]/[<english_name>].astro` (bracket file, not `index.astro`).
5. Pick the helper from the table above for `getStaticPaths`.
6. Read the entry via `getEntry(<collection>, \`${lang}/${translation_key}\`)`. Pass `mdx_Info` and `entry.filePath` to `Layout`.
7. For any in-component link to this route, use `resolve_Top_Level_Page_Path` (or `_Entry_Path`) — never hardcode the leaf.
8. Run `bun run build` and verify (a) the route appears under `dist/<lang>/<localized-leaf>/`, (b) the sitemap step doesn't throw, (c) `<meta property="og:image">` resolves to the generated card.

## Renaming a route — checklist

1. `git mv` the file to its new name.
2. If the file moved out of a folder (e.g. `[blog]/index.astro` → `[blog].astro`), update relative imports inside the file — typically `../../../Scripts/` → `../../Scripts/`.
3. Update the inline path comment in the file header.
4. If `page_File.ts` in the sitemap resolver doesn't already honor the new layout, extend it (see above).
5. Run `bun run build` — the page count should be unchanged.
6. Restart the dev server (`bun run dev`) — Astro doesn't HMR-refresh content schemas after the parsing step, so a schema-touching rename plus a manual restart avoids the stale-data-store error.

## Related

- [Slugs and IDs](../Content/slug_And_Id.md) — the per-language `slug:` frontmatter field, `route_Slugs.ts`, `translation_key` concept.
- [Astro_Frontmatter](../Scripts/astro_Frontmatter.md) — full reference for the routing helpers under `Scripts/Astro_Frontmatter/Common/Routing/`.
- [Sitemap and robots](../SEO/sitemap_And_Robots.md) — the `<lastmod>` resolver and how it maps URLs back to source files.
- [Content vs Config vs Labels](../Content/content_Vs_Config_Vs_Labels.md) — the prior-question decision (does this even belong as a page, or as a config value or label?).
