# Multi-Lang Mode

Multi-lang mode is active when `langs_Config.supported_Langs.length > 1`.

In this mode, every real public page lives under a language root:

```txt
/de/
/en/
/es/
/de/kontakt/
/en/contact/
/es/contacto/
```

Bare `/` is not public content. Production server config must redirect `/` to `/<main_Lang>/` before Astro serves the static root document.

## Bare Root

`src/pages/index.astro` renders a deployment diagnostic in multi-lang mode. If this page is visible in production, deployment is wrong.

The diagnostic document must:

- use the project theme enough to look intentional,
- tell the operator that server-level root linking is missing,
- emit `<meta name="robots" content="noindex, nofollow">`,
- avoid canonical, OG, hreflang, JSON-LD, and sitemap-link semantics that make it look like a public page,
- stay out of the sitemap.

## Sitemap

`src/Scripts/Astro_Frontmatter/Astro_Config/sitemap_Integration.ts` excludes bare `/` and includes each language root.

Expected multi-lang sitemap shape:

```txt
https://example.com/de/
https://example.com/en/
https://example.com/es/
```

Not expected:

```txt
https://example.com/
```

## SEO Head

Normal public pages emit:

- canonical for the current language URL,
- hreflang alternates for available sibling languages,
- `x-default` pointing at the main-language variant,
- `og:locale` for current language,
- `og:locale:alternate` for other available languages.

The root diagnostic is outside this normal page contract.

## Navigation

Language switchers render in desktop and mobile chrome. Their URLs come from `full_Url_By_Lang` / `all_avail_langs`, not from hardcoded path swaps.

## Content

Content collections should contain entries for every supported language that the page advertises. Missing supported-language content should fail loudly rather than silently falling back to `main_Lang`.

## Related

- [Affected surfaces](affected_Surfaces.md)
- [Single-lang mode](single_Lang_Mode.md)
- [Page routes](../page_Routes.md)
- [Sitemap and robots](../../SEO/sitemap_And_Robots.md)
- [Head tags](../../SEO/head_Tags.md)
