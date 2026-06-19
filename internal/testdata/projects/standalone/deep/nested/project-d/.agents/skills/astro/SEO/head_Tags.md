# Head tags

The root Layout (conventionally `src/Layouts/Base_Layouts/Init_Layout.astro`) emits the SEO head-tag block on every page. This doc covers what tags belong there, why, the order they go in, and the prop contract the Layout exposes so pages can override defaults without bypassing the block.

## The complete block

In order — the order matters for a few tags (see [Order rules](#order-rules)):

1. **`<meta charset="UTF-8">`** — must be in the first 1024 bytes of `<head>`. Always first.
2. **`<meta name="viewport" content="width=device-width, initial-scale=1.0">`** — mobile rendering. Always second.
3. **Pre-paint scripts** (theme bootstrap, anything that must run before the first paint to avoid FOUC). `is:inline` so Astro doesn't move them.
4. **Fonts** — `<Font cssVariable="…" preload={…} />`. See [`../Fonts/setup.md`](../Fonts/setup.md).
5. **`<title>`** and **`<meta name="description">`** — the two highest-impact SEO surfaces. Keep title under ~60 chars and description under ~155 chars; longer values are truncated in SERPs.
6. **Favicon** — SVG first (modern browsers), ICO fallback. Chromium-based browsers silently request `/favicon.ico` and ignore the SVG link, so both are required.
7. **Author / referrer / theme-color** — `<meta name="author">`, `<meta name="referrer" content="strict-origin-when-cross-origin">`, two `<meta name="theme-color">` (light + dark via `media` attribute).
8. **Open Graph** — the full og:* set (see [Open Graph](#open-graph)).
9. **Twitter Card** — `summary_large_image` + title/description/image + site/creator. See [Twitter Card](#twitter-card).
10. **Canonical** — `<link rel="canonical" href={url.href}>`, self-referencing per-language URL.
11. **Hreflang alternates** — one `<link rel="alternate">` per supported language plus `x-default`. See [Hreflang](#hreflang-and-x-default).
12. **Sitemap link** — `<link rel="sitemap" href="/sitemap-index.xml">`. The `@astrojs/sitemap` integration produces `sitemap-index.xml`; this tag advertises it to crawlers that don't parse robots.txt.
13. **JSON-LD scripts** — one `<script type="application/ld+json">` per schema object. See [JSON-LD](json_Ld.md).
14. **Analytics** — last, `defer`-ed, so it never blocks any of the above. For self-hosted Umami behind private infrastructure, emit a same-origin script path plus `data-host-url="/"`; Caddy owns the private proxy to Umami.

## Order rules

- `<meta charset>` and `<meta viewport>` are first, in that order. Browsers stop parsing the document if they encounter content before charset.
- Pre-paint scripts (theme bootstrap, language detection that mutates DOM) run before any rendered content — put them in `<head>` early, marked `is:inline`. Putting them after `<title>` is fine; putting them after a `<style>` block that depends on the theme is not.
- Preload links (`<link rel="preload">`, `<Font preload>`) should appear before the resources they preload would otherwise be discovered. Fonts at step 4 — before `<title>` — gives the parser the earliest possible hint.
- JSON-LD goes near the end of `<head>`. Crawlers parse it from anywhere in the document, but it can be large and would otherwise delay other tags.
- Analytics is last. Defer it; analytics scripts have no business participating in the first paint. Do not expose a private analytics hostname in public markup; use a same-origin proxy path such as `/assets/js/i18n.js` and set `data-host-url="/"` so Umami event/API paths resolve from the site root.

## The Layout's prop contract

The root Layout takes the data each page needs to specialise the block. The exact prop names are a project choice, but every project converges on roughly this shape:

```ts
interface Props {
  /** Page-specific title. Layout wraps with site-name suffix per project convention. */
  title: string;

  /** Page-specific description. Optional only for pages that genuinely have no
      summary worth indexing (very rare). */
  description?: string;

  /** Override the default OG image. Path relative to site root, e.g. `/og-services.jpg`.
      Skipped if not provided; Layout falls back to the deterministic per-page card
      computed by the OG image pipeline. */
  image?: string;

  /** Per-lang URL map for hreflang alternates. Required for pages with translated
      slugs (blog posts, glossary entries) where a plain /de/ → /en/ path-prefix
      swap would point to the wrong slug. Keys are lang codes, values are absolute
      paths starting with /. Should include the current lang for self-reference.
      Pages with parallel slugs across langs can omit this — the Layout falls back
      to the path-prefix swap. */
  alternates?: Partial<Record<string, string>>;

  /** JSON-LD schema objects to emit in <head>. See json_Ld.md.
      Built per-page in frontmatter via the builders under
      Scripts/Astro_Frontmatter/Common/JsonLd/. Filtered for null/undefined so
      callers can include conditionals inline without guarding the array shape
      (e.g. faq builder returns null when there are no items). */
  extra_Schemas?: (object | null | undefined)[];

  /** Selects the default JSON-LD schema set. "website" | "article" | "profile".
      Defaults to "website". See json_Ld.md, og_Type dispatcher. */
  og_Type?: "website" | "article" | "profile";
}
```

The Layout never reads page-specific data from anything other than `Astro.props` and `Astro.url` — keeping it pure means a wrong tag on a single page can always be traced back to the prop or the source data, not to Layout state leaking across pages.

## Title format

Pick one of two patterns at the project level and apply consistently:

- **Page — Site Name** (em-dash separator). What most sites use. Long-tail SERP-friendly because the unique part comes first.
- **Page · Site Name** or **Page | Site Name**. Same idea, different separator.

Special case: when the page title *equals* the site name (the landing page in many projects), emit just the site name — don't render "Site Name — Site Name".

## Description

- One sentence, 120–155 chars, written for the human who'll click the result. Not a keyword list.
- Make it page-specific. Reusing the site-wide description across every page wastes the highest-impact SEO surface there is.
- For collection pages (blog index, glossary index), summarise what's *in* the index, not a generic site pitch.

## Open Graph

The full set the Layout emits on every page:

| Tag | Value | Notes |
|---|---|---|
| `og:type` | `website` / `article` / `profile` | Driven by `og_Type` prop. Default `website`. |
| `og:title` | Page title | Often the same as `<title>` minus the site-name suffix. |
| `og:description` | Page description | Same as `<meta name="description">` unless the page sets an OG-specific override (e.g. shorter for social cards). |
| `og:url` | Canonical URL | Self-referencing per-language URL, same as the `<link rel="canonical">`. |
| `og:locale` | BCP-47 with underscore | `en_US`, not `en-US`. Spec requires underscore. |
| `og:site_name` | Site / brand name | Constant per project. |
| `og:image` | Absolute URL | Always emit when an image is available. Crawlers reject relative URLs. |
| `og:image:width` | `1200` | Required by Twitter/X and several other crawlers to lay out the preview. |
| `og:image:height` | `630` | Same. |
| `og:image:type` | `image/jpeg` (or `image/png`) | Pair with the actual file format the OG pipeline emits. |
| `og:image:alt` | Same as `og:title` | Accessibility for screen readers on platforms that render alt text. |
| `og:locale:alternate` | BCP-47 with underscore | One per other supported language. |

**OG title vs HTML title.** If the project's HTML title format adds a suffix (" — Site Name"), the OG title should usually be the raw page title without it. Crawlers truncate around 55–60 chars, and the suffix wastes that budget on social cards. Either compute both from a single source (`title`) plus a `fullTitle` derivation, or expose an `og_title` override on the SEO schema.

**OG description vs HTML description.** Same field by default. Override per page only when the social card benefits from different copy (shorter, line breaks via `\n`, dropping prefixes like "CV:" that help in browser tabs but waste space on cards).

## Twitter Card

`summary_large_image` is the right card type for every page in this org. The smaller `summary` card doesn't render large enough to be useful.

| Tag | Value | Notes |
|---|---|---|
| `twitter:card` | `summary_large_image` | Constant. |
| `twitter:title` | Same as `og:title` | Twitter reads `og:title` as a fallback but emit explicitly. |
| `twitter:description` | Same as `og:description` | Same reasoning. |
| `twitter:image` | Same as `og:image` | Same. |
| `twitter:site` | `@handle` | Omit if the site has no Twitter/X account — emit conditionally, not as a hardcoded empty string. |
| `twitter:creator` | `@handle` | Same as `twitter:site` for single-author sites. Per-author only when the project has multiple authors with distinct handles. |

**Conditional emission pattern.** The handle lives in `Data/Common/mandatory_Inp.ts` as `twitter_Handle`. Set it to an empty string when unused; the Layout emits the tags only when the value is truthy:

```astro
{twitter_Handle && <meta name="twitter:site"    content={twitter_Handle} />}
{twitter_Handle && <meta name="twitter:creator" content={twitter_Handle} />}
```

Never hardcode `content=""` — an empty `twitter:site` tag is noise and some parsers treat it as a signal. Omit entirely when unused.

## Canonical

```astro
<link rel="canonical" href={canonicalUrl} />
```

Always self-referencing. Per-language URL — the German page's canonical is the German URL, not the English one. Hreflang handles the cross-language relationship; canonical handles "this is the URL to credit".

Compute it from the production `site:` (set in `astro.config.mjs`) plus `Astro.url.pathname`:

```ts
const canonicalUrl = new URL(Astro.url.pathname, site_Domain).toString();
```

Reading `Astro.url.href` directly works in production but breaks in dev (uses `localhost`). Always build from the production `site:`.

## Hreflang and x-default

Two emission modes, picked by whether the page provides a translated-slug `alternates` map:

**Mode 1 — translated-slug pages** (blog posts, glossary entries, anything whose URL slug differs per language). Page passes `alternates: { de: "/de/blog/karies", en: "/en/blog/cavities", es: "/es/blog/caries" }`. Layout emits one `<link rel="alternate">` per entry. Missing langs (no sibling translation) are skipped — emitting a path-prefix-swapped URL there would advertise a non-existent page.

**Mode 2 — parallel-slug pages** (most pages). Layout swaps the first path segment with each supported lang code. Cheap, correct for any page whose slug is identical across languages.

```astro
{langs_Config.supported_Langs.map((l) => {
  let altPathname: string | undefined;
  if (alternates) {
    altPathname = alternates[l.url];
    if (!altPathname) return null;
  } else {
    altPathname = pathname.replace(/^\/[a-z]{2}(\/|$)/, `/${l.url}$1`);
  }
  const altUrl = new URL(altPathname, site_Domain).toString();
  return <link rel="alternate" hreflang={l.bcp} href={altUrl} />;
})}
```

**x-default.** Points at the `main_Lang` variant — the language to serve when the visitor's preferences match nothing in `supported_Langs`. Emit it *after* the per-language alternates. If the main-lang translation doesn't exist for this page, omit `x-default` entirely rather than advertising a non-existent URL.

**Trailing slash.** Whatever the project's canonical form is (with or without trailing slash), every hreflang URL and the canonical must match it. Mismatched trailing slashes cause Google to treat the variants as separate URLs and fragment the link equity.

## Theme-color

```astro
<meta name="theme-color" content="#ffffff" media="(prefers-color-scheme: light)" />
<meta name="theme-color" content="#0e0f12" media="(prefers-color-scheme: dark)" />
```

Sets the browser chrome / status-bar tint on Safari iOS, Chrome Android, and several PWA contexts. Two entries with `media` queries so light and dark themes get the right tint. Values must mirror the project's `--background` token from `Styles/theme.css` for each mode.

**CSS custom properties cannot be used in `<meta>` attributes.** `content="var(--background)"` does not work — the browser reads the attribute before the CSS cascade runs. The value must be a static hex string. Derive it by converting the token's `oklch(…)` value to hex (a small Python/Node snippet or browser DevTools color picker is the reliable path; eyeballing oklch is error-prone). Add a comment in the Layout pairing the hex with the source token so the next theme change knows which value to update:

```astro
<!-- Light: oklch(1 0 0) = #ffffff | Dark: oklch(0.17 0.005 264) = #0e0f12
     CSS vars cannot be used in meta content — values mirror --background from theme.css -->
<meta name="theme-color" content="#ffffff" media="(prefers-color-scheme: light)" />
<meta name="theme-color" content="#0e0f12" media="(prefers-color-scheme: dark)" />
```

## Favicon

```astro
<link rel="icon" type="image/svg+xml" href="/favicon.svg" />
<link rel="icon" href="/favicon.ico" />
```

Both are required. Firefox and most modern browsers honour the SVG. Chromium-based browsers (Chrome, Brave, Edge, Vivaldi, Opera) silently request `/favicon.ico` regardless of the `<link>` tag — without the ICO they show a fallback / log a 404. Drop both files in `public/`.

## Add or rename — checklist

Adding a new SEO-relevant tag to the head block:

1. Decide the order it goes in (see [Order rules](#order-rules)).
2. If the value varies per page, expose it as a Layout prop — never compute it from `Astro.locals` or globals; that hides the dependency from page templates.
3. If the value is constant per project, source it from `src/Data/Common/` not from a hardcoded string in the Layout.
4. Add the tag to the OG / Twitter sections together if it's a content tag that crawlers expect on both surfaces (title, description, image).
5. Test in production-ish mode (`bun run build && bun run preview`) — dev mode's `localhost` URLs make canonical/OG/hreflang inspection misleading.

Adding a new supported language:

1. Add the lang to `Data/Common/langs_Config.supported_Langs`. Everything in this doc (OG alternates, hreflang map, x-default fallback) reads from that registry — no per-tag changes needed.
2. Make sure the BCP-47 mapping (`bcp_From_Url`) and the OG-locale mapping (`og_Locale_From_Url`, which uses underscore form) include the new lang.
3. Trigger a build and inspect one page's `<head>` to confirm a new `og:locale:alternate` and `<link rel="alternate">` appear for the new lang.
