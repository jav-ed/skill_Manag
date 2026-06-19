# Charts pipeline — deep dive

How `<E_Chart>` actually works: which CSS variables it reads, which DOM contracts it depends on, how it stays in sync with theme + Reader Settings, and what breaks when something is renamed. Use this when modifying the component, debugging a chart that paints wrong, or adapting the pattern to a new project. For the author-facing flow (props, options, embed, pitfalls) see [charts.md](charts.md).

## Architecture overview

Charts are **fully resolved at runtime in the browser**. There is no CLI step, no committed render artefact, no Astro-build post-process. The component ships:

1. A `<figure>` with a chart container `<div>` and the serialised `options` written into a `data-echarts-options` attribute at frontmatter time.
2. A `<script>` block that imports `echarts`, reads the attribute, parses it with `JSON.parse`, registers a theme called `"jav"` against the project's CSS variables, and calls `echarts.init(container, "jav", { renderer: "svg" })`.

Everything else — theme adaptation, font swap, Reader-Settings reactivity, view-transition re-init — is hooks installed on the document by that same script.

Two runtimes are involved:

| Runtime | Lives in | Role |
|---|---|---|
| Astro build | `src/Components/Mdx/Import/E_Chart.astro` (frontmatter) | Stringifies the `options` object into `data-echarts-options` |
| Browser | `E_Chart.astro` (`<script>` block) | Parses the attribute, registers the theme, paints, installs the four live hooks |

ECharts SVG renderer is the default for this component. Canvas would also work but SVG keeps the chart in the same DOM-text-selection world as the rest of the article, and the SVG output is more screenshot-friendly.

## The five-dependency contract

The component reads from **five separate places** in the project at runtime. If any of these contracts drift, the chart silently degrades. Each is listed with the file that owns it, the surface the chart reads, and the failure mode if it's missing.

### 1. Font registry — the family the chart paints with

| Owned by | `src/Data/Common/font_Config.js` |
| Surface read | `getComputedStyle(.prose).fontFamily` |
| Failure mode | Chart paints in the browser default serif |

The chart never imports `font_Config.js` directly. It reads the resolved family from the `.prose` element via `getActiveFontFamily()` (E_Chart.astro:62-65). The chain that has to be intact:

1. `font_Config.js` declares `primary` with `astro_Css_Variable: "astro_Fnt_Primary"`.
2. `Init_Layout.astro` emits `<Font cssVariable={font_Role.css_Variable} />` for every configured role, so Astro injects `@font-face` rules and a `--astro_Fnt_Primary` variable on the page.
3. `fonts_Base.css:26` maps the bridge variable: `--font-primary: var(--astro_Fnt_Primary, var(--font-sans)), var(--font-sans);`.
4. `prose_Style.css:24` applies it: `.prose { font-family: var(--font-primary) }`.

Any of these breaking falls back to the next: a missing Astro variable falls through to `var(--font-sans)`, a missing `.prose` element falls to `"sans-serif"`. The chart never hard-errors, it just paints with whatever resolved.

### 2. Theme colour tokens — five values

| Owned by | `src/Styles/Base/theme.css` + `prose_Style.css` |
| Surface read | `--border`, `--card`, `--muted-foreground`, `--foreground` on `:root` / `[data-theme="dark"]`; `--tw-prose-body` on `.prose` |
| Failure mode | `cssVarToHex` returns `"#888888"` for the missing token; chart paints with no contrast |

The chart calls `cssVarToHex(name, element)` once per token. By default `element` is `document.documentElement` (the root-level theme tokens). `--tw-prose-body` is explicitly passed `.prose` because Tailwind Typography's scoped override (`prose_Style.css:30`) sets it via `color-mix(in oklch, var(--foreground) 85%, transparent)` *inside* the `.prose` block. Reading it from `:root` returns empty.

The five reads in `registerTheme()`:

```ts
const border    = cssVarToHex("--border");
const card      = cssVarToHex("--card");
const mutedFg   = cssVarToHex("--muted-foreground");
const fg        = cssVarToHex("--foreground");
const proseColor = prose ? cssVarToHex("--tw-prose-body", prose) : fg;
```

Renaming any of these tokens in `theme.css` silently breaks the chart's contrast. There is no schema enforcing the contract — it's pure name-match.

