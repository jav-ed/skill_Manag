# Content Collections

Astro's content-collection system is the **source-of-truth layer for typed MDX/MD content** — blog posts, glossary entries, page bodies, anything authored as text that needs to render as a route. Collections live under `src/Content/<Collection>/`, get registered in `src/content.config.ts`, and have their frontmatter validated by Zod schemas in `src/Scripts/Content_Schemas/`.

Once a collection is registered, pages query it with `getCollection()` or `getEntry()` and Astro guarantees the entries match the schema at build time. A `.mdx` file with the wrong frontmatter shape fails the build with a path-pointing error.

This skill folder documents the **patterns** that the org's Astro projects use on top of Astro's built-in system. The Astro docs ([docs.astro.build/en/guides/content-collections](https://docs.astro.build/en/guides/content-collections/)) are the authoritative reference for the API itself — read them once. This folder is for the **organizational and multilingual conventions** the API doesn't prescribe.

## The three places content collections touch

Three folders cooperate. Knowing which lives where prevents confusion when a single collection needs work across all three.

| Folder | What lives here | When you touch it |
|---|---|---|
| `src/Content/<Collection>/` | The actual `.mdx` / `.md` source files, organized by collection | Writing content, restructuring folders, adding translations |
| `src/Scripts/Content_Schemas/` | Zod schema files — one per collection | Adding a field, changing types, introducing a new collection |
| `src/content.config.ts` | Registration — maps collection names to schemas and glob patterns | Adding a new collection or changing how the loader scopes files |

See [Scripts/Content_Schemas](../Scripts/content_Schemas.md) for the schema-file side. This folder focuses on the **content** side — folder layout, multilingual organization, identity vs URL, the workflow for adding a collection.

## The org-standard pattern (decided)

Two of the questions Astro's API leaves open have a settled answer in this org. They are not per-project decisions:

1. **Multilingual organization.** Flat language siblings — `src/Content/<Collection>/{de,en,es}/<entry>.mdx`. No `en/` primary + `translation/<lang>/` split. Caddy handles the root-path `/` → `/<main-lang>/` redirect at the reverse-proxy layer, so the folder tree stays symmetric across languages.
2. **Entry identity.** Path-based ID via `generateId: path_Based_Id` in the loader, with a separate `slug:` frontmatter field per language for the public URL. The filename stem is the **translation_key** (the stable cross-language identifier). This enables per-language URL slugs — German customers get German-spelled URLs without the entry IDs colliding.

See [Multilingual organization](./multilingual.md) and [Slugs and IDs](./slug_And_Id.md) for the full mechanics. Jav_Web's original `en/` primary + algorithmic slug-transform pattern survives only on `000_Landing_Page` post-migration (2026-05-18); it's documented as a legacy carve-out in those docs for projects that may still inherit it. Don't replicate it on new collections.

The remaining choices are genuine per-project decisions:

1. **Folder naming.** Numeric prefix (`500_Python_Series/`) or plain name (`blog/`)? Project-count threshold decides — see [Folder structure](./folder_Structure.md).
2. **Asset co-location.** Per-post asset folders (`Post_Assets/<post>/`) or central assets directory? Depends on whether entries need their own binaries.

## Before you create a collection — does this even belong in one?

The prior question is "should this value live in a collection at all?" Three homes exist for "stuff with text in it" — MDX in `src/Content/`, JS/TS in `src/Data/Common/`, and JS/TS in `src/Scripts/Multi_Lang_Txts/`. Page-bound content goes here; site-wide config and repeated UI labels do not. See [Content vs Config vs Labels](./content_Vs_Config_Vs_Labels.md) for the rule, the clear cases, and a frank look at the gray-zone cases where either home is defensible.

## Deep-dives

- [Content vs Config vs Labels](./content_Vs_Config_Vs_Labels.md): the format-decision rule before creating a collection — MDX vs `Data/Common` vs `Multi_Lang_Txts`. Three-way table, deciding question, clear cases, honest gray-zone section (opening hours, address, repeated hero phrases) with a tiebreaker. Read before adding a new collection, or when you're not sure whether a string belongs in a translation table or a content entry.
- [Folder structure](./folder_Structure.md): `src/Content/<Collection>/` layout, the numeric-prefix-vs-plain-name decision with trade-offs of each, the `Post_Assets/` co-location pattern and why the glob pattern excludes it.
- [Multilingual organization](./multilingual.md): the org-standard flat-language-siblings pattern (`{en,de,es,fr,zh}/` under each collection), the loader configuration, routing via path-split + frontmatter slug, adding translations, translation-completeness checks, and a short legacy carve-out for Jav_Web's `000_Landing_Page` (the only collection still on `en/` + `translation/<lang>/`).
- [Slugs and IDs](./slug_And_Id.md): how `path_Based_Id` decouples entry identity from URL slug, the `translation_key` concept (filename = stable cross-lang ID), the `slug:` frontmatter field, the extension for URLs whose middle segments also localize (per-language structural-segment map like Jav's `route_Slugs.ts` + `build_Per_Lang_Slug_Paths` — see [Scripts/astro_Frontmatter](../Scripts/astro_Frontmatter.md) for the helper file layout), and the landing-page legacy carve-out with migration steps.
- [Schema patterns](./schema_Patterns.md): six reusable Zod field patterns Partner has converged on — status enum, `image()` helper (with its function-form footgun), translation_key cross-references, sort_key override, surface/variant enums, and embedded structured frontmatter (FAQ + sources for JSON-LD).
- [Adding a new collection](./add_New.md): end-to-end workflow — apply the standard pattern, make the two per-project choices, write the schema, register in `content.config.ts`, create the first entry, wire to a page template via `getStaticPaths`.

## Templates — copy-paste artifacts

Code files in [`Templates/`](./Templates/) are the **canonical copy-paste sources** for the small reusable helpers the `.md` docs describe. The `.md` docs explain the *why* and the *when*; the `.ts` files are the verbatim *what*. Top-of-file comments in each template name the doc that explains it.

- [`path_Based_Id.ts`](./Templates/path_Based_Id.ts) — the 3-line `generateId` helper that decouples entry ID from frontmatter `slug:`. Copy into `src/content.config.ts` (inline) or `src/Utils/Common/` (if reused).
- [`url_Slug.ts`](./Templates/url_Slug.ts) — the Zod schema piece for kebab-case URL slugs. Copy as a sibling of `src/Scripts/Content_Schemas/schema_Seo.ts` and import from every collection schema.
- [`sister_Entries.ts`](./Templates/sister_Entries.ts) — typed helper to find every language version of a content-collection entry by filename match. Copy into `src/Utils/Common/` (or a feature folder if only one collection needs it).
- [`content.config.example.ts`](./Templates/content.config.example.ts) — starter `src/content.config.ts` with the standard pattern wired up, ready to adapt by adding schema imports and collection blocks.

## What does NOT belong here

| Pattern | Where it goes |
|---|---|
| The Zod schema file for a collection | `src/Scripts/Content_Schemas/` — see [Scripts/Content_Schemas](../Scripts/content_Schemas.md) |
| Build-time helpers that read collections (`getCollection`, slug builders) | `src/Scripts/Astro_Frontmatter/` |
| Translation strings (UI labels in 5 languages) | `src/Scripts/Multi_Lang_Txts/` — content is long-form authored text; UI labels are short keyed strings |
| Image / font binaries used by content | `src/Assets/` or per-post `Post_Assets/<post>/` (see [Folder structure](./folder_Structure.md)) |

## The leaf-layer rule still applies

Content collections themselves consume schemas (from `Scripts/Content_Schemas/`) and may reference assets (from `src/Assets/` or co-located `Post_Assets/`). They do **not** reach into pages or components — that direction is wrong. Pages read collections via `getCollection()`; collections do not know which pages render them.
