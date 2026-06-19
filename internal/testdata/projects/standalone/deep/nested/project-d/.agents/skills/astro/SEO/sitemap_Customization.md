# Sitemap customization — `serialize()`

[Sitemap and robots](sitemap_And_Robots.md) covers the basic `@astrojs/sitemap` install, the `filter` callback, and the native `i18n` option for hreflang. This doc covers the **custom `serialize` block** that adds two things the native i18n option cannot:

1. **Per-file `<lastmod>` from git timestamps** — real crawl-priority signals instead of build time / nothing.
2. **hreflang alternates for translated slugs** — the case where `/de/impressum/` and `/en/legal-notice/` are the same page but the leaf segment localizes.

If your project keeps slugs identical across languages and doesn't care about per-URL `lastmod`, you can skip this doc — the native `i18n` option alone is enough. Most org projects want at least the lastmod, and any project with translated slugs needs the hreflang piece.

## Shape

```js
// astro.config.mjs
import { sitemap_Integration } from "./src/Scripts/Astro_Frontmatter/Astro_Config/sitemap_Integration";

integrations: [
  sitemap_Integration(),
]
```

The helper owns the actual customization:

```ts
// src/Scripts/Astro_Frontmatter/Astro_Config/sitemap_Integration.ts
import sitemap from "@astrojs/sitemap";
import { get_Sister_Urls, resolve_Last_Mod } from "./last_Mod";

sitemap({
  // … site, filter, i18n per sitemap_And_Robots.md …
  serialize(item) {
    const sister_Urls = get_Sister_Urls(item.url);
    return {
      ...item,
      lastmod: resolve_Last_Mod(item.url),
      ...(sister_Urls ? { links: sister_Urls } : {}),
    };
  },
})
```

Both helpers are pure functions: synchronous, no side effects per call, all the heavy work happens once at module load via two IIFEs.

## Per-file `lastmod` from git

`resolve_Last_Mod(url)` returns an ISO-8601 timestamp via a three-tier resolver:

1. **Git commit time** — one `git log --name-only` traversal at module load builds `Map<file_Path, ISO>` for every tracked file under `src/Content/` and `src/pages/`. Per-URL lookup is then a `Map.get()`; no spawn cost in the hot path.
2. **Filesystem mtime** — for files that aren't in git yet (staged, untracked, brand-new).
3. **Hard error** — if the URL maps to no file at all. Loud failure beats silently emitting build time for stale routes.

### Architecture

The resolver lives in a focused subfolder, one file per concern. The top-level `last_Mod.ts` is a 5-line barrel so the import path stays stable even when internals get refactored:

```
src/Scripts/Astro_Frontmatter/Astro_Config/
├── last_Mod.ts                            # barrel re-export
└── Last_Mod/
    ├── index.ts                           # public exports + architecture overview
    ├── stats.ts                           # LASTMOD_PROFILE counters + debug flag
    ├── url_Parse.ts                       # pathname → { lang, slug }
    ├── filename_Transform.ts              # kebab / camel filename candidates
    ├── slug_Index.ts                      # IIFE: src/Content/ → slug → file map
    ├── sister_Urls.ts                     # get_Sister_Urls (see hreflang section)
    ├── page_File.ts                       # URLs rendered by src/pages/*.astro (no MDX)
    ├── resolve_File.ts                    # orchestrates the three lookup tiers
    ├── git_Times.ts                       # IIFE: one-shot git log → file → ISO map
    └── resolve_Last_Mod.ts                # public resolve_Last_Mod + cache
```

Open `Last_Mod/index.ts` for the architecture comment that maps each file to its role. Each implementation file is small (19-110 lines), one concern per file.

### URL → file: three tiers

`resolve_File_Path(lang, slug)` walks three tiers in priority order. The first hit returns; only if all three miss does the resolver throw.

