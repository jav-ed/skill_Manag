# Create The Astro 404 Page

Astro will build a custom 404 page when a project defines `src/pages/404.astro` or `src/pages/[lang]/404.astro`. Most deploy services can discover the root `404.html`; language-aware Caddy handling can additionally serve `/<lang>/404/` for missing localized URLs.

## Route files

Use this shape for a multilingual static site:

```txt
src/pages/404.astro
src/pages/[lang]/404.astro
src/Components/Pages/Error_404/Error_404_Document.astro
src/Scripts/Multi_Lang_Txts/Pages/error_404_Txt.ts
```

The route files should stay thin. They pick the language, collect the project data, and render the shared 404 document/component. The visual design and layout live in the component folder, not duplicated across route files.

Do not create `src/pages/[lang]/[404].astro`. That makes `404` a dynamic param name, not a literal route segment. The correct literal localized route is:

```txt
src/pages/[lang]/404.astro
```

## Root fallback

Keep `src/pages/404.astro` even in a multilingual site. It gives static hosts and root-level Caddy fallback a normal `/404.html` artifact, and it covers URLs that do not start with a supported language prefix.

Typical behavior:

- `/de/kaputt/` -> serve `/de/404/` with status `404`
- `/en/missing/` -> serve `/en/404/` with status `404`
- `/unknown/missing/` -> serve `/404.html` with status `404`

The language-specific selection happens in Caddy; see [Serve it with Caddy](caddy.md).

## Translation copy

Use `src/Scripts/Multi_Lang_Txts/Pages/error_404_Txt.ts` or the closest existing page-copy folder in the project. The full translation-table rules are already covered in [Scripts/Multi_Lang_Txts](../../../Scripts/multi_Lang_Txts.md); do not re-invent them here.

Minimum expectations:

- one exported object for the user-facing 404 copy,
- all supported languages covered,
- `assert_All_Langs(...)` called for every exported translation object,
- English programmer-facing keys,
- no hardcoded German/English strings inside the route wrapper.

## SEO policy

A 404 page should be useful to humans and invisible to search results.

Add:

```html
<meta name="robots" content="noindex" />
```

If the project uses a shared `Layout`, pass the equivalent page prop instead of hardcoding the tag. If the 404 page is a standalone document, include the tag directly in the document head.

Also exclude all 404 routes from the sitemap:

```js
filter: (page) => {
  const url = new URL(page);
  const path = url.pathname;

  if (path === "/404" || path === "/404/" || path === "/404.html") return false;
  if (/^\/(de|en|es)\/404\/?$/.test(path)) return false;

  return true;
}
```

Adapt the language list to the project's registry. The broader sitemap policy lives in [Sitemap and robots](../../../SEO/sitemap_And_Robots.md).

## Visual standard

Treat the page as a premium recovery surface:

- clear brand at the top,
- one unmistakable error statement,
- short explanation,
- two or three useful onward paths,
- no marketing section,
- no playful copy unless the brand already speaks that way,
- full dark/light theme support when the site supports themes,
- responsive mobile layout checked directly.

The page should feel calm and intentional, not like a CMS default.

## Verification

Run:

```sh
bun run build
```

Then check the emitted files:

```sh
test -f dist/404.html
test -f dist/de/404/index.html
test -f dist/en/404/index.html
test -f dist/es/404/index.html
```

Adapt the language list for the project.

Check sitemap exclusion:

```sh
rg '/404|404\.html' dist/sitemap*.xml
```

Expected result: no output.

Finally, inspect at least one desktop and one mobile viewport. Check that the headline does not overlap surrounding content, buttons fit, and the theme switcher/header state matches the rest of the site.
