# Scripts/Astro_Frontmatter

`src/Scripts/Astro_Frontmatter/` holds build-time helpers called from inside `.astro` frontmatter — the `---`-fenced top of an `.astro` file, which runs once on the server when the page is generated. Routing factories, multilingual lang providers, OG image naming, per-page data collectors all live here.

Anything that needs `getCollection()`, page paths, build-time SEO data, or static analysis of the content tree belongs in this folder. If the same code also runs in the browser, it doesn't belong here — that's `Browser_Client/`. If it returns a pure value with no Astro dependencies, that's `Utils/`.

## What lives here

Two distinct shapes:

| Shape | Example | Imported by |
|---|---|---|
| Cross-page utilities | `Routing/` (route-slug map + getStaticPaths helpers + canonical-link resolvers), language provider, OG image naming | Many `.astro` page templates |
| Per-page data collectors | Overview data collector, unique-overview finder | A specific page template or layout |

The cross-page utilities live in `Common/`. The per-page collectors live in a subfolder named after the **English page concept** (`Blog/`, `Writing/`, `About/`).

## Internal structure

```
Astro_Frontmatter/
├── Common/                  # cross-page utilities — always present
│   ├── Routing/             # all multilingual URL construction lives here
│   │   ├── route_Slugs.ts            # per-language structural-segment map + build_Path
│   │   ├── per_Lang_Slug.ts          # build_Per_Lang_Slug_Paths + attach_Sister_Info
│   │   ├── breadcrumb_Chain.ts       # URL → {label, href}[] chain (visible nav + JSON-LD)
│   │   ├── content_Route_Index.ts    # cached canonical-link → per-lang-URL index
│   │   └── resolve_Site_Link.ts      # unified resolver across content + top-level pages
│   ├── Og_Image/
│   │   └── og_Img_Name.js
│   └── Lang/
│       └── lang_Provider.ts
├── <PageName>/              # per-page collectors — present when a page has build-time logic
│   └── overview_Data.ts
├── Astro_Config/            # helpers imported directly by astro.config.mjs (NOT by pages)
│   ├── sitemap_Integration.ts # @astrojs/sitemap wrapper + language-mode filter
│   └── svgo_Config.ts
└── Icon_Registry/           # icon lookup table consumed by content collections at build time
    └── registry.ts
```

`Routing/` is the busiest area of `Common/` because it owns every URL the site emits. The files have separate jobs:

