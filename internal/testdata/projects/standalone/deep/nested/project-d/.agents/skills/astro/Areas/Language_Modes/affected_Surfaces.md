# Language Mode Affected Surfaces

Changing from multi-lang to single-lang, or adding languages back, affects more than URLs. Check every surface below before calling the mode change complete.

## Source Of Truth

| Surface | Implementation | What changes |
|---|---|---|
| Mandatory language input | `src/Data/Common/mandatory_Inp.ts` | `langs_Config.main_Lang`, `langs_Config.supported_Langs`, `supported_Url_Langs`, `is_Single_Lang`, BCP maps, OG locale maps. |

`mandatory_Inp.ts` is the only place that decides whether the project is single-lang or multi-lang. Other code consumes `is_Single_Lang`.

## Routing And Pages

| Surface | Implementation | Multi-lang mode | Single-lang mode |
|---|---|---|---|
| Bare root route | `src/pages/index.astro` | Deployment diagnostic only. Server config must redirect `/` to `/<main_Lang>/`. | Real public landing page at `/`. |
| Language-root landing | `src/pages/[lang]/index.astro` + `Landing_Page.astro` | Actual translated landing pages at `/<lang>/`. | May still be built at `/<main_Lang>/`, but treated as redundant to `/`. |
| Non-root routes | `src/pages/[lang]/...` | Public routes live under `/<lang>/...`. | Currently still emitted under `/<main_Lang>/...`; clean single-lang URLs require a separate routing migration. |
| Route-slug reverse index | `route_Slugs.ts` | All supported languages are indexed. | Must filter authored route slugs to `supported_Url_Langs` so inactive languages do not crash module load. |
| Content route helpers | `per_Lang_Slug.ts`, `routing_Factory.ts` | Expect one entry per supported language for translated pages. | Unsupported language folders must be removed from `src/Content/` or helpers throw intentionally. |

## SEO And Crawl Signals

| Surface | Implementation | Multi-lang mode | Single-lang mode |
|---|---|---|---|
| SEO head alternates | `Init_Layout.astro` | Emits hreflang alternates and `x-default`. | Skips hreflang alternates and `x-default`. |
| OG locale alternates | `Init_Layout.astro` | Emits `og:locale:alternate` for sibling languages. | No sibling languages, so no alternates. |
| Canonical URL | `Init_Layout.astro` | Canonical is the current `/<lang>/...` URL. | `/` is the canonical landing URL for the root page. Review duplicate `/<main_Lang>/` if needed. |
| Sitemap filter | `src/Scripts/Astro_Frontmatter/Astro_Config/sitemap_Integration.ts` | Excludes bare `/`; includes language roots. | Includes bare `/`; excludes `/<main_Lang>/`. |
| Robots root handling | `Root_Deployment_Error_Document.astro`, `robots.txt` | Diagnostic root carries `noindex, nofollow`; robots.txt needs no special `/` rule. | Do not put `noindex, nofollow` on `/`; it is the public landing page. |
| Lastmod page resolver | `Last_Mod/page_File.ts` | `index` maps to `src/pages/[lang]/index.astro`. | `index` maps to `src/pages/index.astro`. |
| Sitemap hreflang links | `sitemap_Integration.ts`, `Last_Mod/sister_Urls.ts` | Emits translated-slug alternates where siblings exist. | No sibling alternates in true single-lang content. |

## UI Chrome

| Surface | Implementation | Multi-lang mode | Single-lang mode |
|---|---|---|---|
| Desktop language switcher | `Top_Header_Main.astro` | Rendered. | Hidden. |
| Mobile language switcher | `Nav_Overlay_Zetun_Web.astro` | Rendered. | Hidden. |
| Language switcher data | `Page_Layout.astro`, `Init_Layout.astro` | Uses `full_Url_By_Lang` and `all_avail_langs`. | Still validated, but switcher UI is hidden. |

## Content Tree

Single-lang forks must slim content folders to match `supported_Url_Langs`. If `supported_Langs` contains only `de`, then collection folders should not keep active `en/` or `es/` siblings unless the project deliberately still builds those routes.

This is intentional. Hard failures from route/content helpers are signal that the language registry and content tree disagree.

## Review Items

These are not fully mode-owned today, but should be checked during a real single-lang production fork:

- JSON-LD identity URLs in `jsonLd_Local_Business.js` and `jsonLd_Person.js` currently point at `/${main_Lang}/`. If `/` is canonical in single-lang mode, schema identity URLs may need to point at `/`.
- Clean single-lang URLs such as `/kontakt/` instead of `/<main_Lang>/kontakt/` are not implemented by the current shared route tree. Treat that as a separate routing migration, not as an automatic language-mode toggle.

## Verification Checklist

After changing language mode:

1. Run `bun run build`.
2. Inspect `dist/sitemap-0.xml`:
   - multi-lang: no bare production root URL, language roots present.
   - single-lang: bare production root URL present, `/<main_Lang>/` absent.
3. Inspect one built page head:
   - multi-lang: hreflang alternates and `x-default` present.
   - single-lang: no hreflang alternates or `x-default`.
4. Inspect header/mobile nav:
   - multi-lang: language switcher visible.
   - single-lang: language switcher hidden.
5. Confirm content folders match `supported_Url_Langs`.
