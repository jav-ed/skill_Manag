# CJK Rendering

Typography rules for pages and OG images rendered in `zh`, `ja`, or `ko`. These are **rendering-layer** rules — for text formatting (pangu spacing, quote characters, list punctuation), that's a content concern documented separately per project.

The two rules: **no negative letter-spacing** (any glyph compression is unreadable on CJK), and **no bold/extrabold weights in OG image rendering** (heavy strokes blur at small sizes). Site CSS handles the first; OG generators handle the second.

## Detection — the `b_Cjk` boolean

CJK locales share these rendering rules. One regex at the top of any template / stylesheet emitter that needs to differentiate:

```js
const b_Cjk = /^(zh|ja|ko)/i.test(bcp_Lang);
```

`bcp_Lang` is the BCP-47 code (`zh-CN`, `ja-JP`, `ko`, etc.). Anchor at the start because `xx-ZH` (a hypothetical region tag) should not match. The flag is the input for every CJK-specific branch below.

## Rule 1 — never compress glyphs with negative tracking

CJK glyphs are monospaced by design. Compressing them with `letter-spacing: -0.02em` (or any negative value) collapses adjacent strokes into unreadable blobs. **Positive** tracking (widening) is fine — only negative is forbidden.

Tailwind v4's tracking utilities resolve through CSS variables, so the cleanest neutralisation overrides the variables in the CJK locale scope:

```css
/* Tailwind v4 emits letter-spacing: var(--tracking-tight) / var(--tracking-tighter).
   Reducing both vars to 0em in CJK scope makes every existing `tracking-tight` /
   `tracking-tighter` usage in the codebase automatically CJK-safe — no per-class
   whitelist needed.
*/
:lang(zh), :lang(ja), :lang(ko) {
  --tracking-tight:   0em;
  --tracking-tighter: 0em;
}
```

**Belt-and-braces for arbitrary values.** Tailwind's bracket syntax (`tracking-[-0.02em]`) bypasses the variable system — those classes need a direct override:

```css
:lang(zh) [class*="tracking-[-"],
:lang(ja) [class*="tracking-[-"],
:lang(ko) [class*="tracking-[-"] {
  letter-spacing: 0em !important;
}
```

The `!important` is unavoidable here — Tailwind's arbitrary-value class lives in `@layer utilities`, and beating a layered utility requires either another layered rule with higher cascade or an unlayered `!important`. Unlayered `!important` is the smaller hammer.

**Prose heading reset.** If the project's prose stylesheet hard-codes `letter-spacing: -0.01em` (or similar) on headings — common to match design specs for Latin display type — reset those in CJK scope:

```css
:lang(zh) article.prose h1,
:lang(zh) article.prose h2,
:lang(zh) article.prose h3,
:lang(zh) article.prose h5,
:lang(zh) article.prose h6,
:lang(zh) article.prose h1.display-title {
  letter-spacing: 0em;
}
/* h4 carries positive 0.02em — safe, leave alone */
```

Repeat for `:lang(ja)` and `:lang(ko)` (or wrap in a multi-selector list as above).

## Rule 2 — drop heavy weights in OG image rendering

This rule applies to **OG image generation** (the canvas/PNG-emit pipeline), not to live page CSS. Live page CSS renders body text at 400 and headings at 500–600, which is already CJK-safe on screen.

The reason it matters in OG: OG image generators often use heavier weights (700–800) for short headline-style strings to read well at thumbnail size on social cards. CJK strokes are denser than Latin glyphs; the same heavy weight that gives a Latin headline impact turns a CJK string into an ink blob.

Weight downshift table for any OG template that renders CJK:

| Latin default | CJK override |
|---|---|
| `font-extrabold` (800) | not allowed — drop to 500 or below |
| `font-bold` (700) | not allowed — drop to 500 or below |
| `font-semibold` (600) | `font-medium` (500) |
| `font-medium` (500) | `font-normal` (400) |
| `font-normal` (400) | no change |

Implementation pattern in a Satori / canvas template:

```jsx
<h1 className={b_Cjk ? "font-medium" : "font-bold"}>{title}</h1>
```

Or, for templates that interpolate Tailwind classes as strings, swap the heavy class out conditionally:

