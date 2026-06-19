# Design — typography roles

Typography strategy is a brand and niche decision. This file documents the general role system. It does not prescribe a universal serif/sans pairing and it does not name exact font families.

For font loading, preloading, Astro Fonts API setup, and semantic CSS variables, use [Fonts](../Fonts/linker_Fonts.md). For dentist-specific serif/sans guidance, use [Dentist Typography](../Niches/Dentist/typography.md).

## Core Principle

Choose fonts by job:

- **Functional clarity:** nav, buttons, forms, tables, filters, cards, metadata, utility controls, technical pages, and dense body text.
- **Emotional/editorial voice:** large hero statements, quote-like moments, major section titles, and other sparse display surfaces.

Most functional surfaces should use the project's cleanest, most legible sans unless the project has a deliberate reason to make body text serif. Display surfaces may use a serif, sans, or another display face depending on the brand register.

## Two Fonts Is A Strategy, Not A Default

A two-font system can work when the roles are strict:

```txt
display/emotional moments -> display role
body/functional UI        -> functional role
```

It fails when the decorative font leaks everywhere:

```txt
nav/buttons/forms/cards/footer -> decorative/display font
```

Do not infer "premium means serif." Some premium brands should stay mostly sans because they need to read technical, precise, architectural, or operational. Other premium brands can use a restrained serif because they need to read warm, human, editorial, or practice-led.

## Role Examples

Dentist premium demo:

- large emotional headings can use a restrained serif
- body, nav, buttons, forms, metadata, cards, and dense sections stay sans
- see [Dentist Typography](../Niches/Dentist/typography.md)

Technical provider / agency site:

- main identity can stay sans-only
- technical pages, feature rows, pricing, tables, forms, and nav stay sans
- serif, if used, is a rare accent for a quote-like or emotional moment
- the main hero does not become serif unless the project brand explicitly chooses that softer register

Editorial publication:

- body serif may be correct
- UI chrome can still stay sans
- reader settings and prose components must be checked separately

## Implementation Boundary

Components should reference semantic font tokens, not family names:

- `font-primary`
- `font-secondary`
- `font-modern`
- `font-elegant`
- project-specific semantic tokens when the project defines them

Exact family choices and preload choices live in `src/Data/Common/font_Config.js`, font base CSS, and project brand docs. Do not hardcode family names in component markup.

## Audit Questions

- Does each font have a clear job?
- Does the display font appear only where its emotional/editorial voice is needed?
- Are functional surfaces still legible and calm?
- Does the body font support long reading?
- Does the site still feel like the intended brand, not just more decorative?
- Are all shipped font roles configured and preloaded through the font pipeline when needed?
