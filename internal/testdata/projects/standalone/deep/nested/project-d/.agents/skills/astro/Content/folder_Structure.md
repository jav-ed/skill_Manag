# Content — Folder Structure

`src/Content/` holds one subfolder per content collection. Each collection subfolder contains the content files plus optional asset folders. The two main project-level decisions: **how to name the collection folder** (numeric prefix or plain name) and **how to organize languages inside it** (covered separately in [multilingual.md](./multilingual.md)).

## The base shape

Every collection lives under `src/Content/<Collection>/` with content files nested by language. A minimal collection:

```
src/Content/
└── <Collection>/
    └── <lang>/
        └── <entry>.mdx
```

The full layout depends on the project's choices for numeric prefixes, multilingual organization, and asset co-location.

## Folder naming — numeric prefix vs plain name

| Pattern | Example | When it fits |
|---|---|---|
| **Numeric prefix** | `500_Python_Series/`, `101_Coding_Overview/` | Many collections with a deliberate hierarchy or display order. The number encodes role (1xx = level-1 overview, 5xx = entry series, 9xx = legal/CV). |
| **Plain name** | `blog/`, `glossary/`, `static_Pages/` | A handful of collections with no hierarchical relationship between them. Names are self-explanatory. |

The numeric-prefix pattern is the **org's documented exception** to the general "no numeric prefixes on code folders" rule (see [Areas/file_Structure.md](../Areas/file_Structure.md)). Content collection folders are allowed numerics because the number is a *stable identifier* — referenced by the schema file name, the collection variable in `content.config.ts`, and (in projects that use them) sort order. Code folders never have that justification; content folders sometimes do.

**How to decide:** count the collections.

- Up to ~5 collections, no hierarchy → plain names. The cost of remembering "500 means Python series" exceeds the benefit when there are five folders to scan.
- 10+ collections, clear hierarchy → numeric prefix. The number becomes load-bearing — it tells the reader where in the IA they are.

A clinic site with `blog`, `glossary`, `static_Pages` would never need numerics. A multi-topic personal site with 17 collections across landing / overview / entry / legal tiers needs them to stay navigable. Don't pre-commit to either; pick when the count crosses the threshold.

## Numeric prefix on entry files

If the project uses numeric prefixes for **sort order within a collection** (e.g. blog series where post `0004` follows `0003`), apply them to entry filenames:

```
500_Python_Series/en/0001_Coding_Term.mdx
500_Python_Series/en/0002_No_Fear.mdx
```

The prefix is stripped from the URL by the routing transform (see [slug_And_Id.md](./slug_And_Id.md)). Use this when blog entries have a real sequence; skip it when posts are independent and ordered by `date` frontmatter.

## Asset co-location — `Post_Assets/`

When entries need their own images, code samples, video components, or other assets, the org convention is a sibling folder named `Post_Assets/` (or similar) at the collection's root, keyed by the entry's translation_key (the stable filename stem):

```
src/Content/blog/
├── de/ en/ es/           ← content
└── Post_Assets/
    ├── 0001_first-dental-visit-child/
    │   ├── hero.avif
    │   └── illustration_b.svg
    └── 0002_professional-dental-cleaning/
        └── tooth_diagram.svg
```

Why this works:

- **Language-neutral.** A post's translations share the same images — putting assets next to one language's file would create implicit ownership; putting them in a sibling folder keeps the assets visible to all languages.
- **Discoverable.** Editing a post and editing its assets happen in the same area of the tree.
- **Not picked up by the collection.** The `defineCollection` loader's `glob` pattern is scoped to language folders only (e.g. `pattern: "{de,en,es}/**/*.mdx"`), so a stray `.mdx` file inside `Post_Assets/` would NOT become a phantom collection entry.

**Critical:** when configuring the loader, scope the glob deliberately. Two safe shapes, both used in the org:

```ts
// Narrow — explicit language-folder enumeration. Safest: anything outside the listed
// folders is invisible to the collection, regardless of file type.
loader: glob({
  pattern: '{en,de,es,fr,zh}/**/*.{md,mdx}',
  base:    './src/Content/500_Python_Series',
  generateId: path_Based_Id,
})

// Wide — everything under base. Concise. Safe ONLY when no `.md`/`.mdx` files
// will ever sit outside language folders at this level (Post_Assets/ holds
// binaries only).
loader: glob({
  pattern:    '**/*.mdx',
  base:       './src/Content/blog',
  generateId: path_Based_Id,
})
```

Both glob from a base directory whose only `.mdx`-bearing children are the language folders. The narrow form fails closed (a stray `.mdx` in `Post_Assets/` is silently ignored); the wide form fails open (the same stray file becomes a phantom collection entry). For new collections, prefer narrow if the project knows its supported languages up front (almost always), wide only if the language list is genuinely fluid. `generateId: path_Based_Id` is required for both — see [slug_And_Id.md](./slug_And_Id.md).

## What if a collection has no multilingual aspect?

A monolingual project skips the language layer entirely:

```
src/Content/
└── blog/
    └── 0001_first_post.mdx
```

`content.config.ts` scopes the glob to `**/*.mdx` (or per-entry-type), and the page template skips `lang` from `getStaticPaths`. The org's reference repos are all multilingual, so this pattern isn't deeply explored — Astro's docs cover the minimal monolingual case.

## Add or rename a collection — folder side

1. Decide: numeric prefix or plain name (use the threshold above).
2. Decide: multilingual organization (see [multilingual.md](./multilingual.md)).
3. Create `src/Content/<Collection>/<lang>/` and write the first entry file.
4. (Optional) Create `src/Content/<Collection>/Post_Assets/` if entries will have co-located assets.
5. Write the schema in `src/Scripts/Content_Schemas/schema_<Collection>.ts` — see [Scripts/Content_Schemas](../Scripts/content_Schemas.md).
6. Register in `src/content.config.ts` — see [add_New.md](./add_New.md) for the full workflow.
7. On rename, grep `content.config.ts`, every page template that references the collection key, and `Project_Manag/Docs/` — the collection name appears in many places.
