# Fonts

Astro projects in this org load fonts through Astro's Fonts API exclusively. The API is configured in `astro.config.mjs` under the top-level `fonts: [...]` array, and consumed in `<head>` via `<Font cssVariable="..." preload={...} />` (imported from `astro:assets`). The API handles fetching, subsetting, self-hosting, and `<link rel="preload">` emission. No manual `<link rel="stylesheet">` to Google/jsDelivr CSS, no inline `@import url(...)`, and (with one exception, see below) no hand-written `@font-face`.

Reference implementations: `Voice/astro.config.mjs` (the gold standard — uses both providers in one config) and Partner/Jav_Web (Fontsource-only).

## No Google Fonts, ever

Never `fontProviders.google()`. Never `<link href="https://fonts.googleapis.com/...">`. Never `@import url("https://fonts.googleapis.com/...")` or `https://fonts.gstatic.com/...`. Every typeface request to Google's CDN sends the visiting IP address to Google, which under EU/GDPR is an unlawful transfer of personal data to a third party without explicit consent — German courts have ruled individual sites liable for this. The rule is absolute: not in production, not in dev, not in a fallback. Fontsource ships the same open-source faces (Public Sans, Spectral, Lora, Crimson Pro, IBM Plex, Geist, Jakarta Sans, Maple Mono, Instrument Sans, Newsreader, etc.) packaged for self-hosting; it is the only third-party provider this org uses.

Also forbidden: any runtime font loader that fetches a font file from an external URL when a real user loads the page, regardless of which URL. If you find yourself reaching for a Google Fonts URL, find the matching `@fontsource/<family>` package instead. If the face is not on Fontsource, ship it as a local font (next section).

## Two providers, one API

Both are imported from `astro/config`:

```js
import { defineConfig, fontProviders } from "astro/config";
```

### 1. `fontProviders.fontsource()` — for open-source faces

