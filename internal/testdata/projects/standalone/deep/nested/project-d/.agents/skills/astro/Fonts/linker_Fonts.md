# Fonts

Everything about loading, configuring, rendering, and runtime-swapping fonts in Astro projects in this org. Fonts has its own folder (not under `Areas/`) because it grew past one cohesive file — provider config, the variable chain pattern, runtime swap UI, language-specific rendering, and performance verification each warrant their own doc.

For binary asset *placement* (where `.woff2` files live on disk), see [`../Areas/assets.md`](../Areas/assets.md). This folder is about the system that consumes them.

## Routing

- [Bootstrap](bootstrap.md): the project-setup view of the font system. The slot=category convention (Modern=sans, Elegant=serif, Precise=mono), the three real edit points (`font_Config.js`, `fonts_Base.css`, the rare `Init_Layout.astro` touch), a dependency map showing how five consumers derive from `font_Config.js`, what auto-inherits at runtime vs. what bakes at build (the Mermaid caveat), single-bundled-font and two-or-more-font playbooks, and a six-step project audit checklist. Open this when starting a new repo, picking which fonts to ship, or auditing an inherited project.
- [Setup](setup.md): the everyday doc — Astro Fonts API config in `astro.config.mjs`, the two providers (`fontProviders.fontsource()` and `fontProviders.local()`), why Google Fonts is never used (privacy / GDPR), `--astro_Fnt_<Family>` cssVariable convention, the two-vs-four-layer variable chain (bridge vars + semantic tokens + Tailwind preference), preload strategy from the root Layout's `<head>`, the weights-default-to-`["400"]` gotcha, and add/rename checklists. Start here for any "how do I add / change / wire up a font?" question.
- [Reader Settings](reader_Settings.md): the runtime typography panel for blog posts — three controls (size / spacing / font), the `data-rs-*` button protocol, panel toggle pattern (click-outside + ESC + `data-open` grid-row animation), per-font size scaling, localStorage persistence (`rs-size`, `rs-line`, `rs-font`), the inline-style-on-`article.prose` mechanism (not `<head>` injection), accessibility expectations, and what to wire when porting the pattern. Open when implementing or modifying a blog reader-settings UI, debugging persistence, or planning a similar runtime swap on a non-blog surface.
- [CJK rendering](cjk.md): rules for `zh` / `ja` / `ko` pages and OG images — the negative-tracking ban (Tailwind v4 `--tracking-tight` override + arbitrary-value attribute-selector belt-and-braces + prose heading reset), the weight ceiling for OG image generation (no `bold`/`extrabold` on CJK strings; downshift table), the `b_Cjk` detection regex, and the relationship to non-CJK prose. Open when adding or modifying a CJK locale, working on the OG image generator, or auditing typography on a CJK page.
- [Performance & debugging](performance.md): what `<Font />` actually emits in the page HTML (the `<style>` block with `@font-face` + the `<link rel="preload">` entries), how to verify font loading in DevTools (Network → Fonts, Coverage panel, Performance timeline), the `dist/_astro/` build output (content-hashed filenames, typical sizes), cache behavior across builds and how to force a clean re-emit, and the relative impact ranking of preload / display / subsetting / format choices. Open when fonts are slow, when a face isn't loading, when a build emits unexpected files, or when an LCP audit fingers a font as the bottleneck.

## When to open which

- "I'm starting a new project / inheriting one, where do fonts plug in?" → [Bootstrap](bootstrap.md)
- "Which file gets edited when I change which font is the body?" → [Bootstrap](bootstrap.md), Three edit points + Dependency map
- "Does Mermaid follow Reader Settings?" → [Bootstrap](bootstrap.md), What inherits automatically table
- "How do I add a font?" → [Setup](setup.md), Adding a new font checklist
- "Why doesn't `font-bold` render?" → [Setup](setup.md), Weights gotcha
- "I'm building a reader-facing font picker" → [Reader Settings](reader_Settings.md)
- "A JS-driven chart / canvas / diagram lib doesn't pick up the right font" → [Setup](setup.md), four-layer chain section (look for `getPropertyValue` caveat)
- "We're adding Chinese / Japanese / Korean" → [CJK rendering](cjk.md)
- "Lighthouse is complaining about the LCP and points at a font" → [Performance & debugging](performance.md)
- "The font file isn't in `dist/`" → [Performance & debugging](performance.md), build output section
