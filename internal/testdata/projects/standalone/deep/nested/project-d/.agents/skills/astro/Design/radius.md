# Design — border radius

Small, fixed radius vocabulary. The whole site reaches for three values plus pill — nothing else. Pairs with [Styles/tokens.md](../Styles/tokens.md) which declares the full token scale.

## The rule

**Never hardcode `border-radius` values.** No `border-radius: 0.75rem`, `rounded-xl`, or arbitrary `rounded-[12px]` in templates or CSS. Use only the allowed steps.

```html
<!-- wrong -->
<div class="rounded-lg">
<div class="rounded-2xl">
<div style="border-radius: 0.75rem">

<!-- right -->
<div class="rounded-xs">
<div class="rounded-md">
<div class="rounded-full">  <!-- pills only -->
```

## Allowed values

| Utility | Token | Computed | Use for |
|---|---|---|---|
| `rounded-xs` | `--radius-xs` | 2px | Smallest elements — hover outlines on large selectors, inline code, subtle rounding on inline elements |
| `rounded-sm` | `--radius-sm` | 4px | Buttons, inputs, chips, code blocks, small interactive elements |
| `rounded-md` | `--radius-md` | 6px | Cards, panels, modals, section containers |
| `rounded-full` | — | pill | Avatars, pill badges, tags |

Three steps + pill is the entire vocabulary. The scale (2 → 4 → 6 px) maps cleanly to element size hierarchy: small → medium → large.

## Why only three

Premium brands use fewer radius values with clear separation. A tight range keeps everything sharp and precise — authoritative territory, not playful. More variation reads as either:

- **Inconsistent** — every component picked its own radius, no visual rhythm.
- **Decorative** — radius being used to add character instead of expressing structure.

Three steps forces the choice: am I working on something inline, interactive, or surface-level? That mental model carries the design without per-component decisions.

## The remaining tokens exist, but not for page work

The project token file declares `--radius-lg`, `--radius-xl`, `--radius-2xl`, `--radius-3xl` too. These are for third-party component internals (Starwind, shadcn-style libraries) that depend on having the full scale. Page-level work — buttons, cards, callouts, custom components — uses only `xs`/`sm`/`md`/`full`.

If you find yourself wanting `rounded-lg` for a card, the card needs `rounded-md` instead. If you find yourself wanting `rounded-2xl` for a feature highlight, the feature needs a different design choice (more whitespace, more typographic weight, a token-driven background tint), not a bigger radius.

## Adding to the vocabulary

Don't, in most cases. If a new project genuinely needs a fourth step (very rare), the rule is: it must replace one of the existing three or push the scale outward, not slot in between. The 2/4/6 spacing is part of what makes the three steps feel distinct.

When in doubt, prove the design needs more than the existing scale by mocking it both ways. The constrained version almost always reads better.
