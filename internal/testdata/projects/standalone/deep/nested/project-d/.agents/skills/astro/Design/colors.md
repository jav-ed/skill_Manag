# Design — colors

Rules for *which* color token to reach for in markup. Pairs with [Styles/tokens.md](../Styles/tokens.md) which documents the token system mechanics. These rules are project-agnostic — they hold whether the brand accent is emerald, midnight blue, or anything else.

## The cardinal rule

**Never use raw color values in templates or JS.** No `bg-white`, `text-green-500`, `white/20`, hex literals, or inline `oklch(...)` in markup. Always reference a token.

```html
<!-- wrong -->
<div class="bg-white/20 text-green-500 outline-white">
<div style="background: white">

<!-- right -->
<div class="bg-muted text-primary outline-foreground">
<div style="background: var(--muted-foreground)">
```

In JS: `el.style.background = 'var(--muted-foreground)'`, not `'white'`.

Every raw color breaks dark-mode and theme-flip behavior. Tokens pivot per theme; hardcoded values don't.

## The monochromatic-default rule

The site reads monochromatic unless proven otherwise. Don't reach for `primary` or `secondary` because an element is *interactive* — reach for `foreground`, `muted-foreground`, or `muted`.

Color enters the system when it carries **meaning** (active state, deliberate accent, callout), not when it confirms **affordance** (hover, focus, click target — those belong to neutral tokens).

A practical consequence: `--primary` should appear on at most ~10% of any given screen surface. Its scarcity is its signal strength. Spread it and it becomes noise.

## Brand Color Decision Flow

Before using `primary` or `secondary`, answer what job the color is doing.

Use `primary` when the element is structurally important:

- true primary CTA
- explicitly approved selected/filter state
- rare brand identity moment
- primary-action hover/active surface via `primary-dim` or `primary-accent`

Use `secondary` when the element is a supporting accent:

- small premium mark
- featured metadata
- restrained callout
- decorative detail that has been intentionally approved

Keep these neutral:

- nav hovers
- breadcrumbs
- menu rows
- utility controls
- ordinary row links
- card hover states
- generic focus rings

Focus uses `--outline`. A project may choose an outline token that harmonizes with the brand, but components should not reach directly for `primary` just because they are focusable.

## The three-tier text rule

Body-heavy pages use exactly three text tiers. No intermediate opacities like `/85` or `/90`.

| Tier | Class | Used for |
|---|---|---|
| **Primary** | `text-foreground` | Headings, body paragraphs, pull-quotes, service titles, FAQ questions |
| **Secondary** | `text-foreground/70` | Descriptions subordinate to a title, FAQ answers, CTA supporting text |
| **Tertiary** | `text-muted-foreground` | Section labels, footnotes, hints, navigation links |

Three tiers is the entire vocabulary. If a design seems to need a fourth, the structure is wrong — either promote/demote an existing tier or introduce a token, don't invent an opacity step.

## Hover affordances

Hover affordances have their own state rules in [interactions.md](interactions.md). The short version:

- compact neutral controls use `.surface-hover`
- card-level affordances use `foreground/80`
- inline arrows match the hovered text color

Mismatching these makes the page feel busy: a card with a `foreground` arrow and a `foreground/80` outline reads as two competing affordances.

## Token reference (by role)

This table documents what each role-token is *for*. Actual base values live in each project's `Base/theme.css` or equivalent token file. Dependent section/surface values live in the project's derived color file, such as `Base/derived_Color_Vars.css`.

### Surfaces
| Token | Use for |
|---|---|
| `--background` | Page background |
| `--foreground` | Primary text, bright UI chrome |
| `--card` | Card / panel backgrounds |
| `--card-foreground` | Text on cards |

### Brand
| Token | Use for |
|---|---|
| `--primary` | Main brand/action accent — rare, deliberate moments only. Default to foreground/muted for interactive states. |
| `--primary-foreground` | Text on primary |
| `--primary-dim` | Tinted primary surface — hover/active only for elements whose meaning is already primary |
| `--primary-accent` | Hover state on primary elements |
| `--secondary` | Secondary brand/accent role for featured content, callouts, badges, or small premium marks |
| `--secondary-dim` | Tinted secondary surface derived from the secondary role |

### Muted / Neutral
| Token | Use for |
|---|---|
| `--muted` | Chips, tags, disabled backgrounds |
| `--muted-foreground` | Secondary text, dimmed labels, subtle indicators |
| `--neutral` | Neutral surfaces |

### Borders & inputs
| Token | Use for |
|---|---|
| `--border` | Dividers, separator lines — very subtle chrome |
| `--outline` | Focus rings (same luminance as foreground) |

### Semantic states
| Token | Use for |
|---|---|
| `--info` / `--info-foreground` | Informational messages |
| `--success` / `--success-foreground` | Success states |
| `--warning` / `--warning-foreground` | Warning states |
| `--error` / `--error-foreground` | Error states |

## Contrast traps

- **`--border` against `--background` has very low contrast by design** — use it for lines that should nearly disappear. For visible UI indicators, use `--muted-foreground` or `--foreground`.
- **`--primary` is not "the brand color reasserted everywhere."** It's structurally an emphasis tone. If something genuinely needs to scream (critical alert, paid-plan callout), introduce a separate `--emphasis` or `--alert` token. Reaching for `--primary` for that purpose breaks the monochrome character of the site.
- **Chroma at high lightness reads as saturated color, even when other tokens are neutral.** A dark-mode `--primary` at `oklch(0.86 0.06 252)` reads as visibly blue against pure-neutral surroundings. If a project is monochrome, dark-mode tokens above ~0.75 lightness must sit at chroma ≈ 0. See [Styles/tokens.md](../Styles/tokens.md) on why oklch makes this rule enforceable.

## Light vs dark

All color tokens have light and dark theme variants in the project's token files. Designs using the token system adapt automatically; hardcoded colors do not. This is the entire reason the never-raw-colors rule exists.
