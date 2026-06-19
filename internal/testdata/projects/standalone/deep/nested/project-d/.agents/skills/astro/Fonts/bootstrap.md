# Bootstrap

The project-setup view of the font system. Open this when starting a new repo, when picking which fonts to ship, when auditing an existing setup, or when you need to know which files a font change will ripple into.

For the *how* of registering individual fonts (Astro Fonts API mechanics, preload tuning, weight gotchas, local vs Fontsource provider), see [`setup.md`](./setup.md). For the runtime panel that lets readers swap typography live, see [`reader_Settings.md`](./reader_Settings.md). This file is about the project-level wiring that ties everything together.

## The slot convention

The Reader Settings panel has three slots, each permanently bound to a font category. The i18n labels are the contract, not an implementation detail.

| Slot | Category | CSS var | Tailwind utility |
|---|---|---|---|
| Modern | sans-serif | `--font-modern` | `font-modern` |
| Elegant | serif | `--font-elegant` | `font-elegant` |
| Precise | monospace | `--font-precise` | `font-precise` |

Never route a font of the wrong category into a slot. A serif behind the "Modern" button makes the panel lie to the reader, and once that drift is in place a future agent will copy it forward.

The slot routing lives in `src/Styles/Base/fonts_Base.css`. The script (`src/Scripts/Browser_Client/Blog/reader.js`) references those slot vars by stable name and never needs editing per project. The `FONTS` lookup table inside the script (`sans → --font-modern`, `serif → --font-elegant`, `mono → --font-precise`) is part of the convention, not a project knob.

## The three edit points

Setting up fonts for a new project touches three files, in this order. Everything else derives.

### 1. `src/Data/Common/font_Config.js` (the single source of truth)

Register one entry per font you ship. Pick the role:

- `primary`: the body font. Becomes `--astro_Fnt_Primary` in CSS and the family that `<html class="font-primary">` resolves to. Every project has exactly one. Mermaid SVGs and OG image cards both inherit this role directly.
- `serif`: a second branded font in the serif category, if you want a real face behind the Elegant slot instead of the system serif fallback.
- `mono`: same idea for the Precise slot.
- `secondary`: present for legacy / template reasons. Most projects leave it `null`.

Set `preload: true` on roles that paint above the fold. The body font almost always qualifies.

### 2. `src/Styles/Base/fonts_Base.css` (the one decision file)

Two chains live here. The `--font-primary` chain feeds the body font. The slot chain feeds Reader Settings.

If the body font is **sans-serif**, the default routing already does the right thing. The Modern slot falls through to `--font-primary`, which is your branded sans. Done.

If the body font is **serif**, the default routing puts the body serif behind the Modern slot, which violates the slot convention. Rewire the slot chain so the body font feeds the Elegant slot and the Modern slot falls through to a system sans:

```css
--font-modern:  var(--font-sans);                              /* system sans */
--font-elegant: var(--astro_Fnt_Primary, var(--font-serif));   /* body font */
--font-precise: var(--astro_Fnt_Mono, var(--font-mono));       /* system mono */
```

If the body font is **monospace** (rare), swap the same logic for Precise.

### 3. `src/Layouts/Base_Layouts/Init_Layout.astro` (read-only most of the time)

The layout iterates `configured_Font_Roles` and emits a `<Font cssVariable={...} preload={...} />` per role automatically. New fonts surface here without any edit.

The one line worth knowing about is `<html class="font-primary">`. The body inherits its font from whatever `--font-primary` resolves to, so this class stays `font-primary` regardless of whether the body is sans, serif, or mono. The CSS chain in `fonts_Base.css` decides what `--font-primary` *means*; the class itself never changes.

## Dependency map

`font_Config.js` is the source. Five consumers derive from it:

```
src/Data/Common/font_Config.js
  ├─→ astro.config.mjs                  fonts: array (provider, weights, cssVariable)
  ├─→ Init_Layout.astro                 <Font preload={...} /> per role, in <head>
  ├─→ fonts_Base.css                    --font-primary chain + slot chain
  ├─→ OG manifest builder (worker_Mn.js) reads og_Font_Roles + primary_Font;
  │                                     resolves files from .astro/fonts/
  └─→ Mermaid post_Process.js           bakes var(--astro_Fnt_Primary, ...) into static SVG
```

The first three are runtime-CSS consumers. The OG manifest builder is a build-time step (Astro virtual route) that writes resolved font-file paths into `Cache/Og_Gen/og_Manifest.json`; an external Bun renderer consumes the manifest afterwards. The Mermaid post-processor patches generated SVG files at build time so the rendered diagrams inherit the role-primary font at runtime.