1. **Slug index** — built once at module load by walking `src/Content/`, reading the first 16 KB of each `.mdx`/`.md`, and extracting the `slug:` frontmatter field (top-level or nested under `mdx_Info:`). Keyed by `<lang>/<slug>` → absolute file path. Handles collections where the URL slug differs from the filename stem (per-language slugs like `legal_Notice.mdx` → `/de/impressum/` ↔ `/en/legal-notice/`).
2. **Per-folder filename match** — for collections without a `slug:` field. Tries two naming conventions: `camel_Case_With_Underline` (Jav-Web filename style) and kebab-case as-is (Partner static_Pages style), both with optional `NNNN_` numeric prefix. Searches under `<folder>/<lang>/` (flat-language-siblings, Partner default) and `<folder>/translation/<lang>/` (Jav-Web legacy).
3. **Page-file fallback** — for URLs rendered by `src/pages/*.astro` rather than an MDX collection (landing, `kontakt`, blog/team/glossary list pages). Maps:
   - `slug === "index"` → `src/pages/[lang]/index.astro`, or `src/pages/index.astro` when [`is_Single_Lang`](../Areas/Language_Modes/single_Lang_Mode.md)
   - `/<lang>/<seg>/` → `src/pages/[lang]/<seg>.astro` (static route)
   - `/<lang>/<seg>/` → `src/pages/[lang]/[<route_key>]/index.astro`, reverse-looked-up via `route_Slugs` so `/de/glossar/` resolves to `[glossary]/index.astro`

Without Tier 3 the resolver would hard-fail on every page-driven route — which is the bug we hit before this subsystem existed.

### Hard-fail philosophy

`resolve_Last_Mod` throws when no file resolves. The error names the URL, language, slug, expected location, and "what to do next." This is intentional: sitemap should never list a URL whose content can't be located.

Common reasons for the throw:

- **Translation missing.** Collection has a `de/foo.mdx` but no `en/foo.mdx`; sitemap emits `/en/foo/` (Astro built it from a stub or stale routing) but the file isn't there.
- **Stale page route.** A `src/pages/[lang]/<page>.astro` was renamed but a referrer still links to the old URL, so Astro emits the page but the resolver can't find the source.
- **Content-tree drift** in customer forks — `supported_Langs` says `[de]` but `en/` and `es/` folders still exist. See [single-lang mode](../Areas/Language_Modes/single_Lang_Mode.md) for content-tree expectations.

The fix is always "make the file exist or fix the route" — never "soften the resolver." The hard error is a feature.

### Profiling

`LASTMOD_PROFILE=1 bun run build` prints per-tier timing/hit counts at process exit:

```
[lastmod profile]
  IIFE (module load):       12.34 ms   walked=141  parsed=141  slugs=23
  resolve_Last_Mod:         18.92 ms   calls=73    cache_hits=2   cache_misses=71
  resolve_File_Path:        16.40 ms   calls=73    indexed=23  fs=42  page=8  miss=0
  git_Times index build:   183.50 ms   files=128
  get_Git_Time:              1.20 ms   calls=73    hits=73   misses=0
```

Useful when the build feels slow or when adding a new collection that doesn't fit the existing tiers.

## hreflang for translated slugs

The native `i18n` option groups URLs into hreflang sets by **path suffix**: `/en/blog/foo/` and `/de/blog/foo/` are siblings because they share `/blog/foo/`. That's correct only when the slug is the same in every language.

It fails on **translated slugs** — when leaf segments localize:

| de | en | es |
|---|---|---|
| `/de/impressum/` | `/en/legal-notice/` | `/es/aviso-legal/` |
| `/de/blog/erster-zahnarztbesuch-kind/` | `/en/blog/first-dental-visit-child/` | `/es/blog/primera-visita-dentista-nino/` |
| `/de/glossar/karies/` | `/en/glossary/caries/` | `/es/glosario/caries/` |

Path-suffix matching can't link these — they share no suffix. Astro's native sitemap emits **no hreflang alternates** between them, which is a real SEO loss (Google can't tell they're translations).

The fix is the `links` override in `serialize`, populated by `get_Sister_Urls(url)`:

