# Content — Slugs and IDs

Astro's content-collection system gives every entry two coordinates: an **ID** (how the build system identifies the entry internally) and a **URL slug** (what the visitor sees in the address bar). They are not the same thing, and the org's standard pattern keeps them deliberately separate.

The pattern: **path-based ID + per-language `slug:` field**. The org standard, used by Partner and (after the 2026-05-18 routing migration) by every Jav_Web collection except its landing page. Jav_Web's pre-migration default-ID + algorithmic-transform pattern survives only on `000_Landing_Page` and is documented briefly at the end as a legacy carve-out.

## Core distinction — ID vs URL slug

- **ID** is the entry's stable internal name. Used by `getEntry()`, `getCollection()`, and to identify "this same post across all languages." Two entries with the same ID would collide.
- **URL slug** is the public-facing path segment. `https://site.com/de/blog/<URL slug>`. Translated per language — German readers expect `/de/blog/erste-zahnreinigung`, English readers expect `/en/blog/first-cleaning`.

In a multilingual project the URL slug **must** be per-language. The ID **must not** be — every translation of the same post needs to share an identifier so the language switcher and translation-completeness checks can find them.

## The standard pattern — path-based ID + frontmatter `slug:`

The ID is forced to be the **file path** (without extension); the URL slug lives in a frontmatter `slug:` field, translated per language.

```ts
// src/content.config.ts
const path_Based_Id = ({ entry }: { entry: string }) =>
  entry.replace(/\.mdx?$/, "");

const blog = defineCollection({
  loader: glob({
    pattern: "**/*.mdx",
    base: "./src/Content/blog",
    generateId: path_Based_Id,
  }),
  schema: ({ image }) => z.object({
    slug: url_Slug,
    title: z.string(),
    /* ... */
  }),
});
```

**The translation_key concept.** The filename stem (no language, no extension) is the **translation_key** — the stable cross-language identifier. Two posts are translations of each other iff their filenames match across language folders:

```
src/Content/blog/
├── de/0001_first-dental-visit-child.mdx    ← translation_key: "0001_first-dental-visit-child"
├── en/0001_first-dental-visit-child.mdx    ← same translation_key — these are translations
└── es/0001_first-dental-visit-child.mdx    ← same translation_key
```

Each file's `slug:` frontmatter holds the translated URL slug:

```mdx
// de/0001_first-dental-visit-child.mdx
---
slug: erster-zahnarztbesuch-kind
title: Erster Zahnarztbesuch beim Kind
---
```

```mdx
// en/0001_first-dental-visit-child.mdx
---
slug: first-dental-visit-child
title: First Dental Visit for Your Child
---
```

**Why path-based ID is critical.** Without `generateId: path_Based_Id`, Astro's default behavior uses the frontmatter `slug:` as the ID. With per-language translated slugs, two posts could legitimately have `slug: caries` (English glossary term, German glossary term) — and the IDs would collide. Forcing the ID to be the file path means each language's file has a unique ID (`de/0001_...`, `en/0001_...`), and translations are linked by their **filename**, not their slug.

## Routing with this pattern

The page template's `getStaticPaths` reads `data.slug` for the URL leaf and derives the language from the file path:

```ts
export async function getStaticPaths() {
  const all_Posts = await getCollection("blog");
  return all_Posts.map(post => {
    const [lang, ...rest] = post.id.split("/");
    return {
      params: { lang, slug: post.data.slug },
      props:  { post },
    };
  });
}
```

The transform is **two-step** and explicit: pull `lang` from the path, pull `slug` from the frontmatter. No regex transforms, no implicit conventions.

### When URL middle segments also need to localize

The simple two-step above covers collections whose URL has only one variable leaf — `/{lang}/blog/{slug}/`. When the URL also has *structural* segments between `/{lang}/` and the leaf that should translate per language (e.g. `/{lang}/pub/{topic}/{subtopic}/{slug}/` where `pub`, `topic`, `subtopic` all localize to `publikationen`, `programmierung`, etc.), the standard pattern extends with a **per-language structural-segment map**:

- Declare each structural segment's per-language form once, keyed by a canonical English key (`pub`, `coding`, `digital_solutions`). The map is the single source of truth — every URL that touches that segment reads from it.
- A shared `getStaticPaths` helper looks up the per-language form of each segment at build time and emits one path per `<entry, lang>` pair.

