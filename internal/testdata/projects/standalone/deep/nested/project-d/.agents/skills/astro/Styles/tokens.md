# Styles — tokens

Design tokens are split by responsibility. Base semantic values live in `Base/theme.css`; derived color roles live in `Base/derived_Color_Vars.css`; font bridge tokens live in the font base file for the project. These files re-export their public roles into Tailwind's namespace through `@theme inline` so utilities like `bg-primary`, `bg-section-raised`, and `text-foreground` resolve at runtime.

This file documents the token system itself (mechanics, name-reserved pattern, full catalogue). For *which token to reach for in markup* (text hierarchy, hover affordances, link patterns), see [Design/colors.md](../Design/colors.md).

## Token Layers

**1. Base semantic vars** — actual theme values under `:root` (light defaults) and `[data-theme="dark"]` (dark overrides). In Partner this is `Base/theme.css`.

**2. Derived vars** — values computed from base semantic vars. In Partner this is `Base/derived_Color_Vars.css`, which derives reusable section colors from roles such as `--background`, `--primary-dim`, `--muted`, `--secondary-dim`, and `--border`. Derived files must not introduce standalone palette values.

**3. `@theme inline { ... }` exports** — maps each public var to a `--color-*`, `--font-*`, or `--radius-*` Tailwind namespace var. Because `inline` defers resolution to runtime, `[data-theme="light"]` overrides take effect without a Tailwind rebuild.

```css
:root {
  --primary:    oklch(0.22 0.06 252);
  --background: oklch(1 0 0);
}
[data-theme="dark"] {
  --primary:    oklch(0.86 0 0);
  --background: oklch(0.17 0 0);
}
@theme inline {
  --color-primary:    var(--primary);
  --color-background: var(--background);
}
```

After this, `bg-primary` / `bg-section-raised` / `text-foreground` / etc. work as standard Tailwind utilities and flip with the `[data-theme]` attribute.

## Pre-paint theme attribute (no flash of wrong theme)

The `<html data-theme>` attribute is set by a pre-paint inline script in `Layout.astro`, *before* any CSS loads:

```html
<script is:inline>
  const t = localStorage.getItem("theme") ||
            (matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");
  document.documentElement.setAttribute("data-theme", t);
</script>
```

This runs synchronously before paint, so the very first frame already has the right tokens active. Don't skip this — without it, every page flashes light theme for a few ms before the JS swaps it.

## Name-reserved tokens — declare everything, fill in over time

The recommended pattern: declare the **full token catalogue** in `@theme inline`, even for surfaces the project doesn't style yet. Tokens with a matching raw `var(--name)` resolve to a real color. Tokens *without* a raw backing are name-reserved — registered so the Tailwind utility exists, but `var(--popover)` resolves to an invalid value the browser drops.

That's deliberate. Three benefits:

- **No renames later.** Writing `bg-popover` in markup today drops silently. The day the project gets a popover surface, you add `--popover: oklch(...)` to `:root` + `[data-theme="dark"]` and the utility lights up site-wide. No codebase-wide search-and-replace.
- **Consistent vocabulary across projects.** Every project in this org uses the same token names. Cross-project ports keep `bg-primary` / `text-muted-foreground` working without rename mapping.
- **Documents intent.** Declaring `--color-popover` says "this project will eventually have popovers." Other contributors don't need to invent a new name when that day comes.

Partner uses this pattern explicitly (see the comments at the top of `src/Styles/Base/theme.css` and `src/Styles/Base/derived_Color_Vars.css`).

## The full token catalogue

Declare all of these in `@theme inline`. Provide raw backing vars for the ones the project actually uses. Names below are the canonical org-wide set — don't invent variants.

### Surfaces
| Token | Use for |
|---|---|
| `--background` | Page background |
| `--foreground` | Primary text, bright UI chrome |
| `--card` | Card / panel backgrounds |
| `--card-foreground` | Text on cards |
| `--popover` | Popover / dropdown surfaces |
| `--popover-foreground` | Text on popovers |

### Brand
| Token | Use for |
|---|---|
| `--primary` | Project's brand accent — sparingly. Most interactive states should use `foreground`/`muted`, not `primary`. |
| `--primary-foreground` | Text on primary surfaces |
| `--primary-dim` | Tinted primary hover/active background for elements whose meaning is already primary |
| `--primary-accent` | Hover state on primary elements |
| `--secondary` | Secondary accent (callouts, badges, featured content) |
| `--secondary-foreground` | Text on secondary surfaces |
| `--secondary-dim` | Tinted secondary hover/active background |
| `--secondary-accent` | Hover state on secondary elements |
| `--accent` | Component-internal hover/active state |
| `--accent-foreground` | Text on accent |