**`og_Style` and `og_Subset` are the OG gate.** A font role enters OG generation only when both fields are set on the role object. A role without them is registered with Astro Fonts API and emitted to the page, but skipped by the manifest builder. Use that to keep runtime-only faces (script-specific fonts, icon fonts, dev-only switcher candidates) out of OG cards.

## What inherits automatically, what does not

| Consumer | Picks up new fonts from `font_Config.js`? | Responds to Reader Settings at runtime? |
|---|---|---|
| OG manifest builder (`worker_Mn.js`) | Yes via `og_Font_Roles` (the `og_Style` + `og_Subset` fields gate inclusion) and `primary_Font` for the default face. Resolves files from `.astro/fonts/`. The external Bun renderer consumes the manifest afterwards. | N/A, build-time only |
| E_Chart (`E_Chart.astro`) | Yes (reads computed `font-family` from `.prose`) | Yes, click listeners on `[data-rs-font]` / `[data-rs-size]` trigger a re-render |
| Reader Settings panel (`Panel.astro` + `reader.js`) | Yes (via the slot vars in `fonts_Base.css`) | It is the source |
| Mermaid diagrams (`mermaid_Post_Process.js`) | Yes (`primary_Font.css_Variable` baked into the SVG as `var(--astro_Fnt_Primary, ...)`) | No, references the role var directly, not the slot |
| Other static SVG diagrams (`initDiagrams()` in `reader.js`) | N/A (no font wiring) | Width restored once at load, font inherits from the surrounding article |

The Mermaid caveat is the load-bearing one. Mermaid diagrams stay on the body font even when the reader switches Reader Settings to a different slot. If a project specifically needs Mermaid to re-font with the reader's choice, change the post-process replacement target in `mermaid_Post_Process.js` from `--astro_Fnt_Primary` to `--font-primary`, so the SVG resolves against the article-level inline style instead of the global role var.

## Single-bundled-font playbook

The slot chain falls through to Tailwind's system stacks (`--font-sans`, `--font-serif`, `--font-mono`) for any slot that has no branded face behind it. So a project ships one branded font and still presents three visually distinct Reader Settings options.

| Body font category | Register in `font_Config.js` as | `fonts_Base.css` slot wiring | Panel shows |
|---|---|---|---|
| Sans-serif | role `primary` | default routing | Modern = your font, Elegant = system serif, Precise = system mono |
| Serif | role `primary` | route Elegant to `--astro_Fnt_Primary`, Modern to system sans | Modern = system sans, Elegant = your font, Precise = system mono |
| Monospace | role `primary` | route Precise to `--astro_Fnt_Primary`, Modern to system sans | Modern = system sans, Elegant = system serif, Precise = your font |

The body font always lives in role `primary`. The category decides which slot to wire it into.

## Two-or-more-font playbook

A project that ships a branded sans body plus a branded serif for editorial accents:

- Register the sans in role `primary`.
- Register the serif in role `serif`.
- Leave `fonts_Base.css` on default routing.

The default chain resolves Modern to `--font-primary` (your branded sans), Elegant to `--astro_Fnt_Serif` (your branded serif), Precise to the system mono. Both fonts get preloaded according to their `preload:` field. Reader Settings gets two real branded options plus a system mono fallback.

For sans body plus branded mono: register sans in `primary`, mono in `mono`. Default routing covers it.

For serif body plus branded sans accent: register serif in `primary`, sans in `secondary` (or just leave the Modern slot to system sans). Rewire `fonts_Base.css` per the single-font serif row above so Elegant feeds the body and Modern feeds either the secondary role or the system sans stack.

## Project audit checklist

When inheriting an existing project, walk these in order:

1. **`font_Config.js`**: which roles are populated, which family names sit behind them, and does each entry have a sensible `preload:` value?
2. **`fonts_Base.css`**: does the slot routing match the body font's category? Specifically, does the body font sit behind the slot whose i18n label matches its visual category?
3. **`Init_Layout.astro`**: `<html class="font-primary">` should still be on `font-primary`. Anything else means an exotic body-font choice that needs justification.
4. **Open the blog Reader Settings panel**: each button should show a visually distinct font that matches its label. Modern reads as sans, Elegant reads as serif, Precise reads as monospaced.
5. **Trigger a Mermaid diagram in a post**: the rendered SVG should use the body font. If it falls back to system sans, the post-process pipeline drifted from `font_Config.js` and the `primary_Font` import needs checking.
6. **Trigger an E_Chart in a post**, change a Reader Settings option, confirm the chart re-renders with the new font and size.

When all six pass, the font system is wired correctly.
