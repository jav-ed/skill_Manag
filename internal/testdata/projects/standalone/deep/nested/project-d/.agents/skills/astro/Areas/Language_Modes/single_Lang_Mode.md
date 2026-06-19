# Single-Lang Mode

Single-lang mode is active when `langs_Config.supported_Langs.length === 1`.

This is not "multi-lang with one language." It is a separate routing and SEO mode. Surfaces that exist to disambiguate language variants stop carrying signal and should be hidden or skipped.

## Helper

Use the derived helper from `src/Data/Common/mandatory_Inp.ts`:

```ts
export const is_Single_Lang = langs_Config.supported_Langs.length === 1;
```

Read this flag wherever language count materially changes behavior. Do not branch directly on `supported_Langs.length` in surface code.

## Bare Root

In single-lang mode, bare `/` is the real public landing page.

`src/pages/index.astro` renders the same `Landing_Page.astro` component used by the language-root landing route, but `/` is the URL that should be indexed and evaluated.

Do not copy multi-lang root behavior into single-lang mode:

- no deployment diagnostic at `/`,
- no `noindex, nofollow` on `/`,
- no server-required `/ -> /<main_Lang>/` redirect,
- no sitemap exclusion for `/`.

## Redundant Language Root

The shared route tree may still build `/<main_Lang>/`. In the current template, that URL is redundant to `/` and is excluded from the sitemap.

If duplicate content matters for a final client fork, decide explicitly between:

- redirecting `/<main_Lang>/` to `/`, or
- setting canonical for `/<main_Lang>/` to `/`.

Do not silently invent this in shared template code without a project decision.

## What Does Not Change Today

Most non-root routes still carry the `/<main_Lang>/` prefix:

```txt
/de/kontakt/
/de/technische-qualitaet/
```

Clean single-lang URLs such as `/kontakt/` or `/technische-qualitaet/` require a separate routing migration. Do not treat them as automatic output from `is_Single_Lang`.

## SEO Head

Single-lang pages should not emit hreflang alternates or `x-default`. There are no alternate language URLs to advertise.

`og:locale:alternate` also disappears because there are no sibling locales.

## Navigation

Language switchers are hidden in both desktop and mobile chrome:

- `Top_Header_Main.astro`
- `Nav_Overlay_Zetun_Web.astro`

Theme controls and normal navigation remain.

## Content Tree Expectations

A single-lang customer fork must slim the content tree to match `supported_Url_Langs`. For each collection under `src/Content/<collection>/`, remove inactive language folders unless that project still intentionally builds those languages.

This is intentional discipline: if `supported_Langs` says only `de`, but `en/` and `es/` content folders remain active, route helpers should fail instead of masking drift.

## Verification

1. `bun run build` runs clean after the content tree has been slimmed.
2. `dist/sitemap-0.xml` lists `/` and not `/<main_Lang>/`.
3. Built page heads contain no `<link rel="alternate" hreflang>` tags.
4. Built page heads contain no `x-default`.
5. Header and mobile navigation do not render the language switcher.

## Related

- [Affected surfaces](affected_Surfaces.md)
- [Multi-lang mode](multi_Lang_Mode.md)
- [Page routes](../page_Routes.md)
- [Sitemap and robots](../../SEO/sitemap_And_Robots.md)
- [Head tags](../../SEO/head_Tags.md)
