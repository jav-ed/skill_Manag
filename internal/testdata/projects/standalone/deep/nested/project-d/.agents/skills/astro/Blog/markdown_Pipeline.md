# Markdown pipeline

The build-time chain that turns `.md` and `.mdx` files into rendered HTML. Two cooperating pieces: **Expressive Code** handles fenced code blocks (syntax highlighting, copy button, language badges, color chips) and **remark plugins** transform the markdown AST before Astro renders it (compute reading time, localise smart quotes).

Diagrams are handled outside this pipeline. Hand-author `.mmd` files, render to static SVG via `bun run mermaid:render` (`beautiful-mermaid` under `Code/Mermaid/`), embed via a `<figure class="diagram-static">` wrapper. No runtime mermaid library, no per-page JS cost. See [Scripts/build.md](../Scripts/build.md) and the static CLI under `Code/Mermaid/`.

Both are wired in `astro.config.mjs`. The wire order matters — see [Wire order](#wire-order-in-astroconfigmjs) below.

## Expressive Code (fenced code blocks)

[`astro-expressive-code`](https://expressive-code.com/) replaces Astro's default Shiki highlighting with a richer renderer: copy button, light/dark theme switching via CSS class, syntax-highlighted line numbers, frames around code, language labels, configurable wrap behaviour, custom Shiki language grammars.

### Minimal wiring

```js
// astro.config.mjs
import expressiveCode from "astro-expressive-code";

export default defineConfig({
  integrations: [
    expressiveCode({}),   // accepts a config object — most projects use the separate ec.config.mjs file instead
    /* …rest of integrations… */
  ],
});
```

### `ec.config.mjs` — the full config home

When config grows past a couple options, lift to `ec.config.mjs` at the repo root. `astro-expressive-code` auto-picks it up.

```js
// ec.config.mjs
import { defineEcConfig } from "astro-expressive-code";
import { pluginColorChips }   from "expressive-code-color-chips";
import { pluginLanguageBadge } from "expressive-code-language-badge";
import fs from "node:fs";

// custom Shiki grammar — see Custom languages below
const caddyfile_Grammar = {
  ...JSON.parse(fs.readFileSync("./src/Scripts/Build/Grammars/caddyfile.tmLanguage.json", "utf-8")),
  name: "caddy",
};

export default defineEcConfig({
  plugins: [
    pluginColorChips(),
    pluginLanguageBadge({
      textTransform: "uppercase",
      languageMap: { py: "Python", md: "Markdown", cpp: "C++" },
    }),
  ],

  defaultProps: {
    wrap: true,            // long lines wrap by default
    showLineNumbers: true,
    hangingIndent: 4,      // wrapped lines indent 4 spaces under their parent
  },

  themes: ["github-dark", "github-light"],

  // The theme is selected by which CSS class is on <html>. Pair with the project's
  // theme-bootstrap script (see ../Styles/tokens for the [data-theme] pattern).
  themeCssSelector: (theme) => `.theme-${theme.name}`,

  shiki: {
    langs:     [caddyfile_Grammar],
    langAlias: { caddyfile: "caddy" },
  },

  // Style the language badge with project design tokens so light/dark switching
  // is automatic — no hardcoded colors, no theme-array gymnastics.
  styleOverrides: {
    languageBadge: {
      fontColor:    "var(--muted-foreground)",
      background:   "var(--muted)",
      borderColor:  "var(--border)",
      borderWidth:  "1px",
      borderRadius: "0.25rem",
      fontSize:     "0.7rem",
    },
  },
});
```

### Useful plugins

- **`expressive-code-color-chips`** — renders a small circular color swatch inline next to any CSS color value (hex, rgb, hsl, oklch, named keywords). Only activates on CSS-dialect blocks; zero effect on others.
- **`expressive-code-language-badge`** — small label in the top-right corner of every code block showing the language. `languageMap` remaps fence identifiers to friendlier labels (`py` → `Python`).
- **`expressive-code-collapsible-sections`** — `// collapse from=20 to=40` directives in code blocks.
- **`expressive-code-twoslash`** — TypeScript twoslash hover types. Heavy; only worth it on TypeScript-tutorial blogs.

Plugins are opt-in. Don't add a plugin until a post genuinely benefits.

### Themes

EC ships with Shiki themes (`github-dark`, `github-light`, `dracula`, `solarized-dark`, etc.). For the dark/light split this org uses:

```js
themes: ["github-dark", "github-light"],
themeCssSelector: (theme) => `.theme-${theme.name}`,
```

Then in the project's theme-bootstrap script (see [`../Styles/linker_Styles.md`](../Styles/linker_Styles.md) § tokens), set the matching class on `<html>` before paint:

```html
<script is:inline>
  // pre-paint: set data-theme AND the theme-github-* class for Expressive Code
  document.documentElement.classList.add('theme-github-' + effective);
</script>
```

### Custom languages

When Shiki doesn't ship a grammar for a language the project uses (Caddyfile, custom DSLs), drop a TextMate grammar JSON under `src/Scripts/Build/Grammars/` and register it via `shiki.langs`:

```js
shiki: {
  langs:     [caddyfile_Grammar],
  langAlias: { caddyfile: "caddy" },   // accept ```caddyfile fences too
},
```

Source TextMate grammars from the language's VS Code extension or from `https://github.com/shikijs/textmate-grammars-themes/tree/main/packages/tm-grammars/grammars`. Treat the JSON as vendor — don't hand-edit.

## Custom remark plugins

Remark plugins run before Astro renders the markdown. They get the markdown AST (an `mdast` tree) and mutate it. Three patterns the org uses:

### `remark_Reading_Time`

Computes a minute count from the post body and stores it on the entry's frontmatter so the layout can display it with a localized unit:

```js
// src/Scripts/Build/MD_Plugins/remark_Reading_Time.js
import getReadingTime from "reading-time";
import { toString } from "mdast-util-to-string";

export function remarkReadingTime() {
  return (tree, { data }) => {
    const text = toString(tree);
    const reading_time = Math.max(1, Math.round(getReadingTime(text).minutes));

    const mdx_Info = data.astro.frontmatter.mdx_Info;
    if (mdx_Info) mdx_Info.reading_time = reading_time;
  };
}
```

The page template reads `remarkPluginFrontmatter.mdx_Info.reading_time` and renders it on the post card with the localized unit from `t_Reading_Time_Unit`. The value must stay a number. Do not store `reading-time`'s `.text` value, because that emits English strings like `"3 min read"` and breaks localized pages.

Note: the value lives on `remarkPluginFrontmatter` (Astro's special slot for remark-plugin-injected data), **not** on the regular `frontmatter` — different prop names because the data wasn't authored by hand.

### `remark_Locale_Quotes`

Astro's built-in SmartyPants converts straight quotes to English curly quotes (`"foo"` → `"foo"`). For multilingual sites that's wrong on German (`„foo"`), French/Spanish (`«foo»`), and Chinese (`「foo」`). This plugin runs *after* SmartyPants and replaces the English forms with locale-specific ones:

```js
// src/Scripts/Build/MD_Plugins/remark_Locale_Quotes.js
import { visit } from "unist-util-visit";

const locale_Quotes = {
  de: { open: "„", close: "“" },  // „ "
  fr: { open: "«", close: "»" },  // « »
  es: { open: "«", close: "»" },  // « »
  zh: { open: "「", close: "」" },  // 「 」
};

function detect_Locale(file_Path) {
  // derive lang from file path: /Content/blog/de/0001_foo.mdx → "de"
  // …
}

export function remarkLocaleQuotes() {
  return (tree, file) => {
    const lang = detect_Locale(file.path);
    if (!lang || !locale_Quotes[lang]) return;
    const { open, close } = locale_Quotes[lang];
    visit(tree, "text", (node) => {
      node.value = node.value
        .replace(/“/g, open)
        .replace(/”/g, close);
    });
  };
}
```

Targets text nodes only. Code blocks, inline code, HTML nodes are never visited — same boundary SmartyPants itself respects. Only double quotes; single quotes left alone to avoid collisions with apostrophes (`don't`, `it's`).

## Wire order in `astro.config.mjs`

```js
// astro.config.mjs
import { defineConfig } from "astro/config";
import remarkMath  from "remark-math";
import expressiveCode from "astro-expressive-code";

import { remarkReadingTime }  from "./src/Scripts/Build/MD_Plugins/remark_Reading_Time.js";
import { remarkLocaleQuotes } from "./src/Scripts/Build/MD_Plugins/remark_Locale_Quotes.js";

export default defineConfig({
  markdown: {
    // ORDER MATTERS:
    //   1. remarkReadingTime  — pure read; computes reading time from full text. Runs first so subsequent
    //                            transforms (quote swap) don't affect the word count.
    //   2. remarkMath          — needs the original math nodes; must run before any transform that
    //                            could rewrite them.
    //   3. remarkLocaleQuotes  — runs after SmartyPants (built in via Astro), which produces the English
    //                            curly quotes we then localise.
    remarkPlugins: [
      remarkReadingTime,
      remarkMath,
      remarkLocaleQuotes,
    ],
  },

  integrations: [
    // Expressive Code is an integration (not a remark plugin) — it runs after the remark chain.
    expressiveCode({}),
    // …rest of integrations…
  ],
});
```

## Add a new remark plugin — checklist

1. New file under `src/Scripts/Build/MD_Plugins/` named `remark_<Concern>.js` (or `.ts`).
2. Export a function that returns a transformer `(tree, file) => { /* mutate tree */ }`.
3. Use `unist-util-visit` for AST walking. Don't roll your own recursion.
4. Decide where it slots in the `remarkPlugins` array — runs in order, and order matters when one plugin's output is another's input.
5. Test against a real post with `bun run build`. Build errors surface here, not in dev (Astro's dev mode is partially cached).
6. Document in [Scripts/build.md](../Scripts/build.md) if the plugin is broadly useful; document in the project's blog docs if it's blog-specific.

## Add a new Shiki language — checklist

1. Source the TextMate grammar JSON. VS Code extensions are the usual source.
2. Drop it under `src/Scripts/Build/Grammars/<lang>.tmLanguage.json`. Treat as vendor; don't hand-edit.
3. Register in `ec.config.mjs` via `shiki.langs` (and `shiki.langAlias` if fences use a different identifier than the grammar's name).
4. Verify a `.mdx` post can fence a block with the language and that highlighting appears in `bun run build && bun run preview`.

## Debugging

- **Reading time is missing or has the wrong type** → check `remarkReadingTime` is in `remarkPlugins` and that the project's collection schema includes `mdx_Info` (the plugin writes to `data.astro.frontmatter.mdx_Info.reading_time`). The plugin must write a number, not `"X min read"`. Restart the dev server after changing remark plugins; Astro/Vite can keep the previous plugin module alive in dev.
- **Diagram looks wrong / not rendering** → diagrams are static SVGs from `bun run mermaid:render` (see `Code/Mermaid/`), embedded via `<figure class="diagram-static">`. Re-render the `.mmd` source after a font swap or token change; the SVG bakes in width metrics.
- **German post has English curly quotes** → `remarkLocaleQuotes` is missing, or the lang detection in the plugin doesn't recognise the project's file-path shape.
- **A new language fence renders unhighlighted** → grammar JSON path wrong, or `shiki.langs` doesn't include it, or fence identifier needs `langAlias`.
- **EC theme doesn't switch when toggling dark mode** → `themeCssSelector` doesn't match the actual class the project's theme-bootstrap script sets on `<html>`.