### 3. `data-theme` toggle script — the re-render trigger

| Owned by | `src/Layouts/Header/Top_Header_Main.astro` (inline pre-paint script) |
| Surface read | `MutationObserver` on `document.documentElement` attributes filtered to `data-theme` |
| Failure mode | Chart never re-renders on light/dark switch — stays on initial theme forever |

The chart installs a `MutationObserver` (E_Chart.astro:222-225) watching `<html>` for `data-theme` changes. The observer fires once at boot from the pre-paint script, and again every time the user toggles theme. Both fire `initCharts()` which calls `registerTheme()` and `setOption()` with fresh colours.

The pre-paint script is **inline** in `Top_Header_Main.astro:75-89`, intentionally not imported from `theme_State.ts`. It has to fire before any CSS loads (FART prevention), so duplicating its tiny logic on purpose is the design. If the page doesn't include the global header, the script doesn't run, the `data-theme` attribute is never set, and the chart never re-renders.

### 4. Reader-Settings protocol — the second re-render trigger

| Owned by | `src/Scripts/Browser_Client/Blog/reader.js` |
| Surface read | `[data-rs-font]` and `[data-rs-size]` click selectors; `localStorage("rs-font")`, `localStorage("rs-size")` |
| Failure mode | Font / size click doesn't re-render the chart; or `getActiveFontSize` reads the wrong key and falls back to defaults |

The chart installs a document-level click listener (E_Chart.astro:230-235):

```ts
document.addEventListener("click", (e) => {
  const target = e.target as Element;
  if (target.closest?.("[data-rs-font]") || target.closest?.("[data-rs-size]")) {
    setTimeout(() => initCharts(), 0);
  }
});
```

The protocol is exact-name. Renaming the attribute from `data-rs-font` to `data-rs-typeface` in either reader.js or the panel component silently breaks the click hook. The localStorage keys (`rs-font`, `rs-size`) are read directly in `getActiveFontSize()` (E_Chart.astro:69-76) — same exact-name contract.

The `setTimeout(0)` matters: element listeners (the button's own click handler in `reader.js`) run before document-level bubble listeners, so by the time the chart's listener fires, `localStorage` is already updated. The `setTimeout(0)` defers `initCharts` one more tick to be safe.

### 5. `.prose` element selector — the font/colour scope anchor

| Owned by | `Post_Layout.astro` (the article markup) |
| Surface read | `document.querySelector(".prose")` |
| Failure mode | `getActiveFontFamily()` returns `"sans-serif"`; `--tw-prose-body` reads empty and falls back to `--foreground` |

Three of the other contracts rely on a `.prose` element existing somewhere on the page. The chart uses `querySelector` (not `querySelectorAll`), so it reads from the **first** match — fine on a blog post where there's exactly one `<article class="prose">`. On pages without an `article.prose`, the chart still works but reads root-level tokens and the page's default font.

This is why charts work cleanly on blog posts and a bit worse anywhere else. If a project wants to use `<E_Chart>` on landing pages or marketing pages, either wrap a `.prose` element around it or accept the degraded behaviour.

## The canvas colour-conversion trick (`cssVarToHex`)

ECharts only accepts hex or `rgb(...)` strings for colour values. The project's theme tokens are `oklch(...)` and `color-mix(in oklch, ...)`. The bridge is `cssVarToHex` (E_Chart.astro:50-60):

```ts
function cssVarToHex(cssVar: string, element: Element = document.documentElement): string {
  const raw = getComputedStyle(element).getPropertyValue(cssVar).trim();
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = 1;
  const ctx = canvas.getContext("2d");
  if (!ctx) return "#888888";
  ctx.fillStyle = raw;                  // canvas accepts any valid CSS color
  ctx.fillRect(0, 0, 1, 1);
  const [r, g, b] = ctx.getImageData(0, 0, 1, 1).data;
  return "#" + [r, g, b].map(n => n.toString(16).padStart(2, "0")).join("");
}
```

The canvas's `fillStyle` setter accepts **every** valid CSS colour: hex, named, `rgb`, `hsl`, `oklch`, `color-mix`, even calc'd values. After `fillRect`, the pixel data is read back as RGB — the canvas has done the parsing for us. We then re-encode as hex for ECharts.

The fallback `"#888888"` exists for the (rare) case where `getContext("2d")` returns null. Browsers will return null when running in a restricted context like an HTML-to-PDF renderer. The fallback is neutral enough to not look catastrophic on either theme.

**Subtle risk to flag:** the trick depends on the browser's canvas implementation continuing to accept `oklch` and `color-mix` as `fillStyle`. If a browser ever stops accepting these (unlikely — they're CSS Color 4 standard), the chart silently paints `#888888` for every token. This is the kind of regression that would only show up in a screenshot diff.

