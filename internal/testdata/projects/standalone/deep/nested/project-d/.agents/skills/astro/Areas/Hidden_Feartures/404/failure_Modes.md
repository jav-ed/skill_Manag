# 404 Failure Modes

Custom 404 handling only works when three layers agree:

- Astro must emit the 404 files you expect,
- the deploy host must treat missing URLs as missing,
- Caddy or the hosting platform must serve the custom body while preserving status `404`.

If any layer rewrites, redirects, or claims the route first, the page can look correct in local navigation and still fail for real missing URLs.

## Catch-all Astro routes claim `/404`

This is the main case that makes `src/pages/404.astro` unreliable.

Check for root rest routes such as:

```txt
src/pages/[...index].astro
src/pages/[...slug].astro
src/pages/[...path].astro
```

In static builds, a rest route can claim `/404` before the literal 404 page emits the expected artifact. Symptoms:

- `dist/404.html` is missing,
- `dist/404.html` contains a redirect or the wrong page,
- `prerenderConflictBehavior: "error"` does not report a conflict,
- the deployed error page is the catch-all fallback instead of the custom 404.

First try to remove the catch-all or change it into explicit dynamic routes. If the catch-all is required for the project, treat a standalone static `Public/404.html` served through Caddy `handle_errors` as a valid exception to the normal "prefer Astro routes" rule.

Always inspect the emitted artifact:

```sh
bun run build
test -f dist/404.html
sed -n '1,80p' dist/404.html
```

## The server never produces a 404

Caddy `handle_errors 404` only runs when the normal handler returns a 404. It will not fix a deployment that rewrites every unknown URL to an existing file with status `200`.

Common causes:

- SPA fallback rules that rewrite missing URLs to `/index.html`,
- broad `rewrite` rules before `file_server`,
- reverse proxies that return their own 404 body,
- application servers that redirect missing URLs to `/404/`,
- middleware that catches every path and returns `200`.

Verify the real deployed URL, not only `/404/`:

```sh
curl -i https://example.com/de/not-real
```

Expected: the address stays `/de/not-real`, the status is `404`, and the body is the localized 404 page.

## The deployment is not static Astro behind Caddy

The Caddy template assumes a static Astro build where Caddy serves files from `dist/`.

Do not copy it unchanged for:

- SSR or hybrid Astro adapters,
- Netlify, Vercel, Cloudflare Pages, or other managed hosts with their own 404 conventions,
- deployments where another server sits in front of Caddy and handles errors first,
- projects mounted under a non-stripped base path without matching Caddy path rules.

For those cases, keep the Astro page/component pattern, but follow the host's own 404/error routing contract.

## Localized 404 files were never generated

`src/pages/[lang]/404.astro` still needs valid static paths. If the route does not use the project language registry correctly, Caddy can rewrite to a file that does not exist.

Check after every build:

```sh
test -f dist/404.html
test -f dist/de/404/index.html
test -f dist/en/404/index.html
test -f dist/es/404/index.html
```

Adapt the language folders to the project. Missing files usually mean the route has no `getStaticPaths`, an incomplete language list, or an adapter/prerender setting that changed what Astro emits.

## Caddy matches the wrong path

Subpath deployments and `handle_path` can change what Caddy sees before the `handle_errors` block runs.

If the site is mounted at `/partner/`, determine whether the prefix is stripped:

- stripped prefix: match `/de/*`, `/en/*`, etc.,
- unstripped prefix: match `/partner/de/*`, `/partner/en/*`, etc.

Do not infer this from local Astro routes. Verify with `curl -i` against a missing deployed URL under the real prefix.

## Static public file is in the wrong public directory

When using the fallback `Public/404.html` approach, check `astro.config.mjs`.

Astro defaults to lowercase `public/`, but some repos in this org use:

```js
publicDir: "Public"
```

Put the standalone file in the configured directory, then verify that it lands at:

```txt
dist/404.html
```

## Minimum preflight

Before declaring a custom 404 done:

```sh
rg -n "\\[\\.\\.\\.|rewrite \\* /index\\.html|handle_errors|file_server|publicDir|prerender" src astro.config.mjs Project_Manag .woodpecker.yml
bun run build
test -f dist/404.html
rg '/404|404\.html' dist/sitemap*.xml
```

Then verify production or the closest deployed preview with `curl -i` against missing URLs in each supported language and one unsupported prefix.
