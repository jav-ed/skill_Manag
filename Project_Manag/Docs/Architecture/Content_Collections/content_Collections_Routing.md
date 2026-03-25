# Content Collections Routing

## Page file → collection mapping

| Page file | Collection key | URL pattern |
|---|---|---|
| `pages/[...index].astro` | `landing_Page_000` | `/` (en), `/de`, `/es`, `/fr`, `/zh` |
| `pages/[lang]/pub/coding/python-series/[...python_series].astro` | `python_Series_500` | `/{lang}/pub/coding/python-series/{slug}` |
| `pages/[lang]/pub/coding/web/[...web_entries].astro` | `web_Entries_501` | `/{lang}/pub/coding/web/{slug}` |
| `pages/[lang]/pub/justice/palestine/[...palestine_entries].astro` | `palestine_Entries_502` | `/{lang}/pub/justice/palestine/{slug}` |
| `pages/[lang]/[about].astro` | `cv_Entries_900` | `/{lang}/about` |

## Slug transform

Each page runs `getStaticPaths()` which converts a content collection file ID into a URL. The transform is applied in this order:

**1. Strip `translation/` prefix**

```
translation/de/0001_Coding_Term.mdx  →  de/0001_Coding_Term.mdx
en/0001_Coding_Term.mdx              →  (unchanged)
```

**2. Split lang from slug**

```
de/0001_Coding_Term.mdx  →  lang = "de",  slugParts = ["0001_Coding_Term.mdx"]
```

**3. Replace underscores with hyphens**

```
0001_Coding_Term.mdx  →  0001-Coding-Term.mdx
```

**4. Strip leading digits from the last slug segment**

Regex: `/^\d+-/` — removes the leading `XXXX-` sequence.

```
0001-Coding-Term.mdx  →  Coding-Term.mdx
```

**Full example:**

| File ID | lang | Final slug | Full URL |
|---|---|---|---|
| `en/0001_Coding_Term.mdx` | `en` | `coding-term` | `/en/pub/coding/python-series/coding-term` |
| `translation/de/0001_Coding_Term.mdx` | `de` | `coding-term` | `/de/pub/coding/python-series/coding-term` |

Slugs are identical across languages — only `lang` differs.

## Multilingual availability — `avail_lang_provider`

Each page file calls `avail_lang_provider(collectionKey)` to determine which languages exist for each entry. This powers the language-switcher UI.

```ts
import { avail_lang_provider } from "src/Scripts/Astro_Frontmatter/2_Common/0_Multi_Lingual/0_Multi_Ling";

const page_lang_avail = await avail_lang_provider("python_Series_500");
// later, per entry:
const all_avail_langs = page_lang_avail[python_series].languages;
```

The result is attached to `entry.data.mdx_Info.all_avail_langs` before passing to the layout.

## How `getStaticPaths` is structured

The slug transform logic lives in one place:

```
src/Scripts/Astro_Frontmatter/2_Common/1_Routing/routing_Factory.ts
```

All standard pages call the factory — only the collection key and param name differ:

```ts
import { create_Static_Paths } from 'src/Scripts/Astro_Frontmatter/2_Common/1_Routing/routing_Factory';

export const ct_Collec_Reader = "python_Series_500";  // ← change per page

export async function getStaticPaths() {
  return create_Static_Paths({ collection_Key: ct_Collec_Reader, param_Name: "python_series" });
}
```

The factory applies the full slug transform (translation strip → lang split → underscores→hyphens → leading digit removal) internally.

**Exception:** `[...index].astro` (landing page) does not use the factory — its routing is unique (English → `/`, other langs → `/de` etc.) and is handled inline in that file.
