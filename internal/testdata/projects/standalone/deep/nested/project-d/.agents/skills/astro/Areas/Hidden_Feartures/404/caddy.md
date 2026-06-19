# Serve The 404 Page With Caddy

Astro emits the 404 HTML files. Caddy decides which one a visitor receives for a missing URL.

The rule: serve the localized 404 body while preserving the HTTP `404` status. Do not redirect the visitor to `/de/404/`; a missing URL should remain the URL in the address bar and return status `404`.

Caddy's `handle_errors` is the right layer for this. Caddy documents that `file_server` preserves the error status code when it runs inside `handle_errors`, assuming the site root is already set.

## Generated files

A multilingual Astro build should emit:

```txt
dist/404.html
dist/de/404/index.html
dist/en/404/index.html
dist/es/404/index.html
```

The exact language folders come from the project language registry.

## Caddy shape

Inside the site block, keep the normal static-file setup and add a `handle_errors` block:

```caddyfile
example.com {
    root * /srv/web/example.com/main/current
    encode zstd br gzip

    handle_errors 404 {
        @de path /de /de/*
        handle @de {
            rewrite * /de/404/
            file_server
        }

        @en path /en /en/*
        handle @en {
            rewrite * /en/404/
            file_server
        }

        @es path /es /es/*
        handle @es {
            rewrite * /es/404/
            file_server
        }

        handle {
            rewrite * /404.html
            file_server
        }
    }

    file_server
}
```

Use the copyable starter at [Templates/caddy_Localized_404.caddy](Templates/caddy_Localized_404.caddy).

## Subpath deployments

If a site is mounted under a path prefix such as `/partner/`, check where the prefix is stripped:

- If the Caddy site uses `handle_path /partner/*`, matching happens after the prefix is stripped. The template can still match `/de/*`, `/en/*`, etc.
- If the prefix is not stripped, the matchers need to include the prefix, e.g. `/partner/de/*`.

Do not guess. Verify with `curl -i` against a missing URL under the real deployed path.

## Verification

After editing the project-local Caddy file, validate before reload:

```sh
ssh <deploy-host> 'sudo caddy validate --config /etc/caddy/Caddyfile --adapter caddyfile'
ssh <deploy-host> 'sudo systemctl reload caddy'
```

Then verify status and body:

```sh
curl -i https://example.com/de/not-real
curl -i https://example.com/en/not-real
curl -i https://example.com/es/not-real
curl -i https://example.com/not-real
```

Expected:

- HTTP status is `404`,
- the `/de/...` body is German,
- the `/en/...` body is English,
- the `/es/...` body is Spanish,
- the root/unknown prefix fallback serves `/404.html`,
- the browser URL remains the originally requested missing URL.

## Project documentation

Store the real Caddy file in the project's own docs, then copy it to the server. For repos using this org's docs layout, the convention is:

```txt
Project_Manag/Docs/Architecture/CI_CD/Caddy/<site>.caddy
```

The skill template is generic. The project file is the source of truth for the real domain, real language list, and real deploy paths.