## The four live-update hooks

The component installs four hooks that all call `initCharts()`. Their sources, in the order they appear in `E_Chart.astro`:

| Source | Line | Why |
|---|---|---|
| `window.resize` listener | 219 | Calls `instance.resize()` on every chart so SVG geometry tracks viewport |
| `MutationObserver` on `data-theme` | 222-225 | Theme toggle re-runs `registerTheme()` against new token values |
| Document `click` on `[data-rs-font]` / `[data-rs-size]` | 230-235 | Reader Settings font/size change re-renders with new family/size |
| `document.addEventListener("astro:after-swap", …)` | 237 | View-transition swap re-initialises every chart on the new page |

The `window.resize` hook calls `instance.resize()` (cheap, just re-layouts), the other three call `initCharts()` (rebuild from `data-echarts-options`). Each `initCharts()` disposes the previous instance for the same id before re-initialising (E_Chart.astro:149-152) so the instance map stays bounded.

## Hardcoded knobs

These are constants inside `E_Chart.astro`. Tuning them requires editing the file — they are not derived from any config:

| Constant | Value | Effect |
|---|---|---|
| `CHART_BASE` | `12` | Baseline font size in px for axis labels, legend, tooltip. Prose body is too large (~18–20px) for chart text. |
| `SIZE_SCALE` | `{ s: 0.85, m: 1, l: 1.2 }` | Reader-Settings size multiplier. Mirrors `reader.js` values exactly. |
| `FONT_SCALE.mono` | `0.9` | Extra compensation when reader chooses mono — same as `reader.js`'s `FONTS.mono.sizeScale`. |
| `EXTERNAL_LABEL_PX` | `90` | Fixed margin reserved on any side that has external series labels. |
| `EXTERNAL_LABEL_CONTENT_PX` | `EXTERNAL_LABEL_PX - 16` | Wrap width for labels inside the reserved margin. The 16px is a visual buffer. |
| Default palette | `color: [mutedFg]` | Single-colour monochromatic default. Authors override per-series. |

`SIZE_SCALE` and `FONT_SCALE.mono` duplicate `reader.js`'s constants on purpose — both files have to agree on what `s/m/l` and `mono` mean. Changing one without the other will desync chart text from body text on the same page.

## How author overrides layer

The chart accepts the author's `options` object verbatim, with three patches applied at `initCharts()` time:

1. **Series labels** are deep-patched (E_Chart.astro:178-197). Author's `label` keys win for everything except `fontFamily`, `fontSize`, `textBorderWidth`, and `textBorderColor` — those are forced to the theme's resolved values. External-position labels additionally get `overflow: "break"` and `width: EXTERNAL_LABEL_CONTENT_PX` injected (unless the author set them).
2. **Grid margins** are auto-filled (E_Chart.astro:201-205). For every side that has an external label, the chart writes `${EXTERNAL_LABEL_PX}px` if the author didn't set that side explicitly. Author values win.
3. **Tooltip** defaults to `{ trigger: "axis" }` (E_Chart.astro:207-212). Author can override by writing their own `tooltip` block in `options`.

Other top-level keys (`xAxis`, `yAxis`, `series` types other than labels, `legend`, `title`) flow through unchanged. Series-level `color` is passed through verbatim — author hex strings reach ECharts directly.

## What breaks if X is renamed

The five-dependency contract is enforced only by string matching. Concrete failure modes:

