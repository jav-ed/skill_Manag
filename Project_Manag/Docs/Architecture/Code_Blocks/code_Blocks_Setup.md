# Code Blocks Setup

## Package

`astro-expressive-code@0.41.7` — latest as of March 2026.

Not a replacement for Astro's built-in Shiki — a full UI layer on top of it with its own bundled Shiki instance (3.x). Astro 6's internal Shiki 4 upgrade does not affect code blocks rendered through this integration.

## Integration

Registered in `astro.config.mjs`. Must come **before** `mdx()`:

```js
import expressiveCode from "astro-expressive-code";

integrations: [
  expressiveCode({}),
  mdx({ extendMarkdownConfig: true }),
  // ...
]
```

Options are passed as empty `{}` — all config is delegated to `ec.config.mjs`.

## Config file

`ec.config.mjs` at project root:

```js
import { defineEcConfig } from 'astro-expressive-code'

export default defineEcConfig({
  defaultProps: {
    wrap: true,
    showLineNumbers: true,
    hangingIndent: 4,
  },
  themes: ["github-dark", "github-light"],
  themeCssSelector: (theme) => `.theme-${theme.name}`,
})
```

## Themes

Two themes are bundled: `github-dark` and `github-light`. Active theme is controlled by a CSS class on `<html>`:

| Class on `<html>` | Active theme |
|---|---|
| `.theme-github-dark` | Dark |
| `.theme-github-light` | Light |

## Stylesheet

`emitExternalStylesheet` defaults to `true` — styles are emitted as `/_astro/ec.{hash}.css` rather than inlined per page. Cached indefinitely by the browser if the hosting provider serves `/_astro/*` with `Cache-Control: public, max-age=31536000, immutable`.

## Available plugins (not enabled)

`@expressive-code/plugin-collapsible-sections` — lets long code blocks be collapsed. Uncomment in `ec.config.mjs` to enable:

```js
import { pluginCollapsibleSections } from '@expressive-code/plugin-collapsible-sections'

plugins: [pluginCollapsibleSections()],
```
