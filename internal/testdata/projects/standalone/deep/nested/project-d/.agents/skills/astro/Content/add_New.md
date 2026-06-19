# Content — Adding a New Collection

End-to-end workflow for adding a content collection to an Astro project. Most steps reference the deeper docs in this folder and in [Scripts/Content_Schemas](../Scripts/content_Schemas.md) — this file is the operating-procedure checklist that ties them together.

Walk through it top to bottom on the first new collection. After that, the steps become muscle memory.

## Step 1 — Apply the standard pattern + make the two project choices

The multilingual organization and ID strategy are decided org-wide and not per-project decisions:

- **Multilingual organization.** Flat `<lang>/` siblings under the collection. No `translation/` prefix. See [multilingual.md](./multilingual.md).
- **ID strategy.** `path_Based_Id` in the loader + per-language `slug:` field in frontmatter. See [slug_And_Id.md](./slug_And_Id.md).

Two genuine project-level choices remain:

1. **Folder name.** Numeric prefix (`500_Python_Series/`) or plain name (`blog/`)? See [folder_Structure.md](./folder_Structure.md). Use the project-count threshold to decide; match what the project already does for collections it already has.
2. **Asset co-location.** Does this collection need a `Post_Assets/` sibling for binaries (images, code samples)? Skip the folder if every entry will be pure text.

If you're inheriting an existing collection still on the pre-migration shape (`en/` + `translation/<lang>/` — in Jav_Web today, only `000_Landing_Page`), this is the moment to migrate it. See [slug_And_Id.md § Legacy carve-out](./slug_And_Id.md#legacy-carve-out--jav_webs-landing-page) for the migration steps.

## Step 2 — Write the Zod schema

In `src/Scripts/Content_Schemas/schema_<Collection>.ts` (see [Scripts/Content_Schemas](../Scripts/content_Schemas.md)):

```ts
// src/Scripts/Content_Schemas/schema_Blog.ts
import { z } from "astro/zod";

export const blog_Schema = z.object({
  slug: z.string().regex(/^[a-z0-9]+(-[a-z0-9]+)*$/, "kebab-case slug"),
  title: z.string(),
  description: z.string(),
  date: z.coerce.date(),
  status: z.enum(["draft", "published"]).default("draft"),
  // ...add fields the page template will read
});
```

Compose with the shared SEO schema when the collection's pages need meta tags:

```ts
import { seo_Schema } from "./schema_Seo";

export const blog_Schema = z.object({
  mdx_Info: z.object({
    seo: seo_Schema,
    /* ... */
  }),
});
```

Patterns vary — Jav embeds everything under `mdx_Info.*`; Partner flattens at the top. Match the project's existing collections.

## Step 3 — Register in `content.config.ts`

Add the import and the `defineCollection` block:

```ts
// src/content.config.ts
import { glob } from "astro/loaders";
import { blog_Schema } from "src/Scripts/Content_Schemas/schema_Blog";

const path_Based_Id = ({ entry }: { entry: string }) =>
  entry.replace(/\.mdx?$/, "");

const blog = defineCollection({
  loader: glob({
    pattern: "**/*.mdx",
    base: "./src/Content/blog",
    generateId: path_Based_Id,
  }),
  schema: blog_Schema,
});

export const collections = {
  // ...existing collections
  blog,
};
```

The loader's glob has two safe shapes — both are flat-language-siblings with `generateId: path_Based_Id`:

| Pattern | Glob | Trade-off |
|---|---|---|
| Narrow (explicit lang list) | `pattern: '{en,de,es,fr,zh}/**/*.{md,mdx}'` with `base: './src/Content/<Collection>'` | Anything outside listed folders is invisible — safest |
| Wide (everything under base) | `pattern: '**/*.mdx'` with `base: './src/Content/<Collection>'` | Concise — safe only if `Post_Assets/` will never hold `.mdx` |

See [folder_Structure.md](./folder_Structure.md) for the full discussion. Define the `path_Based_Id` helper once at the top of `content.config.ts` and reuse it for every collection.

## Step 4 — Create the folder + first entry

```
src/Content/<Collection>/
└── <lang>/
    └── <translation_key>.mdx
```

