# Data — Common/

`src/Data/Common/` holds the infrastructure constants imported across the entire Astro project — language list, BCP codes, site domain, third-party service IDs, preview metadata. The defining characteristic: these values change when the project's *shape* changes (new tier ships, new language gets added, domain changes), not when a feature evolves. See the [Data linker](linker_Data.md) for the full Data-vs-Utils mutability rule.

## What lives here

**Yes:**

- **Language configuration** — the canonical lang list, BCP-47 codes, og:locale variants, language endonyms (`Deutsch`/`English`/`Español` for switcher UIs). Every multilingual page reads this.
- **Site domain and URLs** — production domain (with or without trailing slash, document the choice), CDN base, any cross-repo URLs that pages reference.
- **Third-party service IDs** — analytics website IDs, embed origins, anything the page renders into a `<script>` or `<meta>`.
- **Preview / tier / demo metadata** — structural values that only render when the deploy is a sales preview (e.g. `demo_Tier_Key`, pricing URL). Visitor-facing tier copy belongs in `src/Scripts/Multi_Lang_Txts/`.
- **Site identity** — `site_Name` (the public name of the entity running the site — a practice name, person's full name, or brand; used in `og:site_name`, `<title>` suffix, `meta[author]`), `author_Title_Suffix` (derived from `site_Name`, e.g. ` — Zahnarztpraxis Mitte`), footer copyright lines.
- **Optional social handles** — `twitter_Handle` (the site's Twitter/X handle including `@`; empty string when unused so consumers can emit tags conditionally with `{twitter_Handle && …}` rather than emitting empty content).

**No:**

- **Helper functions with their own logic** — these go in [Utils/](../Utils/linker_Utils.md). The bright line: if it takes meaningful input and computes a result, it's a helper.
- **Per-feature lookup data** (service slug → schema.org type, blog category → icon name) — `Utils/<Feature>/`. Data is for site infrastructure, not feature implementation details.
- **Per-tenant or per-customer state** (SMTP creds, branding overrides, per-clinic content) — not here at all. See the linker's "Customer / tenant state" section.
- **Content** — copy, MDX, JSON-LD bodies belong in `src/Content/`.
- **Theme tokens** — colors, radii, fonts go in `Styles/themes.css`. They're already a contract, just declared differently.
- **OG image contracts and generator inputs** — template defaults, reusable OG assets, output paths, and renderer theme tokens live in `Data/Og_Img/`. See [og_Img.md](og_Img.md).
- **Translation strings** — UI labels keyed by lang go in `src/Scripts/Multi_Lang_Txts/`. `Data/Common/` holds language *metadata* (the list of supported langs), not the *strings* shown to users.

## The derived-helpers exception

`mandatory_Inp.ts` exports both raw constants and small derived helpers built from those constants:

```ts
export const langs_Config = {
  main_Lang: "de",
  supported_Langs: [
    { url: "de", bcp: "de-DE" },
    { url: "en", bcp: "en-US" },
    { url: "es", bcp: "es-ES" },
  ],
} as const;

// Derived from the constant above — stays here so the file is self-contained.
export const supported_Url_Langs = langs_Config.supported_Langs.map((l) => l.url);
export const bcp_From_Url = Object.fromEntries(
  langs_Config.supported_Langs.map((l) => [l.url, l.bcp]),
) as Record<string, string>;
```

This is allowed. The rule: derived helpers that are pure projections of the file's own constants stay with the constants — moving them to `Utils/` would force every consumer to import from two places to get language metadata. A helper that joins data from multiple sources, takes runtime input, or implements branching logic belongs in `Utils/`.

If a derived helper grows past a few `map`/`Object.fromEntries`/`Record` lines, lift it into `Utils/Common/`.

## File pattern

One file per coherent concern. Don't pack unrelated constants into a junk-drawer file. Current patterns from Partner and Jav_Web:

| File | Holds |
|---|---|
| `mandatory_Inp.ts` | Language config, domain, `site_Name` + `author_Title_Suffix` + `twitter_Handle`, analytics, demo metadata — the "fork this when project shape changes" file |
| `<future>` | Add new files when a new coherent concern emerges. Don't add fields to `mandatory_Inp.ts` that don't fit "structural infrastructure modified by engineers." |

## Analytics / Umami

When a project uses self-hosted Umami behind private infrastructure, the public page must stay same-origin:

```html
<script defer src="/assets/js/i18n.js" data-website-id="..." data-host-url="/"></script>
```

Do not emit the private analytics hostname in public HTML, for example `https://analytics.example.com/i18n.js`, when that hostname is intended to stay NetBird-only or otherwise private. Public visitors are outside that network, and modern browsers can block public pages that try to load private-network resources.

The Caddy/runtime layer owns the private hop:

```text
/assets/js/i18n.js -> Umami /i18n.js
/api/locale        -> Umami /api/locale
```

`data-host-url="/"` is required when the script is served from a subpath such as `/assets/js/i18n.js`. Without it, Umami derives API endpoints from the script path and can post below `/assets/js/` instead of from the site root.

Header doc comment on every file should state:
1. Who edits this and when.
2. The cross-repo sibling file it must stay in sync with, if any.
3. What's *not* in this file but might be expected to be (route to the right location).

## Add or rename — checklist

1. Verify the constant qualifies as **infrastructure** (the mutability test from the [linker](linker_Data.md)). If it's a helper, fork into `Utils/Common/`. If it's per-tenant, fork into the project's customer-config flow.
2. Pick the right file. If no existing file fits, add a new one — don't grow `mandatory_Inp.ts` past its purpose.
3. Name the constant `noun_With_Capitals` for data, `SCREAMING_SNAKE` only for "do not touch lightly" hard-codes.
4. Add a header doc comment to the file if it's new (see "File pattern" above).
5. On rename, grep across `src/`, `astro.config.mjs`, `ec.config.mjs`, and `Project_Manag/Docs/` — `refac-cli` doesn't reliably update bare `src/` paths or `.astro` frontmatter.
6. If the value has a cross-repo sibling (OG contract mirrors theme tokens, language list might be referenced by the contact-form backend's tenant config), document the link in a comment so the next change picks up both.