Jav_Web's implementation is `src/Scripts/Astro_Frontmatter/Common/Routing/route_Slugs.ts` (the map plus `build_Path` / `build_Path_No_Trailing` / `lookup_Route_Key`) and `per_Lang_Slug.ts` (`build_Per_Lang_Slug_Paths` + `attach_Sister_Info`). See [`../Scripts/astro_Frontmatter.md`](../Scripts/astro_Frontmatter.md) for the helper-folder layout and import patterns. The same shape works in any project that has multi-segment localized URLs; smaller projects with flat `/{lang}/{collection}/{slug}/` URLs (Partner today) don't need the map and can stay with the inline two-step above.

## The slug regex

A simple kebab-case regex catches most accidents:

```ts
const url_Slug = z
  .string()
  .regex(/^[a-z0-9]+(-[a-z0-9]+)*$/, "slug must be kebab-case (lowercase, hyphens)");
```

Forbids leading/trailing hyphens, double hyphens, underscores, uppercase, special chars. Apply it to every `slug:` field across every collection — define `url_Slug` once at the top of `content.config.ts` and reuse it. The validation runs at build time; a bad slug fails the build with the file path.

## Numeric prefix on filenames

Filenames can include a `NNNN_` prefix for sort order within a collection:

```
de/0001_first-dental-visit-child.mdx
de/0002_professional-dental-cleaning.mdx
```

The prefix lives in the **filename only** — which becomes the ID and the translation_key. The visible `slug:` from frontmatter does NOT include the prefix; the public URL is the slug, not the filename. Use the prefix when entries have a real sequence (a tutorial series, an ordered glossary section); skip it when entries are independent and a `date` frontmatter sorts them naturally.

## Cross-language navigation

To render a "view this in another language" switcher, the template needs to know which translations exist for the current entry. The lookup is a filename match:

```ts
// Given current entry "blog/de/0001_<key>.mdx"
const translation_Key = post.id.split("/").slice(1).join("/"); // "0001_<key>"
const all_Posts        = await getCollection("blog");
const sister_Entries   = all_Posts.filter(p => p.id.endsWith(translation_Key));
// sister_Entries now contains [de/..., en/..., es/...] versions if they exist
```

No algorithm. No transform. Just a filename comparison.

## What does NOT belong in `slug:`

- The language prefix (`de/`, `en/`). That comes from the file path or page param.
- A leading slash. The page template adds the `/{lang}/<collection-path>/` prefix.
- Spaces, capitals, special chars. The regex enforces this.
- The file extension. The slug is the URL segment, not the filename.

## Legacy carve-out — Jav_Web's landing page

Jav_Web migrated every content collection to the standard pattern on 2026-05-18, with one carve-out: `000_Landing_Page` still uses the older `en/` primary + `translation/{lang}/` shape and Astro's default entry ID. Its routing reads `strip_Translation` from `src/Scripts/Astro_Frontmatter/Common/Routing/routing_Factory.ts` — the single function that survived the migration. The landing page is the only consumer; the rest of `routing_Factory.ts` and the old `create_Static_Paths` / `transform_Slug_Parts` helpers were deleted.

If you inherit a project still on the pre-migration shape — or want to migrate Jav's landing page later — the steps are:

1. **Move files out of `translation/`.** Lift `translation/<lang>/*` up one level to become a sibling of `en/`. Final shape: `<Collection>/{en,de,es,fr,zh}/...`.
2. **Add `mdx_Info.slug` to every entry's frontmatter.** A Zod kebab-case regex (`schema_Url_Slug.ts` in Jav, see [the regex section](#the-slug-regex) above) enforces the format. Translators can localize the slug per language at the same time.
3. **Add `generateId: path_Based_Id` to the loader** in `content.config.ts` and switch the glob to `{en,de,es,fr,zh}/**/*.{md,mdx}`.
4. **Switch the page template** to the standard `[lang, ...rest] = id.split("/")` + `data.slug` pattern shown above, or — if the URL has localized middle segments — to the structural-segment helper (`build_Per_Lang_Slug_Paths` in Jav) covered in [the structural-segments section](#when-url-middle-segments-also-need-to-localize).
5. **Update cross-language sister-entry lookups** to filename matching. Jav's `attach_Sister_Info` (in `per_Lang_Slug.ts`) is a working reference.

Migrating one collection at a time is fine — Astro's API is per-collection. After every collection migrates, the `strip_Translation` helper can come out too.