- The IIFE in `slug_Index.ts` walks `src/Content/` once, reads each `.mdx`/`.md` frontmatter (top-level or nested `slug:`), and builds three maps: `<lang>/<slug>` → file, `file` → translation group, `group` → list of `{lang, slug}` siblings.
- `get_Sister_Urls(url)` reverses the URL: parses lang + leaf, looks up the file, finds the group, then builds the sister URL for every other language via `build_Path` (which knows how to localize middle segments like `glossar`/`glossary`/`glosario` from `route_Slugs`).
- Returns `null` for URLs whose content has no `slug:` field — those fall back to native path-suffix grouping, which is correct for them.

`build_Path` and `lookup_Route_Key` live in `src/Scripts/Astro_Frontmatter/Common/Routing/route_Slugs.ts`. They're the source of truth for "this is the German segment for the `glossary` route key" — keep that registry up to date and everything downstream stays correct.

### Verifying

After a build, grep the sitemap for one of your translated-slug URLs:

```sh
python3 -c "
import re
with open('dist/sitemap-0.xml') as f: data = f.read()
for m in re.finditer(r'<url>(.*?)</url>', data, re.DOTALL):
    block = m.group(1)
    loc = re.search(r'<loc>(.*?)</loc>', block).group(1)
    if 'impressum' in loc:
        print(loc)
        for link in re.finditer(r'<xhtml:link[^>]*hreflang=\"([^\"]+)\"[^>]*href=\"([^\"]+)\"', block):
            print(f'  hreflang={link.group(1)} → {link.group(2)}')
"
```

You should see one `<loc>` entry plus three `<xhtml:link>` alternates (one per supported lang, including self-reference). If you see only the self-reference, `get_Sister_Urls` returned null — most likely the entry has no `slug:` frontmatter or it's not parsed by the IIFE (check `slug_Index.ts:HEAD_BYTES` if the frontmatter is unusually large).

## Bare-`/` sitemap handling

The language-mode contract lives in [Language modes](../Areas/Language_Modes/linker_Language_Modes.md). This section covers the sitemap side of that contract.

### Multi-Lang Version

Most multi-lang Astro projects in this org route every page under `/<lang>/`. Production server config must issue the real HTTP redirect from `/` to `/<main_Lang>/`. The bare Astro route at `src/pages/index.astro` exists only as a loud deployment diagnostic when the static `dist/` is served without that server-level redirect.

It should not be treated as content and should not silently forward visitors. If it is visible in production, fix the server redirect rather than linking users onward from Astro.

The diagnostic document should carry its own robots instruction:

```html
<meta name="robots" content="noindex, nofollow">
```

That is enough for this page-level case. Do **not** add a matching `Disallow: /` in robots.txt just for the diagnostic: blocking the root can prevent crawlers from seeing the `noindex` directive, and the real production behavior should be an HTTP redirect before this document is served.

For the sitemap: **exclude the bare `/`** from the filter (it's a deployment diagnostic, not a public URL). The filter shape inside `sitemap_Integration.ts`:

```ts
if (!is_Single_Lang && pathname === "/") return false;
```

### Single-Lang Version

Single-lang projects flip this rule. When there is only one language, `/` becomes the actual landing page and `/<main_Lang>/` becomes the redundant duplicate. The sitemap should include `/` and drop `/<main_Lang>/`:

```ts
if (is_Single_Lang && pathname === `/${main_Lang}/`) return false;
```

Do not add `noindex, nofollow` to `/` in single-lang mode. In that mode `/` is the public page. The published shape supports both modes behind one `is_Single_Lang` helper; see [Affected surfaces](../Areas/Language_Modes/affected_Surfaces.md) for the full surface list.

## When to skip this customization

If a project has:

- The same slug in every language (no translated leaves), AND
- No interest in per-file `<lastmod>` accuracy

…then the native `i18n` option alone is sufficient. The custom `serialize` block adds maintenance surface — only adopt it when you actually need one of the two features it provides.

If you adopt only the `lastmod` half, omit the `get_Sister_Urls` call and let the native path-suffix grouping handle hreflang. If you adopt only the hreflang half (unusual), omit `resolve_Last_Mod` — but you give up the cleanest crawl-priority signal you'd otherwise have for free.