```js
const title_Class = b_Cjk
  ? title_Class_Base.replace(/font-(bold|extrabold)/, "font-medium")
  : title_Class_Base;
```

## Font fallback on CJK pages

The Fonts API typically only registers Latin-script families. CJK glyphs need either:

1. **A bundled CJK face** registered via `fontProviders.fontsource()` (e.g. `@fontsource/noto-sans-sc` for Simplified Chinese). Adds significant build size — only worth it if CJK is a primary locale.
2. **System CJK fallback** via the `font-family` cascade (`var(--font-secondary), "PingFang SC", "Hiragino Sans", "Noto Sans CJK SC", sans-serif`). Zero build cost, but the visitor's OS controls the rendering.

For projects where CJK is a secondary locale (e.g. one of many supported languages, not the primary audience), system fallback is usually the right choice — Latin readers get the designed face, CJK readers get their OS's high-quality system font. Don't bundle a CJK family unless the design depends on a specific one.

## OG image fallback

The OG manifest builder (`src/Scripts/OG_Images/Og_Img_Gen/Manifest/worker_Mn.js`) writes resolved font-file paths into the manifest; an external Bun renderer reads those files directly when compositing the card. That renderer does not have the live page's CSS, so if no CJK face is in the manifest a Chinese / Japanese / Korean string renders as `□` boxes.

The Partner pattern for adding a CJK face to OG cards has two steps.

**Step 1: register a CJK role in `font_Config.js`** with both `og_Style` and `og_Subset` populated, so the role enters `og_Font_Roles` and the manifest builder resolves its files from `.astro/fonts/`:

```js
zh_Primary: {
  role_Key:           "zh_primary",
  family_Key:         "zh_primary",
  family_Name:        "Noto Sans SC",
  astro_Css_Variable: "astro_Fnt_Zh_Primary",
  css_Variable:       "--astro_Fnt_Zh_Primary",
  weights:            ["400", "500"],
  preload:            false,
  og_Style:           "normal",
  og_Subset:          "chinese-simplified",
}
```

The role must also be registered via Astro Fonts API for the cached `.woff2` to appear in `.astro/fonts/`. The `configured_Font_Roles` derivation in `font_Config.js` handles that automatically once the role is added to the registry.

**Step 2: route the lang→font map in `worker_Mn.js`** so CJK BCP tags point at the new family. The current `og_Lang_Font_Map` builder assigns `primary_Font.family_Name` to every lang; for a project that ships CJK, branch on the BCP tag:

```js
const og_Lang_Font_Map = Object.fromEntries(
  Object.values(bcp_From_Url).map((bcp_Lang) => {
    const b_Cjk = /^(zh|ja|ko)/i.test(bcp_Lang);
    return [
      bcp_Lang,
      b_Cjk
        ? { primary: "Noto Sans SC", secondary: null, accent: null }
        : { primary: primary_Font.family_Name, secondary: null, accent: null },
    ];
  }),
);
```

The external renderer dispatches per-task on `bcp_Lang` from the manifest and picks the right family for each card.

## Checklist when adding a new CJK locale

1. Add the locale to the project's language registry (BCP code, URL key, etc.).
2. Audit existing CSS for hard-coded negative `letter-spacing` (grep `letter-spacing.*-` and `tracking-\[-`). Add CJK-scoped resets next to each occurrence in the relevant stylesheet.
3. Add the `--tracking-tight: 0em` + `--tracking-tighter: 0em` override under `:lang(<code>)` in `fonts.css` (or equivalent base stylesheet).
4. Add the belt-and-braces `[class*="tracking-[-"]` attribute-selector rule.
5. Add `:lang(<code>)` heading resets in the prose stylesheet for any heading with negative tracking.
6. Decide on CJK font strategy — bundled Fontsource face, or system fallback — and update the `font-family` cascade in the entry stylesheet.
7. Update OG image templates to branch on `b_Cjk` and downshift heavy weights per the table above.
8. Register a CJK role in `font_Config.js` with `og_Style` and `og_Subset` populated, and route the new family into `og_Lang_Font_Map` in `worker_Mn.js` for every CJK BCP tag. See the OG image fallback section above for the two-step pattern.
9. Smoke-test: render a sample article in the new locale, render an OG card, screenshot both at thumbnail and full size.
