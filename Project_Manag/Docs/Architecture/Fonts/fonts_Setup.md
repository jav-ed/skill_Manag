# Fonts Setup

## Fonts in use

| Family | Provider | Type | Role |
|---|---|---|---|
| Platypi | `fontProviders.fontsource()` | variable (wght 300–800) | Default body font |
| Maple Mono | `fontProviders.fontsource()` | variable | Monospace / code blocks |
| Al Qalam Quran | `fontProviders.local()` | static (wght 400) | Quranic text only |

Montserrat was removed — confirmed never used in `src/`.

## File locations

Only Al Qalam requires a local file (not on Fontsource):

```
src/Assets/Fonts/Quran/Al_Qalam_Quran_Regular.woff2
```

Platypi and Maple Mono are fetched by Astro from Fontsource at build time — no local files.

## Astro config

Fonts are declared at top-level in `astro.config.mjs` (stable in Astro 6, no longer under `experimental`):

```js
import { defineConfig, fontProviders } from "astro/config";

fonts: [
  { provider: fontProviders.fontsource(), name: "Platypi",    cssVariable: "--astro_Fnt_Platypi" },
  { provider: fontProviders.fontsource(), name: "Maple Mono", cssVariable: "--astro_Fnt_Maple_Mono" },
  {
    provider: fontProviders.local(),
    name: "Fnt_Al_Qalam_Quran_Regular",
    cssVariable: "--astro_Fnt_Al_Qalam_Quran_Regular",
    options: {
      variants: [{ weight: "400", style: "normal", src: ["./src/Assets/Fonts/Quran/Al_Qalam_Quran_Regular.woff2"], display: "swap" }]
    }
  },
]
```

## Variable chain

Three layers — each layer has a distinct responsibility:

```
--astro_Fnt_Platypi          Injected by Astro <Font /> component (runtime)
       ↓
--fnt_Platypi                Bridge var — set inline in 0_Init_Layout.astro
       ↓                     Decouples CSS/Tailwind from Astro-specific names.
--font-platypi               Tailwind @theme var — used in utility classes + CSS
```

Bridge is declared in `src/Layouts/0_Base_Layouts/0_Init_Layout.astro`:

```astro
<style set:html={`
  :root {
    --fnt_Platypi:    var(--astro_Fnt_Platypi);
    --fnt_Al_Qalam:   var(--astro_Fnt_Al_Qalam_Quran_Regular);
    --fnt_Maple_Mono: var(--astro_Fnt_Maple_Mono);
  }
`} />
```

Tailwind `@theme` is in `src/Styles/0_Base/fonts_Base.css`:

```css
@theme {
  --font-platypi:  var(--fnt_Platypi);
  --font-al_qalam: var(--fnt_Al_Qalam);
  --font-mono:     var(--fnt_Maple_Mono);
}
```

## Preloading

Handled in `src/Layouts/0_Base_Layouts/0_Init_Layout.astro`:

- Platypi normal → preloaded on Latin langs only (`en`, `de`, `es`, `fr`). On `zh`/`ur`, Platypi has no glyph coverage for the body text — system fonts handle CJK/Arabic script — so preloading is wasteful. The `@font-face` is still registered on all pages so Platypi loads lazily for any Latin UI elements (nav, dates, etc.).
- Al Qalam → preloaded only when `b_Quran_Font=true` (landing page only)
- Maple Mono → never preloaded (loaded on demand for code blocks)
