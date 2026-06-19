# Scripts/Content_Schemas

`src/Scripts/Content_Schemas/` holds the **Zod schema definitions** for every content collection on the site. One file per collection, plus a shared `schema_Seo.ts` for SEO metadata that every collection's frontmatter includes. The folder is flat — no subfolders.

These schemas are imported by `src/content.config.ts` and validated by Astro at build time against the `.mdx` / `.md` files in each collection. A missing or wrongly-typed frontmatter field surfaces as a build error with the file path, which makes this folder the single source of truth for "what frontmatter must a content file have?"

## What lives here

| File | Purpose |
|---|---|
| `schema_<Collection>.ts` | The Zod schema for one content collection (Blog posts, About entries, CV sections, Legal pages, etc.) |
| `schema_Seo.ts` | Shared SEO sub-schema (title, description, required `og_Img`) — composed into every collection schema |
| `schema_Og_Img.ts` | Reusable OG image copy/template-override schema imported by `schema_Seo.ts` when the project generates OG cards |

The folder is intentionally flat. Schemas are small (~10-50 lines each), don't import each other beyond `schema_Seo.ts`, and there's no benefit to subdividing.

## Internal structure

```
Content_Schemas/
├── schema_Seo.ts             # shared SEO sub-schema
├── schema_Og_Img.ts          # generated OG image copy contract
├── schema_Landing.ts         # landing page collection
├── schema_About.ts           # About page collection
├── schema_Cv.ts              # CV collection
├── schema_Blog_Posts.ts      # blog post collection
├── schema_Overviews.ts       # overview pages collection
└── schema_Legal.ts           # legal pages collection
```

## Naming

- Files: `schema_<Collection>.ts` — verb-first lowercase (`schema_*`) since the file *describes* a schema rather than being a noun-feature. Use TypeScript (`.ts`) — Zod's inference is the whole point.
- Exports: name the schema `<collection>_Schema` for consistency (`blog_Schema`, `cv_Schema`).
- One schema per file. If a collection has variants (e.g. Jav splits overviews into multiple ID ranges), one schema file per variant is fine.

**Numeric-prefix exception.** Jav_Web uses files like `schema_Entries_5xx.ts` where the `5xx` encodes the collection's ID range (entries 500–599). This is an *acceptable* numeric prefix because it conveys real information about the collection's scope — the rule against numerics on code files explicitly excepts externally-meaningful identifiers. Don't reach for this pattern unless your project genuinely organises collections by ID range; the default is `schema_<Collection>.ts` without a number.

## Composing with the shared SEO schema

`schema_Seo.ts` defines fields like title, description, and the required `og_Img` contract. The OG sub-schema owns card copy, optional `template_Override`, and optional `image` paths for deliberately image-led cards. The canonical versions are at [`../Templates/seo_Schema.ts`](../Templates/seo_Schema.ts) and [`../Templates/og_Img_Schema.ts`](../Templates/og_Img_Schema.ts). Copy both when bootstrapping a project that generates OG cards, then register template names and collection defaults in `src/Data/Og_Img/og_Image_Config.ts` (see [`../Scripts/og_Images.md`](og_Images.md) and [`../SEO/head_Tags.md`](../SEO/head_Tags.md)).

Every collection composes it via Zod's object merging:

```ts
// schema_Blog_Posts.ts
import { z } from "astro/zod";
import { seo_Schema } from "./schema_Seo";

export const blog_Schema = z.object({
  mdx_Info: z.object({
    seo:      seo_Schema,
    title:    z.string(),
    subtitle: z.string().optional(),
    // …
  }),
  draft: z.boolean().default(false),
});
```

If a field belongs in `seo_Schema`, add it there once — every collection inherits it. If it's specific to one collection, keep it in that collection's schema.

## What does NOT belong here

| Pattern | Where it goes |
|---|---|
| A function that *reads* content collections (uses `getCollection()`) | `Scripts/Astro_Frontmatter/` — schemas describe the shape; frontmatter helpers query the data |
| The actual content files (`.mdx`, `.md`) | `src/Content/` — schemas describe; content provides the data |
| Translation strings used in content | `Scripts/Multi_Lang_Txts/` — schemas don't hold strings, they validate them |
| Helpers shared between schemas (custom validators, reusable enum lists) | OK to keep inside a schema file if used once; lift to a sibling `_helpers.ts` if used in 2+ schemas |

## Importing

Schemas are imported by `src/content.config.ts`:

```ts
// src/content.config.ts
import { blog_Schema }      from "./Scripts/Content_Schemas/schema_Blog_Posts";
import { landing_Schema }   from "./Scripts/Content_Schemas/schema_Landing";

export const collections = {
  blog:    defineCollection({ schema: blog_Schema }),
  landing: defineCollection({ schema: landing_Schema }),
};
```

Schemas may also be imported by `Astro_Frontmatter/` helpers when those helpers need to *type* their collection-reading return values. They should not be imported from pages, components, or browser code.

## Add or rename — checklist

1. One file per content collection, named `schema_<Collection>.ts`.
2. Compose `seo_Schema` from `schema_Seo.ts` into every collection schema that needs SEO fields (almost all do).
3. Use Zod's `.optional()` for genuinely optional fields, `.default(...)` for fields with sensible defaults — keep MDX files easy to author.
4. Wire into `src/content.config.ts` after creating the schema.
5. On rename, grep `src/content.config.ts` and any `Astro_Frontmatter/` files that import the schema for typing purposes.
