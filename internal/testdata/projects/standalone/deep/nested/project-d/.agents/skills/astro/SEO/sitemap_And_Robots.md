# Sitemap and robots

Sitemap and robots.txt are paired: the sitemap *lists* the URLs you want indexed, robots.txt *gates* what crawlers may visit and points them at the sitemap. Both should ship on every public Astro project in this org — the cost is near-zero, the discoverability win is real.

## Sitemap — `@astrojs/sitemap`

The official integration. It walks Astro's emitted route list at build time and writes `sitemap-index.xml` + one or more `sitemap-N.xml` chunks into `dist/`. Zero per-page wiring.

### Install and wire

```sh
bun add @astrojs/sitemap
```

```js
// astro.config.mjs
import { defineConfig } from "astro/config";
import { site } from "./src/Data/Site_Config/site_Config.ts";
import { sitemap_Integration } from "./src/Scripts/Astro_Frontmatter/Astro_Config/sitemap_Integration.ts";

export default defineConfig({
  site: site.domain,  // REQUIRED for sitemap to emit absolute URLs

  integrations: [
    sitemap_Integration(),
  ],
});
```

`astro.config.mjs` should stay this thin. The project-local `sitemap_Integration.ts` owns the `@astrojs/sitemap` import, the `filter`, the `i18n` config, `serialize()`, and the multi-lang vs single-lang root handling. It reads the language mode from `mandatory_Inp.ts` (`is_Single_Lang`, `main_Lang`, `supported_Url_Langs`) so adding/removing languages changes sitemap behavior from one source of truth.

### What `site:` does

