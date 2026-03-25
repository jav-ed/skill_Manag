# Content Collections Schema

Schemas live in dedicated files under `src/Scripts/Content_Schemas/` — one file per group. `src/content.config.ts` is a thin entry point that imports them all and exports the `collections` object.

```
src/Scripts/Content_Schemas/
├── schema_Seo.ts           ← shared SEO schema (used by all collections)
├── schema_Landing_000.ts
├── schema_Overviews_1xx.ts ← 101, 102, 103
├── schema_Overviews_2xx.ts ← 200, 201, 202
├── schema_Entries_5xx.ts   ← 500, 501, 502
├── schema_Cv_900.ts
└── schema_Legal_9xx.ts     ← 990, 991
```

## Collection variable naming

Collection variable names mirror the folder name with order reversed:

| Folder | Variable |
|---|---|
| `101_Coding_Overview` | `coding_Overview_101` |
| `500_Python_Series` | `python_Series_500` |
| `900_CV` | `cv_Entries_900` |

## Schema types

There are 4 meaningfully distinct schema shapes:

| Type | Collections | Key fields |
|---|---|---|
| Landing | `000` | `mdx_Info.page_Cont` with hero lines + categories |
| Overview | `1XX`, `2XX` | `mdx_Info.seo` + `mdx_Info.page_Cont` with topic slugs |
| Blog entry | `5XX` | `mdx_Info` with `title` (transformed), `subtitle`, `date`, `seo` + `overview_finder` |
| Special | `900`, `990`, `991` | Varies — see below |

---

## SEO fields — current state and migration

All collections share the same SEO schema (`schema_Seo.ts`). It currently accepts both legacy flat fields and new nested fields:

```yaml
seo:
  meta_Title: "..."      # optional
  meta_Content: "..."    # optional

  # LEGACY — still accepted, existing MDX files use these
  og_Title: "..."        # to be migrated to og.title
  og_Content: "..."      # to be migrated to og.description

  # FUTURE — use for all new MDX files
  og:
    title: "..."
    description: "..."
    image:                 # optional
      variant: "img_Right"
      img_Rel_Path: "path/to/image.webp"
  twitter:               # optional
    site: "@handle"
    creator: "@handle"
```

Once all MDX files are migrated to the nested `og:` structure, the legacy `og_Title`/`og_Content` fields will be removed from the schema.

---

## Overview schema (1XX, 2XX)

```yaml
mdx_Info:
  seo:
    meta_Title: "..."      # optional
    meta_Content: "..."    # optional
    og_Title: "..."        # legacy — use og: {} for new content
  page_Cont:
    python: "python"       # fields vary per collection
    web: "web"
```

`page_Cont` fields differ per collection — they hold the slug identifiers for child topics. Check `src/Scripts/Content_Schemas/schema_Overviews_1xx.ts` for the exact fields of each `1XX` collection.

---

## Blog entry schema (5XX)

```yaml
mdx_Info:
  title: "your title here"       # required — transformed via divideStringIntoThreeParts
  subtitle: "..."                # optional
  desc: "..."                    # optional
  date: "09.06.2024 | 3 Dhul Hijjah 1445"  # optional, dual-calendar format
  seo:
    meta_Title: "..."            # optional
    meta_Content: "..."          # optional
    og_Title: "..."              # optional
    og_Content: "..."            # optional

image: "..."                     # optional (500, 502 only)
overview_finder: "introduction"  # optional — links entry back to its overview section
```

### `title` transform — `divideStringIntoThreeParts`

`title` is run through `divideStringIntoThreeParts` (from `src/Scripts/Astro_Frontmatter/3_Writing/0_Title_Divider`) at build time. Write the title as a plain string — the transform splits it into three display segments for the post header UI. The raw string is what you write; the component receives the split result.

### `overview_finder`

String slug that identifies which section of the parent overview page this entry belongs to. Used by the overview page to group and list entries. Must match a slug key defined in the parent overview's `page_Cont`.

---

## CV schema (900)

The most complex schema. Top-level fields:

| Field | Type | Notes |
|---|---|---|
| `head_cv_descrip` | `string` | Short tagline shown in CV header |
| `mdx_Info` | object | `title`, `subtitle`, `date`, `seo` |
| `quick_navigation` | object | Array of nav items: `href`, `icon`, `text`, `color` |
| `coding` | object | `title`, `description`, `languages[]` |
| `work_experience` | object | `title`, `experiences[]` |
| `languages` | object | `title`, `list[]` with `name`, `level` (number), `description` |
| `education` | object | `title`, `experiences[]` with optional `resources[]` |
| `internships` | object | `title`, `experiences[]` |
| `voluntary` | object | `title`, `experiences[]` |
| `certificates` | object | `title`, `list[]` |
| `publications` | object | `title`, `list[]` |
| `download_docs` | object | `title`, `download_text`, `categories[]` |
| `last_updated` | object | `text` |

`work_experience.details` supports two formats in the same array:
- Plain string: `"Did X"`
- Two-level: `{ main: "Did X", subpoints: ["detail 1", "detail 2"] }`

---

## Special pages schema (990, 991)

Minimal — title + SEO only:

```yaml
mdx_Info:
  title: "Legal Notice"
  seo:
    meta_Title: "..."    # optional
    meta_Content: "..."  # optional
    og_Title: "..."      # optional
    og_Content: "..."    # optional
```