For any face on the Fontsource registry. Astro downloads the package at build time (from jsDelivr's npm mirror), subsets it, and emits self-hosted files into `dist/_astro/`. Visitors never hit a third-party server.

```js
fonts: [
  {
    provider: fontProviders.fontsource(),
    name: "Public Sans",
    cssVariable: "--astro_Fnt_Public_Sans",
    weights: ["100 900"],  // variable font — one file, full weight axis
  },
]
```

If the face is a **variable font** (most modern Fontsource faces are), use a weight range string `"100 900"` — Astro downloads one file covering the full axis instead of separate static files per weight. For non-variable fonts, list individual weights: `weights: [400, 600, 700]`.

### 2. `fontProviders.local()` — for files shipped under `src/Assets/Fonts/`

For faces Fontsource does not carry: licensed commercial fonts, in-house custom faces, niche scripts. Drop the `.woff2` files under `src/Assets/Fonts/<Family>/` (see [`../Areas/assets.md`](../Areas/assets.md) for the folder convention) and register each variant explicitly. The `local()` provider supersedes hand-written `@font-face` — never `@font-face` a local file directly when this provider is available.

```js
fonts: [
  {
    provider: fontProviders.local(),
    name: "Fnt_Al_Qalam_Quran_Regular",
    cssVariable: "--astro_Fnt_Al_Qalam_Quran_Regular",
    options: {
      variants: [
        {
          weight: "400",
          style: "normal",
          src: ["./src/Assets/Fonts/Quran/Al_Qalam_Quran_Regular.woff2"],
          display: "swap",
        },
      ],
    },
  },
]
```

Each variant in `variants[]` becomes one emitted face. Add more entries for italic/bold/etc. — there is no `weights: []` shortcut for local fonts; the API needs the file path per variant.

Real-world example: `Voice/astro.config.mjs` configures three Fontsource families (Maple Mono, Instrument Sans, Newsreader) plus one local family (`Fnt_Al_Qalam_Quran_Regular`) in the same `fonts: [...]` array — the providers compose freely.

## `cssVariable` naming convention

Every entry must declare a `cssVariable`. The org-wide convention is:

```
--astro_Fnt_<Family_Name>
```

- Prefix `astro_Fnt_` makes it obvious in DevTools that the var is owned by Astro's Fonts API and not a Tailwind-defined token.
- Family name in `PascalCase_Underscored` (e.g. `Public_Sans`, `Maple_Mono`, `Al_Qalam_Quran_Regular`) to match the project-wide `improved_Camel_Snake` convention.

The variable surfaces in CSS as `var(--astro_Fnt_Public_Sans)`. Tailwind picks it up through `@theme` in the entry stylesheet:

```css
@theme {
  --font-sans: var(--astro_Fnt_Public_Sans), system-ui, sans-serif;
}
```

After that, `font-sans` works as a standard Tailwind utility. Same pattern for `--font-serif`, `--font-mono`, etc.

## Variable chain — from Astro to your components

This section explains the **pattern**, not a specific font palette. Each project picks its own families; the chain shape stays the same. Family names below are placeholders (`<Heading_Font>`, `<Body_Font>`, `<Mono_Font>`) — substitute whatever the project actually registers in `astro.config.mjs`.

The minimal chain has two layers (Astro var → Tailwind `@theme` var), and that is what Partner and Voice use today. For projects that need to swap fonts at runtime (Reader Settings, see below) or read the font name from JS, extend it to four:

```
--astro_Fnt_<Heading_Font>     Layer 1: injected by Astro <Font /> (runtime, opaque)
       ↓
--fnt_<Heading_Font>           Layer 2: bridge var, set inline in the root Layout.
       ↓                                 Decouples CSS from Astro-specific names.
                                         Also: JS can read this with getPropertyValue.
--font-<heading_font>          Layer 3: Tailwind @theme var (font-specific, rarely used directly)
       ↓
--font-primary                 Layer 4: semantic token — use this everywhere in components
```

**Two-layer (default).** Map the Astro var directly into a Tailwind token. Use when there is no runtime font switching and no JS that reads font names. Generic shape:

```css
/* src/Styles/Base/themes.css */
@theme {
  --font-sans: var(--astro_Fnt_<Body_Font>), system-ui, sans-serif;
}
```

**Four-layer (for runtime switching / JS reads).** Insert a `--fnt_<Family>` bridge var in the root layout, then map the bridge through a font-specific Tailwind var and finally to a semantic token. Generic shape:

```astro
{/* src/layouts/Layout.astro — bridge layer */}
<style set:html={`
  :root {
    --fnt_<Heading_Font>:  var(--astro_Fnt_<Heading_Font>);
    --fnt_<Body_Font>:     var(--astro_Fnt_<Body_Font>);
    --fnt_<Mono_Font>:     var(--astro_Fnt_<Mono_Font>);
  }
`} />
```

```css
/* src/Styles/Base/fonts.css */
@theme {
  --font-<heading_font>:  var(--fnt_<Heading_Font>);
  --font-<body_font>:     var(--fnt_<Body_Font>);
  --font-mono:            var(--fnt_<Mono_Font>);

  --font-primary:         var(--fnt_<Heading_Font>);  /* mapping is project-defined */
  --font-secondary:       var(--fnt_<Body_Font>);     /* mapping is project-defined */
}
```

The semantic-to-actual mapping (`--font-primary` → which family) is **each project's design choice**. A monochrome-typography project might map both `--font-primary` and `--font-secondary` to the same family. An editorial project might map primary to a display serif, secondary to a humanist sans. The skill doesn't prescribe; it just says: components reference semantic tokens, and the mapping lives in one stylesheet so swapping is one edit.

The bridge layer earns its complexity in four cases:

1. **Keeping `@theme` clean of Astro internals.** The bridge gives Tailwind a stable name to map (`var(--fnt_<Family>)`), so the `@theme` block never references `--astro_Fnt_*` directly. Astro is then free to rename its injected var across versions without forcing a rewrite of the Tailwind token layer. This is also the lever that makes Tailwind the preferred API for component authors (next section).
2. **A runtime font switcher** (e.g. Reader Settings) needs a stable name to override on `:root`. Overriding the Astro var directly is fragile; the bridge is stable across Astro versions.
3. **JS that extracts a font name** for a third-party API — e.g. drawing to canvas, configuring a charting library, feeding a runtime diagram lib. `getComputedStyle().getPropertyValue('--font-primary')` returns the raw string `var(--fnt_<Heading_Font>)`, not the resolved font name, because `getPropertyValue` does not follow `var()` chains. The bridge var resolves to an actual string Astro injected, which JS can use directly.
4. **Changing the typography palette without touching every component** — components reference `--font-primary` (semantic) only; the mapping from semantic to actual font lives in one stylesheet.

CSS-only consumption (Tailwind utilities, hand-written `var(--font-primary)`) is unaffected by the chain depth — the browser resolves the chain when applying styles.

## Semantic tokens — what to reference in components

When the four-layer chain is in place, the semantic tokens are the only names that should appear in component markup or stylesheets. Don't reference `--font-<specific_family>` or `--astro_Fnt_*` directly outside the entry stylesheet — that hard-codes the typography choice into the component and defeats the swap-in-one-place property.

| Token | Role | Project decides |
|---|---|---|
| `--font-primary` | Headings, display titles, editorial accents | Which family fills the role |
| `--font-secondary` | Body copy, UI chrome, navigation | Which family fills the role |
| `--font-mono` | Code blocks (usually emitted by Expressive Code, not set by hand) | Which mono family |
| `--font-<script>` | Script-specific (e.g. `--font-al_qalam` for Qur'anic Arabic) | Add tokens when a script needs its own face |
| `--font-modern` | Reader Settings Modern slot | Fixed category: sans-serif. Which sans face feeds it is routed in `fonts_Base.css`. |
| `--font-elegant` | Reader Settings Elegant slot | Fixed category: serif. Which serif face feeds it is routed in `fonts_Base.css`. |
| `--font-precise` | Reader Settings Precise slot | Fixed category: monospace. Which mono face feeds it is routed in `fonts_Base.css`. |

The set of semantic tokens is also project-defined; primary/secondary/mono is a reasonable baseline but a project can add `--font-display`, `--font-quote`, etc. when the design calls for more roles.

The three Reader Settings slot tokens are a separate concern: their identity (slot=category) is fixed across projects, and only the wiring from slot to physical face varies. See [`bootstrap.md`](./bootstrap.md) for the slot-routing rules.

The default `html { font-family: var(--font-secondary), ...; }` rule in `src/Styles/Base/` makes `--font-secondary` the inherited base — body copy needs no explicit class.

### Prefer Tailwind utilities over raw `var()` in components

Each semantic token in `@theme` automatically becomes a Tailwind utility class — `--font-primary` → `font-primary`, `--font-secondary` → `font-secondary`, `--font-mono` → `font-mono`, etc. **In component markup and component-scoped styles, always reach for the utility class first**:

```astro
{/* Good — reads like every other token in the codebase */}
<h1 class="font-primary">…</h1>
<p class="font-secondary">…</p>

{/* Avoid — works, but is heavier and breaks the local convention */}
<h1 style="font-family: var(--font-primary)">…</h1>
```

The utility-first preference is org-wide for the same reason it applies to colors (`bg-primary`, not `style="background: var(--primary)"`) and spacing (`gap-4`, not `style="gap: 1rem"`): one token vocabulary, one place to grep, one IDE autocomplete surface, and dark-mode / responsive / state variants come for free.

Raw `var(--font-…)` is reserved for entry-stylesheet rules (e.g. the global `html { font-family: var(--font-secondary), …; }` in `Base/`) and a small number of utility-class-unfriendly spots like the `font-family` value inside an `@font-face` rule.

## Reader Settings — runtime font swap on blog

A blog-post-scoped panel that lets readers swap font, size, and line-height live. This is the canonical example of why a project plans for the four-layer chain up front — retrofitting the bridge layer is harder than adding it before the panel ships.

The full design — component layout, the size + spacing + font controls, the per-font size scaling, panel toggle pattern, persistence, accessibility — is its own doc: [`reader_Settings.md`](./reader_Settings.md). The summary here is just: **the script writes `--font-primary` as an inline style on `article.prose`; the CSS cascade does the rest, no `<head>` injection needed**.

## Preloading — what to preload, what to skip

`<Font />` belongs in **one place**: the `<head>` of the root layout (typically `src/layouts/Layout.astro`). Every page renders through that layout, so a single `<Font />` declaration there covers the whole site; never repeat it per-page or per-component. If a project has more than one top-level layout, each one needs its own `<Font />` block — but most projects just have the one.

Configuration in `astro.config.mjs` decides which weights/styles are *fetched* (and emitted into `dist/`). Preload in `<Font preload={...} />` is a separate question: which of those should the browser **block the first paint on** by emitting `<link rel="preload">`.

Rule: preload only the LCP-critical variants. Everything else stays available (fetched lazily on first use) but does not bloat the initial preload payload.

```astro
<Font
  cssVariable="--astro_Fnt_Public_Sans"
  preload={[
    { weight: 400, style: "normal" },
    { weight: 700, style: "normal" },
  ]}
/>
```

In Voice's `Layout.astro`, weight `400` is preloaded for body copy and weight `700` for the H1 — those two are guaranteed to render above the fold. Weights `500` and `600` are configured but not preloaded; they fetch on first use after paint.

For a single-weight font, `preload` may be a plain `true`:

```astro
<Font cssVariable="--astro_Fnt_Quran" preload />
```

For a local font with one variant, preload the one variant.

## Weights gotcha — Astro defaults to `["400"]`

If you omit `weights` on a Fontsource entry, the API silently fetches only the 400 weight. Any `font-medium` (500), `font-semibold` (600), or `font-bold` (700) usage in markup then falls back to the system stack — no error, no warning, just unbranded text.

For **variable fonts**, use a range string to get the full axis in one file:

```js
weights: ["100 900"]
```

For **non-variable fonts**, list every weight the template uses:

```js
weights: [400, 600, 700]
```

To find which weights a template uses:

```bash
grep -rno "font-\(thin\|extralight\|light\|normal\|medium\|semibold\|bold\|extrabold\|black\)" src/
```

For local fonts, the equivalent miss is forgetting to add a `variants[]` entry for an italic or weight — but here at least the missing file will be obvious in DevTools (the face just doesn't render).

## Adding a new font — checklist

1. **Is it on Fontsource?** Search `https://fontsource.org/fonts` or check `npm view @fontsource/<family>`. If yes → use `fontProviders.fontsource()`. If no → option 2.
2. **Not on Fontsource?** Confirm the license permits self-hosting, drop the `.woff2` files under `src/Assets/Fonts/<Family>/`, and use `fontProviders.local()` with one `variants[]` entry per weight/style.
3. **`cssVariable`** in the form `--astro_Fnt_<Family_Name>` (PascalCase_Underscored).
4. **Weights:** if the face is a variable font, use `weights: ["100 900"]` for the full axis in one file. If non-variable, list every weight the template uses. Default `["400"]` is rarely enough.
5. **Wire to Tailwind** by adding the var to a `@theme { --font-* }` token in the entry stylesheet.
6. **Preload** only the LCP-critical variants from `Layout.astro` (or wherever `<head>` lives).
7. **Audit:** boot `bun run dev`, open DevTools → Network → Fonts and confirm the right files load. Tail of a `bun run build` should list the emitted faces under `_astro/`.

## Adding a new weight to an existing font — checklist

1. Add the integer to `weights: [...]` (Fontsource) or a new `variants[]` entry (local) in `astro.config.mjs`.
2. Decide: is this weight LCP-critical? If yes, also add `{ weight: <N>, style: "normal" }` to the `<Font preload={...} />` array.
3. Re-run `bun run dev` — Astro re-fetches and re-emits.
4. Update the audit comment in `astro.config.mjs` with the new weight and what uses it.

## Renaming or moving — checklist

Font registration touches several places. After any change to `name` or `cssVariable`:

1. Grep for the old `cssVariable`:
   ```bash
   grep -rn "--astro_Fnt_Old_Name" src/ astro.config.mjs
   ```
2. Update every `@theme` mapping, every direct `var(--astro_Fnt_...)` reference, and any `<Font cssVariable="...">` usage.
3. For a local font, if the file path changes (folder rename, file rename), also update the `src: [...]` array in `variants[]`.
4. Re-run `bun run build`. Local fonts that point at a non-existent path fail loudly at build time.