- **`route_Slugs.ts`** — the per-language map for structural URL segments (`pub` → `publikationen`, `digital_solutions` → `digitale-loesungen`). Plus `build_Path` / `build_Path_No_Trailing` to compose a path from a lang + segment keys + leaf, and `lookup_Route_Key` for the reverse lookup (used by `breadcrumb_Chain.ts` to recognise a localized URL segment and find its canonical key).
- **`per_Lang_Slug.ts`** — `build_Per_Lang_Slug_Paths(collection_Key, leaf_Param, structural_Params?)` is the shared `getStaticPaths` helper. It reads each entry's `mdx_Info.slug` for the URL leaf and expands the structural params per language. `attach_Sister_Info(entry, collection_Key, structural_Segments)` populates `entry.data.mdx_Info.all_avail_langs` and `full_Url_By_Lang` from a cached sister-info index, so `Init_Layout.astro` and `Lang_Selector.astro` can emit correct hreflang and language-switcher links.
- **`breadcrumb_Chain.ts`** — `build_Breadcrumb_Chain({ pathname, lang, leaf_Title? })` returns the page hierarchy as `{ label, href? }[]` where the last item has no href (it's the current page). Single source of truth for everything that needs to know "what is this page's parent chain": the blog's visible breadcrumb (`<Breadcrumb crumbs={...}/>` inside `Post_Layout.astro`), the non-blog single back-link (`<Back_Link lang={...}/>` picks `chain.at(-2)`), and the JSON-LD `BreadcrumbList` builder. Treats every URL segment that `lookup_Route_Key` recognises as a structural step; everything else is a leaf slug labelled with the optional `leaf_Title` (or the raw slug as fallback). Section labels come from the `t_Route_*` tables — no new translation strings per consumer.
- **`content_Route_Index.ts`** — builds one cached index of routed content entries, keyed twice (by canonical English link for MDX resolution, by `collection:translation_Key` for cards/nav). One scan per build, reused everywhere.
- **`resolve_Site_Link.ts`** — unified resolver that takes a canonical English target (`/contact`, `/services/preventive-care`, `/blog/professional-dental-cleaning`, `/glossary/caries`) and returns the requested language's URL. It dispatches between section indexes, top-level pages, top-level entries, and `content_Route_Index`. Unknown targets and raw localized internal paths should throw during `bun run build`; MDX link components add the authored link and current page path to the error.

The legacy `routing_Factory.ts` in this folder now exports only `strip_Translation` and is read by exactly one consumer (`[...index].astro`, the landing page). Treat it as a relic, not a routing utility.

`Astro_Config/` is the odd one — its files are imported by `astro.config.mjs`, not by `.astro` frontmatter. It lives in `Astro_Frontmatter/` because the logic is build-time Astro-aware glue, but the consumer is the config file rather than a page. Keep `astro.config.mjs` thin: for example, sitemap routing rules belong in `sitemap_Integration.ts`, where they can read `mandatory_Inp.ts` and handle multi-lang vs single-lang mode automatically.

`Icon_Registry/` only exists in projects that ship a centralized icon system (Jav_Web does; not every project will).

## When to add a new subfolder

- A new page accumulates 2+ build-time helpers that are only used by that page → make a `<PageName>/` subfolder.
- A new cross-page concern grows beyond one file → make a `Common/<Concern>/` subfolder.

Don't pre-create empty subfolders. Don't subdivide further than two levels deep — at that point the folder is probably its own purpose folder.

## Naming

- Folders: `PascalCase` named after the page concept in **English** (`Blog/`, `About/`, `Contact/`), never after a local-language slug.
- Files: `snake_Case.ext` — verb-first lowercase for action utilities (`routing_Factory.ts`, `compute_Overview.ts`), noun-first uppercase for feature modules (`Blog_Data.ts`).
- Prefer `.ts` for new code. Legacy `.js` is fine where it already exists.

## What does NOT belong here

| Pattern | Where it goes |
|---|---|
| A function that runs in the browser (touches DOM, `window`) | `Scripts/Browser_Client/` |
| A pure helper with no Astro dependencies, used in 2+ places | `Utils/` |
| A remark/rehype plugin | `Scripts/Build/MD_Plugins/` |
| Translation strings keyed by lang code | `Scripts/Multi_Lang_Txts/` |
| A Zod schema | `Scripts/Content_Schemas/` |

If the helper imports from `astro:content` or constructs page paths, this is the right folder. If it imports from `window` or operates on a DOM node, you're in the wrong folder.

## Importing

From `.astro` frontmatter, use bare `src/` paths:

```ts
---
import {
  build_Per_Lang_Slug_Paths,
  attach_Sister_Info,
} from "src/Scripts/Astro_Frontmatter/Common/Routing/per_Lang_Slug";
import { resolve_Site_Link } from "src/Scripts/Astro_Frontmatter/Common/Routing/resolve_Site_Link";
import { detect_Lang } from "src/Utils/Common/lang";
---
```

Inside `Astro_Frontmatter/` itself, use relative paths for sibling files. Imports from `Utils/` are encouraged (leaf-layer rule); imports from `Scripts/Browser_Client/` are a smell — browser code and build code should not share modules.

## Add or rename — checklist

1. Cross-page utility → `Common/<Concern>/`; per-page collector → `<PageName>/`; config-file glue → `Astro_Config/`.
2. Name the file after what it *does*, not after the page that calls it first — `routing_Factory.ts`, not `Blog_Routes.ts`.
3. If the helper grows into multiple files, lift them into a sub-subfolder named after the concern.
4. On rename, grep `.astro`, `.mjs`, `.mts`, and `.ts` files plus `astro.config.mjs` — `refac-cli` won't catch these.
