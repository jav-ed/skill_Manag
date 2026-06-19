# Content — Multilingual Organization

A multilingual Astro project in this org organises content into **flat language sibling folders** inside each collection. Every language has equal structural weight; no language is "primary" in the folder tree. Caddy handles the root-path `/` → `/<main-lang>/` redirect at the reverse-proxy layer, so Astro doesn't need to encode primary-vs-secondary status in folder structure.

```
src/Content/<Collection>/
├── de/                       ← German entries
│   ├── 0001_first-dental-visit-child.mdx
│   └── 0002_professional-dental-cleaning.mdx
├── en/                       ← English entries
│   ├── 0001_first-dental-visit-child.mdx
│   └── 0002_professional-dental-cleaning.mdx
└── es/                       ← Spanish entries
    └── 0001_first-dental-visit-child.mdx
```

Used by Partner. Used by every new Astro project in this org.

## What this implies

- **Translations linked by filename.** Two entries are translations of each other iff their filename stems match — the filename is the `translation_key`. The Spanish entry `0001_first-dental-visit-child.mdx` is the Spanish translation of the German file with the same stem. See [Slugs and IDs](./slug_And_Id.md) for the full pattern.
- **Per-language URL slugs.** Each language's MDX file has its own `slug:` frontmatter field — German readers get `/de/blog/erste-zahnreinigung`, English readers get `/en/blog/first-cleaning`. The folder structure doesn't constrain the URL.
- **Routing is direct.** `/{lang}/<collection-path>/<slug>` maps directly to `<lang>/<file>.mdx`. No `translation/` prefix to strip, no algorithmic transform.
- **Adding a language is mechanical.** Create `<newlang>/` under each collection, fill in entries. No structural changes elsewhere.

## The loader configuration

`src/content.config.ts` registers each collection with a loader whose glob picks up everything under the collection base:

```ts
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

The `**/*.mdx` glob is wide — it picks up every `.mdx` file under `./src/Content/blog/`. This is safe **only because** the only `.mdx` files in the tree live under language folders. If the collection has a `Post_Assets/` sibling for binaries (images, SVGs), `Post_Assets/` must not contain `.mdx` — see [folder_Structure.md](./folder_Structure.md) for the safety trade-off.

`generateId: path_Based_Id` is mandatory. Without it, Astro derives the entry ID from frontmatter `slug:`, which collides across languages when slugs are translated. See [slug_And_Id.md](./slug_And_Id.md).

## Routing in page templates

The page template's `getStaticPaths` extracts language from the file path and slug from the frontmatter:

```ts
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
```

Two-step, explicit, no algorithm. The route `/de/blog/erste-zahnreinigung` resolves to the file `src/Content/blog/de/0001_first-dental-visit-child.mdx` whose `slug:` frontmatter is `erste-zahnreinigung`.

## Adding a translation

To translate an existing entry to a new language:

1. Copy the entry file from any existing language folder into the new language's folder. **Keep the filename identical** — that's what links them as translations (the `translation_key`).
2. Translate the body content and the frontmatter fields (`title`, `description`, `slug`, etc.).
3. The `slug:` field gets the translated URL slug for that language — German readers expect German-spelled URLs.
4. Run `bun run build` to verify the schema validates.

If a translation is missing for a language, that's fine — `getStaticPaths` only generates routes for files that exist. A post can ship in some languages and not others.

## Translation completeness checks

Because translations are linked by filename, "what's missing" is a filename-set comparison across language folders. A simple build-time helper:

```ts
// pseudocode
const expected = new Set(
  (await getCollection("blog"))
    .filter(p => p.id.startsWith("en/"))
    .map(p => p.id.replace(/^en\//, ""))
);

for (const lang of ["de", "es"]) {
  for (const key of expected) {
    if (!has_file(`src/Content/blog/${lang}/${key}.mdx`)) {
      console.warn(`Missing translation: ${lang}/${key}`);
    }
  }
}
```

Use this when you need to enforce coverage. Many projects don't — incomplete translations are an accepted state, and missing entries simply don't appear on the language switcher.

## Legacy carve-out — Jav_Web's landing page

Jav_Web migrated every content collection to flat language siblings + per-language slugs on 2026-05-18, with one exception: `000_Landing_Page` still uses the older `en/` primary + `translation/{lang}/` shape:

```
src/Content/000_Landing_Page/
├── en/                       ← primary language
└── translation/
    ├── de/
    ├── es/
    ├── fr/
    └── zh/
```

The shape emerged before Caddy handled the root-path redirect at the proxy layer — Astro itself had to serve `/` and English needed structural emphasis. The landing page wasn't part of the migration sprint and still routes through `strip_Translation` from `routing_Factory.ts`.

**Do not replicate the pattern in new projects or new collections.** If you inherit a collection still on this shape — or eventually migrate the landing page — see [slug_And_Id.md § Legacy carve-out](./slug_And_Id.md#legacy-carve-out--jav_webs-landing-page) for the migration steps.
