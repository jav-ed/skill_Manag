# Language Modes

Language mode is a project-shape concern, not a page concern. A site with one supported language and a site with several supported languages are two different routing and SEO modes.

The source of truth is `src/Data/Common/mandatory_Inp.ts`:

```ts
export const langs_Config = {
  main_Lang: "...",
  supported_Langs: [
    { url: "...", bcp: "..." },
  ],
} as const;

export const supported_Url_Langs = langs_Config.supported_Langs.map((l) => l.url);
export const is_Single_Lang = langs_Config.supported_Langs.length === 1;
```

Read `is_Single_Lang` wherever the number of active languages changes behavior. Do not branch directly on `supported_Langs.length` in surface code.

## Routing

- [Affected surfaces](affected_Surfaces.md): start here before changing language count. Maps every code/doc area that changes between multi-lang and single-lang mode.
- [Multi-lang mode](multi_Lang_Mode.md): public pages live under `/<lang>/`; bare `/` is a deployment diagnostic and must stay out of the sitemap.
- [Single-lang mode](single_Lang_Mode.md): bare `/` is the public landing page; language alternates and language switchers disappear.

## Ownership Rule

This folder owns the cross-surface mode contract. It does not replace the owning docs:

- Routing file shape stays in [Page routes](../page_Routes.md).
- Content organization stays in [Content](../../Content/linker_Content.md).
- SEO head behavior stays in [Head tags](../../SEO/head_Tags.md).
- Sitemap and robots behavior stays in [Sitemap and robots](../../SEO/sitemap_And_Robots.md).
- Config-time sitemap machinery stays in [Sitemap customization](../../SEO/sitemap_Customization.md) and [Astro_Frontmatter](../../Scripts/astro_Frontmatter.md).

When a mode rule changes, update this folder first, then update the owning surface doc.
