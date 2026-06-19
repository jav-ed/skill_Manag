# Custom 404 Pages

A custom 404 page is a brand surface, not a throwaway error screen. It should be built through Astro when the project already has Astro routes, theme tokens, fonts, translation tables, and shared page components.

## What belongs here

- [Create the Astro 404 page](custom_404_Page.md): route files, component placement, translation table placement, noindex/sitemap checks, and build verification.
- [Serve it with Caddy](caddy.md): how Caddy selects the localized 404 body for `/de/...`, `/en/...`, `/es/...`, while keeping the response status as `404`.
- [Failure modes](failure_Modes.md): cases where route-based 404s, Caddy handling, localized files, or deployment topology can be expected not to work without adjustment.
- [Caddy localized 404 template](Templates/caddy_Localized_404.caddy): copyable starter snippet for a static Astro site behind Caddy.

## Decision

Prefer Astro routes over a standalone `Public/404.html` when the project has an Astro app shell. A public static file can work for a minimal one-off site, but it cannot naturally consume:

- the language registry,
- translation validation,
- shared components,
- theme and style tokens,
- Astro Fonts API setup,
- the same design system used by the rest of the site.

Use a static `Public/404.html` only when there is intentionally no Astro app layer to reuse, or when [known route/deploy constraints](failure_Modes.md) make an Astro-owned 404 unreliable.

## Related docs

- [Page routes](../../page_Routes.md) — ordinary route-file naming and `getStaticPaths` rules.
- [Scripts/Multi_Lang_Txts](../../../Scripts/multi_Lang_Txts.md) — where translated UI/page strings live.
- [Sitemap and robots](../../../SEO/sitemap_And_Robots.md) — route exclusion, noindex policy, and verification.
- [CI/CD application integration](../../../../ci-cd/Application/Integration/first_Setup.md) — where the project-local Caddy file should be stored and deployed.
- [CI/CD application active use](../../../../ci-cd/Application/Active_Use/daily_Workflow.md) — Caddy validation, reload, and curl checks.