| Rename | Symptom |
|---|---|
| `astro_Fnt_Primary` → anything else (in `font_Config.js`) | Chart paints in `var(--font-sans)` system stack instead of Platypi |
| `--font-primary` → anything else (in `fonts_Base.css` or `prose_Style.css`) | Chart paints in browser default; whole `.prose` font also broken |
| `--border`, `--card`, `--muted-foreground`, `--foreground` → anything else | Affected axis/tooltip/series turns `#888888` |
| `--tw-prose-body` removed from `.prose` | Axis-label colour falls back to `--foreground` (full opacity instead of 85%) |
| `data-theme` attribute moved off `<html>` | Light/dark toggle never re-renders the chart |
| `data-rs-font` / `data-rs-size` renamed | Reader-Settings click hook breaks |
| `rs-font` / `rs-size` localStorage keys renamed | `getActiveFontSize` reads `null` and falls back to defaults |
| `.prose` class removed from the article wrapper | Font + `--tw-prose-body` both fall back |

There is no test catching these. If you rename any of the above, grep `E_Chart.astro` for the old string and update both ends.

## Porting to a new project — checklist

For a new repo that wants to adopt `<E_Chart>`:

1. **Font system in place.** Confirm `src/Data/Common/font_Config.js` declares a primary role with a non-null `astro_Css_Variable`. Confirm `Init_Layout.astro` emits `<Font cssVariable={...} />` for it. Confirm `fonts_Base.css` maps `--astro_Fnt_Primary` (or the project's equivalent) to whatever the `.prose` rule consumes. See [Fonts setup](../Fonts/setup.md).
2. **Theme tokens defined.** `theme.css` must declare `--border`, `--card`, `--foreground`, `--muted-foreground` for both light and dark themes. Names exact. See [Styles tokens](../Styles/tokens.md).
3. **Tailwind Typography wired.** `prose_Style.css` (or equivalent) must scope `--tw-prose-body` under `[data-theme] & .prose.prose { … }` and wire it to the project's foreground token. See [Typography](typography.md).
4. **Pre-paint theme script in place.** The inline script in `Top_Header_Main.astro` (or wherever the project's header sits) must set `data-theme` on `<html>` before CSS loads. See [Styles tokens § pre-paint script](../Styles/tokens.md).
5. **Reader Settings wired** (only if Reader Settings is shipping on this project). `reader.js` and the panel component must use exactly `rs-font` / `rs-size` localStorage keys and `data-rs-font` / `data-rs-size` button attributes. See [Reader Settings](../Fonts/reader_Settings.md).
6. **Copy the component.** `src/Components/Mdx/Import/E_Chart.astro` is the artefact. Drop it in. The numeric knobs (`CHART_BASE`, `EXTERNAL_LABEL_PX`, palette) are project-tunable.
7. **Register the component** in the project's MDX map (`src/Components/Mdx/mdx_Components.ts`) under the key `E_Chart`. See [MDX component map](MDX_Components/component_Map.md).
8. **Add `echarts` to dependencies.** `bun add echarts`.

Steps 1–5 are the contract surface. If the project's design system uses different token names, you have two options: rename them to match the contract (cleanest, smallest diff in `E_Chart.astro`), or fork the constants at the top of the component (clearest, but a maintenance fork).

## Why the chart isn't an external worker

Unlike OG image generation and Mermaid rendering, ECharts runs in the browser. A few reasons:

- **Interactivity.** Tooltips, legend toggles, zoom — those need a live chart instance. A static pre-render would lose them.
- **Theme + Reader Settings reactivity.** The chart has to respond to runtime CSS-variable changes. A build-time render couldn't track `data-theme` or `[data-rs-*]` clicks.
- **SVG output.** ECharts' SVG renderer produces clean markup that respects DOM accessibility and text selection. Canvas-rendered charts would be opaque to a screen reader.
- **Bundle cost is acceptable here.** ECharts is ~1MB unminified but tree-shakes well and is loaded only on pages that import it. Static-only diagrams ship zero runtime; data charts pay for what they buy.

If a future project decides this trade-off is wrong (e.g. AMP-style pages with a hard JS budget), the option is to render charts to PNG at build time the way OG images are generated. The pipeline-internals pattern is documented in [`../Scripts/og_Images_Pipeline.md`](../Scripts/og_Images_Pipeline.md); the author-facing usage view is in [`../Scripts/og_Images.md`](../Scripts/og_Images.md).
