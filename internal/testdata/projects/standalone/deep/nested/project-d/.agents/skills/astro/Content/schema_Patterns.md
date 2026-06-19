# Content — Schema Patterns

Six reusable patterns for the Zod schemas in `src/Scripts/Content_Schemas/`. None are mandatory — a minimal collection schema can be a few `z.string()` fields. These are the patterns Partner has converged on, each solving a real problem that recurs across collections.

Pick the patterns the collection actually needs. The cost of adding them later is low: a schema change + a content edit pass. The cost of inventing your own version of one that already exists is higher — incompatible field shapes across collections make cross-collection tooling (admin views, exports, dashboards) harder.

For where schemas live, naming, and registration in `content.config.ts`, see [Scripts/Content_Schemas](../Scripts/content_Schemas.md) and [add_New.md](./add_New.md).

## 1. Status workflow — `draft` vs `published`

```ts
status: z.enum(["draft", "published"]).default("draft"),
```

New entries default to `draft`, so a half-written post can't accidentally ship. Page templates filter on it in `getStaticPaths`:

```ts
const all_Posts = await getCollection("blog", p => p.data.status === "published");
```

A drafted entry produces no route — it's invisible to the public site until the author flips the field. In `dev` mode you can choose to show drafts (skip the filter) for editing convenience; in `build` mode always filter.

**When to skip this:** legal pages, static pages, or anything where every entry is born ready-to-ship. The cost of always having `status` is one extra frontmatter field; the cost of forgetting and shipping a draft is worse.

## 2. Typed images — the `image()` schema helper

```ts
schema: ({ image }) => z.object({
  cover_image: image().optional(),
  cover_alt:   z.string().optional(),
  /* ... */
}),
```

Astro's `image()` helper turns a relative path in frontmatter (`cover_image: ./hero.avif`) into a typed `ImageMetadata` object at build time. The page template hands it directly to `<Image>` / `<Picture>` for optimisation:

```astro
{post.data.cover_image && (
  <Picture src={post.data.cover_image} alt={post.data.cover_alt ?? ""} />
)}
```

**The footgun:** `image()` is only available when `schema` is a **function** that destructures it from the argument — `schema: ({ image }) => z.object({...})`. The non-function form `schema: z.object({...})` won't have it, and a `cover_image: z.string()` field gives you a raw string the optimiser can't process. If a collection schema needs typed images, use the function form. If it doesn't, the plain form is fine.

Pair `cover_image` with a sibling `cover_alt: z.string().optional()` — alt text is per-language and belongs in frontmatter, not in the image metadata.

## 3. Cross-references via `translation_key`

Glossary entries that link to other glossary entries (a definition references another definition) need stable cross-language IDs. Use the **translation_key** (the filename stem), not the visible `slug:`:

```ts
related: z.array(z.string()).default([]),
```

Example value in MDX frontmatter:

```mdx
related:
  - karies
  - prophylaxe
```

These are filename stems, NOT URL slugs. The renderer resolves each key to "current language's slug for this entry":

```ts
const sister_Entries = await getCollection("glossary");
const links = post.data.related.map(key => {
  const target = sister_Entries.find(e =>
    e.id.startsWith(`${current_Lang}/`) &&
    e.id.endsWith(`/${key}`)
  );
  return target ? { href: `/${current_Lang}/glossar/${target.data.slug}`, label: target.data.term } : null;
}).filter(Boolean);
```

**Why filename and not slug:** slugs are translated per language, so `related: ["caries"]` doesn't resolve in German (where the URL might be `karies`). The filename stem is the stable identifier — same across all languages — so cross-references work regardless of which language the reader is viewing. See [slug_And_Id.md](./slug_And_Id.md) for the translation_key concept in full.

Missing references are silently skipped at render time (the `.find()` returns undefined). That's deliberate — a missing entry in one language is a real state (entry exists in German, not yet translated to English), and you don't want the build to fail because of it.

## 4. Sort-key overrides for non-default ordering

When alphabetic sort by the visible name produces the wrong order — typically locales with non-ASCII characters — provide an override:

```ts
sort_key: z.string().optional(),
```

Example: a German glossary where `Ä` should sort with `A`, not after `Z`:

```mdx
term: Ärztliche Untersuchung
sort_key: Aerztliche Untersuchung
```

The renderer sorts by `entry.data.sort_key ?? entry.data.term`, so entries without an override fall back to the visible name. Most entries skip the field; only the troublemakers fill it in.