The first file's frontmatter must satisfy the schema. A minimal entry:

```mdx
---
slug: my-first-entry
title: My First Entry
description: A short summary for SEO and previews.
date: 2026-01-15
status: published
---

Body content in MDX. Imports work, JSX in-line, all the usual.
```

Run `bun run build` (or `bun run dev`) — Astro will fail if the frontmatter doesn't match the schema, with a path-pointing error message. Fix and retry until clean.

## Step 5 — Wire to a page template

Create a page that renders entries from the collection. For per-entry pages with language and slug in the URL:

```astro
---
// src/pages/[lang]/blog/[slug].astro
import { getCollection } from "astro:content";

export async function getStaticPaths() {
  const all_Posts = await getCollection("blog", p => p.data.status === "published");
  return all_Posts.map(post => {
    const [lang, ...rest] = post.id.split("/");
    return {
      params: { lang, slug: post.data.slug },
      props:  { post },
    };
  });
}

const { post } = Astro.props;
const { Content } = await post.render();
---

<Layout>
  <article>
    <h1>{post.data.title}</h1>
    <Content />
  </article>
</Layout>
```

For index/listing pages, query the collection and iterate; for series with prev/next nav, factor the logic into `src/Utils/<Feature>/` (see [Utils/feature_Utils](../Utils/feature_Utils.md)).

For collections whose URLs have **localized middle segments** (e.g. `/{lang}/pub/{topic}/{subtopic}/{slug}/` where `pub`, `topic`, `subtopic` all translate per language), reach for the shared `build_Per_Lang_Slug_Paths(collection_Key, leaf_Param, structural_Params)` helper instead of rolling the path/slug split inline — it reads the per-language map and emits the right segments. Jav_Web's implementation is in `src/Scripts/Astro_Frontmatter/Common/Routing/per_Lang_Slug.ts`; see [Scripts/astro_Frontmatter](../Scripts/astro_Frontmatter.md) and [slug_And_Id.md § When URL middle segments also need to localize](./slug_And_Id.md#when-url-middle-segments-also-need-to-localize). For flat `/{lang}/{collection}/{slug}/` URLs (Partner today), the inline two-step shown above is enough.

## Step 6 — Add translations

1. Create the translated file at `src/Content/<Collection>/<lang>/<translation_key>.mdx`.
2. The filename **stem** must match the existing entry — that's how the two are linked as translations.
3. Translate the frontmatter — `title`, `description`, body content, and `slug:` (each language gets its own URL slug).
4. Run `bun run build` to verify the schema validates.

A missing translation is fine — entries can ship in some languages and not others. The page template's `getStaticPaths` only generates routes for files that exist; the language switcher only shows links to existing translations.

## Step 7 — Document the collection (optional but recommended)

If the project has a docs folder, add a short entry naming the new collection, its purpose, the page template that renders it, and any custom fields the schema introduces. The catalog file in Jav_Web (`Architecture/Content_Collections/content_Collections_Catalog.md`) is auto-generated from a script — projects that follow the same pattern should hook into that.

## Add or rename — full-collection checklist

1. Project-level choices (folder naming, multilingual, ID strategy, assets) match the project's existing pattern.
2. Schema written in `src/Scripts/Content_Schemas/schema_<Collection>.ts`.
3. Collection registered in `src/content.config.ts` with the right loader glob.
4. Folder + first entry created under `src/Content/<Collection>/<lang>/`.
5. Page template added under `src/pages/` with correct `getStaticPaths`.
6. Translations added under the right path per the multilingual pattern.
7. Build runs clean (`bun run build`).
8. Spot-check a rendered page in `bun run dev`.

## Removing a collection

If a collection is decommissioned:

1. Remove the entry from `collections` export in `content.config.ts`.
2. Remove the import line.
3. Delete the page template(s) under `src/pages/`.
4. Move or archive the content files in `src/Content/<Collection>/` — don't just delete unless you're sure no one needs the history.
5. Optionally delete the schema file in `src/Scripts/Content_Schemas/` (only if it has no other consumers — schemas can be imported by frontmatter helpers for typing).
6. Grep `src/` and `Project_Manag/Docs/` for the collection name and the schema's export name.