### Muted / Neutral
| Token | Use for |
|---|---|
| `--muted` | Chips, tags, disabled backgrounds |
| `--muted-foreground` | Secondary text, dimmed labels, subtle indicators |
| `--neutral` | Neutral surface |
| `--neutral-content` | Text on neutral |

### Borders / inputs
| Token | Use for |
|---|---|
| `--border` | Dividers, separator lines (intentionally low contrast) |
| `--input` | Form input borders |
| `--outline` | Focus rings — same luminance as foreground |

### Semantic states
| Token | Use for |
|---|---|
| `--info` / `--info-foreground` | Informational messages |
| `--success` / `--success-foreground` | Success states |
| `--warning` / `--warning-foreground` | Warning states |
| `--error` / `--error-foreground` | Error states (renamed from shadcn's `--destructive`) |

### Derived section colors
| Token | Use for |
|---|---|
| `--section-base` | Full-width page chapter that should match the main background |
| `--section-raised` | Full-width page chapter with restrained tonal lift |
| `--section-soft` | Soft section background tied to the muted token |
| `--section-warm` | Warm section background derived from the secondary-dim role |
| `--section-border` | Section dividers derived from the border role |

These belong in `Base/derived_Color_Vars.css` because they are not independent palette decisions. They should be recomputed from existing theme variables, not set as fresh `oklch()` swatches.

### Radius
Derived from a single `--radius` base var:

```css
--radius: 0.5rem;            /* base, 8px */
--radius-xs:  calc(var(--radius) - 0.375rem);  /* 2px  — inline elements */
--radius-sm:  calc(var(--radius) - 0.25rem);   /* 4px  — buttons, chips */
--radius-md:  calc(var(--radius) - 0.125rem);  /* 6px  — cards, panels */
--radius-lg:  var(--radius);                   /* 8px  — component internals only */
--radius-xl:  calc(var(--radius) + 0.25rem);   /* 12px — component internals only */
--radius-2xl: calc(var(--radius) + 0.5rem);    /* 16px — component internals only */
--radius-3xl: calc(var(--radius) + 1rem);      /* 24px — component internals only */
```

Page-level work uses `xs`/`sm`/`md` plus `full` for pills. The larger steps exist for third-party component internals (Starwind etc.), not the design vocabulary. See [Design/radius.md](../Design/radius.md).

### Fonts

```css
--font-sans: var(--astro_Fnt_<Family>), -apple-system, BlinkMacSystemFont,
             "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
```

The `--astro_Fnt_<Family>` cssVariable convention comes from Astro Fonts API. See [Fonts](../Fonts/linker_Fonts.md) for the full font pipeline.

If the project has a serif/heading font and a body font, add semantic tokens too:

```css
--font-primary:   var(--astro_Fnt_<Heading_Family>);   /* display titles + headings */
--font-secondary: var(--astro_Fnt_<Body_Family>);      /* body text + prose */
```

Then update those two lines — never the specific names — to change the site's fonts globally.

## Why `oklch()` everywhere

oklch lets you reason about lightness ↔ chroma ↔ hue independently, which the design discipline depends on. Two specific things become possible:

- **Adjust hover by lightness alone.** `oklch(L+0.08 same-C same-H)` produces a perceptually consistent lift regardless of the source hue. Hex/HSL adjustments distort one axis when you mean to move another.
- **Apply the chroma-is-a-function-of-lightness rule** (low-L tokens can carry chroma and still read neutral; high-L tokens must be ~0 chroma to stay monochrome). Trivial to enforce in oklch, fragile in hex.

Don't mix hex literals into the token file. If you must paste a hex, convert it to oklch first.

## Never use raw colors in templates

The rule: no `bg-white/20`, `text-green-500`, hex literals, or inline `oklch(...)` in markup or JS. Always reference a token.

```html
<!-- wrong -->
<div class="bg-white/20 text-green-500 outline-white">
<div style="background: white">

<!-- right -->
<div class="bg-muted text-primary outline-foreground">
<div style="background: var(--muted-foreground)">
```

This applies in JS too — set `el.style.background = 'var(--muted-foreground)'`, not `'white'`.

The discipline matters because every raw color breaks dark-mode + theme-flip behavior. Hardcoded values can't pivot per theme; tokens do.

For the full text-hierarchy / hover / link rules that govern *which* token to reach for, see [Design/colors.md](../Design/colors.md) and [Design/links.md](../Design/links.md).