`site:` is the production origin (no trailing slash). The sitemap uses it to emit absolute URLs. Without it, the integration fails the build (it can't write `<loc>` entries with relative URLs). The same `site:` value is the source of truth for canonical URLs in the head block (see [Head tags](head_Tags.md)).

### Excluding routes

The `filter` callback lives in `src/Scripts/Astro_Frontmatter/Astro_Config/sitemap_Integration.ts`. It runs once per emitted route. Return `true` to include, `false` to exclude. Common exclusions:

- **Tooling routes** — `dev/`, `proto/`, `live-editor/`, `playground/`.
- **Admin / authenticated pages** — `admin/`, `dashboard/`.
- **Search and filter result URLs** — `/search?q=…` (low-value, infinite combinations).
- **Pagination beyond page 1** — debatable; include if pages are evergreen, exclude if they churn.
- **Draft preview routes** — anything under `/preview/<token>/`.
- **404 routes** — `404.html`, `/404/`, and localized variants such as `/de/404/`. These pages should exist for humans and server error handling, but they are not public content URLs. Add `noindex` in the page head and exclude them from the sitemap. The full workflow lives in [Custom 404 pages](../Areas/Hidden_Feartures/404/linker_404.md).
- **Bare `/` in multi-lang projects.** This is a deployment diagnostic, not a public URL. When every page lives under `/<lang>/`, production server config must redirect bare `/` to `/<main_Lang>/`. The Astro bare-root page owns its crawler instruction with `<meta name="robots" content="noindex, nofollow">`; robots.txt does not need a special rule for `/`. Exclude bare `/` from the sitemap:

  ```ts
  if (!is_Single_Lang && pathname === "/") return false;
  ```

- **Bare `/` in single-lang projects.** This is the real landing page and should stay in the sitemap. The redundant `/<main_Lang>/` URL is the one to exclude. The full mode contract is in [Language modes](../Areas/Language_Modes/linker_Language_Modes.md).

**Don't exclude robots-Disallowed routes from the sitemap.** A page in robots.txt is told "don't crawl"; a page absent from the sitemap is told "I don't endorse this URL". These are different signals. If you genuinely don't want indexing, both Disallow and exclude — but starting from the sitemap side and checking robots.txt for parity is the right order.

### i18n entries

The `i18n` option makes the integration emit `<xhtml:link>` alternates inside each `<url>` entry, the sitemap-XML equivalent of the hreflang `<link>` tags in `<head>`. Set `defaultLocale` to the project's `main_Lang` and `locales` to the full `{ url_code: bcp_code }` map.

Without this option, the sitemap still works but doesn't tell crawlers which URLs are translations of each other — they have to discover that via the head-block hreflang tags. Cheap insurance to set both.

### Going beyond the native options

Two extensions are common across org projects and live in their own deep-dive: [Sitemap customization](sitemap_Customization.md). Open that doc when you need either of:

- **Per-file `<lastmod>` from git timestamps.** The native integration emits no `<lastmod>` (or build time uniformly). A three-tier resolver gives every URL a real per-file timestamp from `git log` — meaningful crawl-priority signal for free. Lives in a focused `Astro_Config/Last_Mod/` subfolder.
- **hreflang alternates for translated slugs.** Native i18n groups URLs by path suffix, so `/de/impressum/` and `/en/legal-notice/` are NOT detected as siblings (they share no suffix). A custom `serialize` block populates the `links` override via a slug index walked from frontmatter — works for any project that localizes leaf segments.

Both layer on top of the basic `i18n` option without replacing it. Skip the deep-dive if your project has identical slugs across languages and you don't care about per-file lastmod accuracy — the basics here are enough.

### Build output

```
dist/
├── sitemap-index.xml      # references each chunk
└── sitemap-0.xml          # up to 45 000 URLs per chunk; large sites get -1, -2, …
```

Inspect with `cat dist/sitemap-index.xml` after a build. Verify the URL count looks reasonable. If a route you expected is missing, `should_Include_Sitemap_Page()` in `sitemap_Integration.ts` is the first suspect.

## Robots.txt — two patterns

Pick by question: *do the Disallow rules need to react to the language registry, or are they fixed strings?*

### Pattern 1 — static (default)

Most projects. Drop `public/robots.txt`:

```
User-agent: *
Allow: /

# Block internal tooling
Disallow: /admin/
Disallow: /dev/
Disallow: /proto/

Sitemap: https://example.com/sitemap-index.xml
```

Astro copies anything under `public/` verbatim to `dist/`. The `Sitemap:` line must be the full production URL — robots.txt has no relative-URL concept.

**When the static pattern is enough.** The Disallow paths don't include language prefixes (e.g. `/admin/`, not `/de/admin/`). Crawlers handle prefix-based blocking fine — `Disallow: /admin/` blocks `/en/admin/` and `/de/admin/` because Disallow is a substring match.

Wait, that's wrong. `Disallow: /admin/` blocks paths *starting with* `/admin/`, not paths *containing* `/admin/`. If the project routes admin under `/<lang>/admin/`, you need one Disallow per lang prefix — which is exactly when the dynamic pattern earns its complexity.

### Pattern 2 — dynamic API route

When robots.txt must enumerate paths per language (every admin page lives at `/<lang>/admin/`), the static file becomes a maintenance hazard — adding a new supported lang means editing two places (language registry + robots.txt). Move robots.txt to a dynamic API route so it reads the registry:

```ts
// src/pages/robots.txt.ts
import type { APIRoute } from "astro";
import { langs_Config } from "@data/Common/mandatory_Inp";

const supported_Langs = langs_Config.supported_Langs.map((l) => l.url);

const build_Robots_Txt = (sitemap_Url: URL) => {
  const live_Editor_Rules = supported_Langs.map((lang) => `Disallow: /${lang}/live-editor/`).join("\n");
  const admin_Rules       = supported_Langs.map((lang) => `Disallow: /${lang}/admin/`).join("\n");

  return `\
User-agent: *
Allow: /

# Block internal tooling/areas (per-language Disallow generated from the registry)
${live_Editor_Rules}
${admin_Rules}

Sitemap: ${sitemap_Url.href}
`;
};

export const GET: APIRoute = ({ site }) => {
  if (!site) {
    throw new Error("robots.txt: Astro 'site' is undefined. Set 'site' in astro.config.mjs");
  }
  const sitemap_Url = new URL("sitemap-index.xml", site);
  return new Response(build_Robots_Txt(sitemap_Url));
};
```

The hard-fail on missing `site` is intentional — `site` is the only correct source for the absolute `Sitemap:` URL, and silently emitting `http://localhost/sitemap-index.xml` into production would tank crawl discoverability.

Adding a new language flows through the registry. The robots.txt rebuilds on the next deploy without further edits.

### What goes in robots.txt — checklist

- **`User-agent: *`** as the default. Add per-bot blocks only when you actually need to differentiate (e.g. blocking aggressive AI crawlers).
- **`Allow: /`** explicitly. Some legacy crawlers treat missing Allow as ambiguous.
- **`Disallow:`** for tooling, admin, draft previews, search result pages, and anything else that should never appear in search results.
- **`Sitemap:`** as the last line. Absolute URL. One per sitemap if the project somehow has more than one (rare).
- **Don't `Disallow: /api/`** unless the project genuinely exposes a public API the user-facing site doesn't link to. Crawlers won't follow API URLs they don't see; the rule is redundant and clutters the file.
- **No comments meant for crawlers.** `# this is a debug rule` is fine for humans; comments don't gate crawler behaviour.

### Bot-specific blocks

Some AI training crawlers respect robots.txt and accept a `User-agent: <Name>` block. Common ones:

```
User-agent: GPTBot
Disallow: /

User-agent: ClaudeBot
Disallow: /

User-agent: CCBot
Disallow: /

User-agent: Google-Extended
Disallow: /
```

Decision is editorial — the project owner picks. If unsure, leave them all allowed (the default) and revisit when the project has a clear policy.

## Verifying

After a build, the relevant artefacts:

- `dist/sitemap-index.xml` — exists, references chunks.
- `dist/sitemap-0.xml` — exists, URL count matches expectation, internal routes absent.
- `dist/robots.txt` — for the static pattern, copied from `public/`. For the dynamic pattern, generated from the API route at build time (static build) or per-request (SSR build).

Run `bun run build && bun run preview` and:

- `curl http://localhost:4321/robots.txt` — full text, `Sitemap:` line present and absolute.
- `curl http://localhost:4321/sitemap-index.xml` — XML, references chunks.
- `curl http://localhost:4321/sitemap-0.xml` — `<loc>` entries are absolute production URLs (not `http://localhost`).

If any of those show `localhost`, `site:` in `astro.config.mjs` is wrong or missing.

## Pairing with the head block

The head block (see [Head tags](head_Tags.md)) emits:

```astro
<link rel="sitemap" href="/sitemap-index.xml" />
```

This is the third discovery channel after robots.txt's `Sitemap:` line and Google Search Console submission. Cheap to emit, marginally useful for crawlers that don't parse robots.txt first.

## Add a new excluded route — checklist

1. **Decide the signal.** Do you want crawlers to *not visit* (robots.txt Disallow), to *not index* (sitemap exclusion), or *both*?
2. **Update robots.txt.** Static pattern: edit `public/robots.txt`. Dynamic pattern: extend the rules array in `src/pages/robots.txt.ts`.
3. **Update the sitemap filter.** Edit `should_Include_Sitemap_Page()` in `src/Scripts/Astro_Frontmatter/Astro_Config/sitemap_Integration.ts` to return `false` for the new path.
4. **For multi-language admin/tooling paths**, the dynamic robots.txt pattern is almost always the right move — see [Pattern 2](#pattern-2--dynamic-api-route).
5. **Rebuild and verify** — `cat dist/robots.txt` and grep `dist/sitemap-0.xml` for the excluded path; both should reflect the change.