**Other uses:**
- Display name differs from sort name ("The Tooth Fairy" sorted under "T" looks wrong → `sort_key: "Tooth Fairy, The"`).
- Numeric-prefix sort when the visible name has no number ("Chapter 12" sorting before "Chapter 2" → `sort_key: "12"` won't fix this; in that case use the filename `NNNN_` prefix instead).

Skip the field entirely if the collection has no sort-order surprises in any supported language.

## 5. Surface / variant enums — one template, multiple looks

When one page template needs to render visually different "surfaces" for different entries, enum frontmatter is cleaner than separate templates:

```ts
surface:      z.enum(["editorial", "legal"]).default("legal"),
show_Eyebrow: z.boolean().default(false),
show_Cta:     z.boolean().default(false),
```

In the template:

```astro
<article class:list={[
  "prose",
  post.data.surface === "editorial" ? "prose-editorial bg-cream" : "prose-block bg-white",
]}>
  {post.data.show_Eyebrow && <p class="eyebrow">{site.practice_Name}</p>}
  <h1>{post.data.title}</h1>
  <Content />
  {post.data.show_Cta && <CtaBlock />}
</article>
```

One template, two visual treatments (warm editorial surface vs. compact legal surface), plus a couple of optional sections. The alternative — separate `editorial.astro` and `legal.astro` templates — duplicates 80% of the layout for the 20% that differs.

**When to reach for this:** when 2-4 entries in the same collection need genuinely different visual treatments AND the differences are bounded (a layered enum, not freeform CSS). The pattern breaks down past 4 surfaces — at that point split into multiple collections or templates.

**Naming:** keep enum values short and English (`editorial`, `legal`, not `editorial-warm-cream-background`). Boolean flags as `show_*` for "render this section" toggles is consistent and readable.

## 6. Embedded structured frontmatter — FAQ, sources, bibliography

For entries that ship structured side-content alongside the body (FAQ blocks, references lists, glossary definitions inline), define the structure in the schema and let the template render it:

```ts
faq: z
  .object({
    items: z.array(
      z.object({
        q: z.string(),
        a: z.string(),
      }),
    ),
  })
  .optional(),

sources: z
  .array(
    z.object({
      n:     z.string(),
      title: z.string(),
      url:   z.string().url(),
    }),
  )
  .optional(),
```

MDX usage:

```mdx
---
title: First Dental Visit
faq:
  items:
    - q: What age should the first visit happen?
      a: Around age one, or within six months of the first tooth.
    - q: What should the parent bring?
      a: Vaccination records, a list of any medications, the child's favourite toy.
sources:
  - n: "1"
    title: AAPD Policy on Early Childhood Caries
    url: https://www.aapd.org/...
---

Body content here. The FAQ and sources render via layout components — the author just fills them in.
```

The template iterates over `post.data.faq?.items` and `post.data.sources`, rendering each. The schema gives you typed access (`item.q` is a string) and the layout decides how to display them (accordion, list, table).

**Why embed structured data in frontmatter instead of in the MDX body:**

- **JSON-LD emission.** A `<FaqSchema>` component reads `post.data.faq.items` and emits `<script type="application/ld+json">` for Google's FAQPage rich snippet. Authoring FAQ in MDX prose makes this impossible.
- **Type safety.** Zod validates the structure at build time — a malformed FAQ item fails the build with a clear error, not silently in production.
- **Reusable rendering.** Different layouts (blog post, glossary entry, landing page) can render the same FAQ data with their own visual treatment.

**When to skip:** if the structured content is rendered identically across all pages and never powers structured data (JSON-LD, RSS extensions), inlining it in MDX is simpler. Frontmatter shines when the same data has multiple consumers (visual rendering + schema.org emission + RSS).

## What's NOT a pattern

A few things that look schema-like but belong elsewhere:

| Pattern | Where it actually belongs |
|---|---|
| The list of supported languages | `Utils/Common/` ([Utils/common.md](../Utils/common.md)) |
| UI labels ("Read more" in 5 languages) | `Scripts/Multi_Lang_Txts/` ([Scripts/multi_Lang_Txts.md](../Scripts/multi_Lang_Txts.md)) |
| Shared SEO fields (title, description, og:image, canonical) | `schema_Seo.ts` composed into every collection ([Scripts/content_Schemas.md](../Scripts/content_Schemas.md)) |
| Author bios, brand-level constants | `Utils/Common/` — not collection schema |

If a field would be **the same value across every entry in a collection**, it's a site-wide constant, not a schema field. Lift it out.
