# Hidden Features

Hidden features are user-facing only when something unusual happens: missing pages, fallback flows, browser-level handoffs, and other surfaces that normal navigation does not expose. They still need the same routing, translation, SEO, design, and deploy discipline as ordinary pages.

## Routing

- [Custom 404 pages](404/linker_404.md): how to create Astro-backed 404 pages, translate their copy, exclude them from indexing, make Caddy serve the right localized body while preserving the HTTP 404 status, and recognize cases where that approach needs adjustment.

## When to add a hidden feature here

Add a subfolder when the feature crosses more than one normal owner. A 404 page is not just a route: it touches Astro routing, multilingual copy, sitemap policy, and Caddy error handling, so it deserves one hidden-feature entry that points to each owner instead of scattering the whole workflow across unrelated docs.
