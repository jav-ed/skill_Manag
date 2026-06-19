# Design - icons and arrows

Visual icon rules for Astro projects. This file owns icon *usage* in UI: which icon to choose, how arrows communicate link type, how icons are sized, colored, and animated. The technical import/SVGO/custom-SVG pipeline lives in [Areas/icons.md](../Areas/icons.md).

## The rule

Use Lucide SVG icons for standard UI. Do not use text glyph arrows or ad hoc inline SVGs for ordinary icons.

```astro
---
import Lucid_Arrow_Right from "@lucid/arrow-right.svg";
import Lucid_Arrow_Up_Right from "@lucid/arrow-up-right.svg";
import Lucid_Chevron_Down from "@lucid/chevron-down.svg";
---
```

Text glyphs such as `→`, `↗`, and `↓` are not acceptable substitutes. They change weight, baseline, metrics, and rendering across fonts. The SVG keeps the visual language consistent.

## Arrow conventions

Arrows communicate link type. The mapping is fixed across projects:

| Meaning | Lucide file | Use |
|---|---|---|
| Internal navigation | `arrow-right.svg` | Stays on the site: service cards, "Mehr erfahren", blog cards |
| External link | `arrow-up-right.svg` | Leaves the site or context: social links, third-party references, `mailto:` when treated as external |
| Expand / collapse | `chevron-down.svg` | Opens content in place: FAQ, dropdown, disclosure |

Do not pick arrows by aesthetics. Pick them by meaning.

## Internal-link arrow pattern

For inline internal links, the arrow belongs to the text. It should match the link color behavior, not act like a separate accent.

```astro
<a class="group inline-flex items-center gap-2 text-muted-foreground hover:text-foreground transition-colors">
  Mehr erfahren
  <Lucid_Arrow_Right
    class="size-4 opacity-0 -translate-x-1 transition-all duration-150 group-hover:translate-x-0 group-hover:opacity-100"
    aria-hidden="true"
  />
</a>
```

Premium note: do not color the arrow with `text-primary`. If the link is not the main CTA, it is neutral.

## External-link arrow pattern

Use `arrow-up-right` for links that leave the current site or context. Keep the icon small and aligned with the text.

```astro
<a class="group inline-flex items-center gap-1.5 border-b border-muted-foreground/40 pb-px hover:text-foreground hover:border-foreground transition-colors">
  Quelle
  <Lucid_Arrow_Up_Right class="size-3.5 opacity-70 group-hover:opacity-100" aria-hidden="true" />
</a>
```

## Disclosure arrow pattern

Use `chevron-down` for in-place expansion. Rotate it on open; do not swap to a text glyph.

```astro
<Lucid_Chevron_Down class="size-4 text-muted-foreground transition-transform group-open:rotate-180" aria-hidden="true" />
```

## Sizing

Use `size-*` utilities. Prefer the standard steps:

| Pixel value | Class |
|---|---|
| 14px | `size-3.5` |
| 16px | `size-4` |
| 20px | `size-5` |
| 24px | `size-6` |
| 28px | `size-7` |
| 32px | `size-8` |

Arbitrary sizes are allowed only when the design genuinely needs them. Do not use arbitrary sizing just to nudge an icon that should be aligned through layout.

## Color

Lucide uses `currentColor`. Color icons with text utilities:

```astro
<Lucid_Arrow_Right class="size-4 text-muted-foreground" />
```

Default choices:

- decorative or secondary icon: `text-muted-foreground`
- icon inside foreground text link: inherit current text color
- icon inside CTA button: inherit button text color
- selected neutral state: foreground or background via inversion

Avoid:

- raw `stroke` / `fill` colors
- `text-primary` for decorative icons
- separate icon color when the icon is part of a text link
- icon-only color as the only hover affordance

## Premium icon discipline

On premium surfaces, icons are chrome, not decoration.

- Menu icons rest muted and lift to foreground on hover.
- Service icons should not be primary by default.
- Bullets, spec marks, numbers, monograms, and decorative arrows should not use primary.
- If every card has a colored icon, the section reads lower-tier.
- If an icon is only there to make a bland component more interesting, solve the component first with spacing, type, hierarchy, or surface rhythm.

## Technical routing

Open [Areas/icons.md](../Areas/icons.md) when you need:

- alias setup for `@lucid/*`
- import naming conventions
- custom SVG folder rules
- `src/Icons/` vs `public/`
- SVGO configuration
- build verification checklist

Open this file when you need to decide what icon belongs in UI and how it should behave visually.
