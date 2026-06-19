# Design — link treatments

Three underline patterns exist site-wide. Each maps to a distinct purpose. Don't introduce a fourth underline pattern. Pairs with [colors.md](colors.md) for the token rules.

Regular page editorial links and internal row links with icons are separate arrow-led patterns, documented below. They are not prose links and they are not another underline style.

## The three patterns

| Pattern | Token | Resting visibility | Purpose | Implementation |
|---|---|---|---|---|
| **Inline link** | `muted-foreground/40` | Subtle | "you can click this" | `border-b` |
| **Active nav** | `foreground/40` | Visible | "you are here" | `text-decoration` |
| **Active filter** | `primary` (brand accent) | Prominent | "this filter is on" | `text-decoration` |

The three are deliberately different so a user can tell at a glance whether they're looking at a clickable affordance, a page-orientation cue, or a selected filter.

## Inline clickable links

Used for: email addresses, breadcrumb parent links, any text link inside body copy or chrome that needs a subtle "clickable" affordance.

```html
<a class="border-b border-muted-foreground/40 pb-px
          hover:text-foreground hover:border-foreground
          transition-colors duration-200">
  link text
</a>
```

- Resting: `border-muted-foreground/40` — one pixel bottom border, clearly visible but not demanding attention.
- Hover: border and text both step up to full `foreground`.
- Implementation: `border-b`, not `text-decoration`. Reasons:
  - Precise control over thickness and offset across fonts.
  - `text-decoration` thickness varies with `text-decoration-thickness` browser support; `border-b` is consistent.
  - Easier to animate (transition on `border-color`).

## Active / current-page nav signal

Used for: nav links marking the current page.

```css
text-decoration: underline;
text-decoration-color: color-mix(in oklch, var(--foreground) 40%, transparent);
text-decoration-thickness: 2px;
text-underline-offset: 4px;
```

- Uses `--foreground` at 40% — stronger than the inline link's `muted-foreground` base, because this is an orientation signal, not just a clickability affordance.
- 2px thickness reads as a definite indicator, not decoration.
- `text-decoration` (not `border-b`) because the underline length should match the text, not the link's box. Nav text often wraps differently than the inline-link case.

## Active filter / selection signal

Used for: the active category button in an overview/filter UI. Marks which filter is currently selected, not which page you are on.

```html
<button class="underline decoration-primary decoration-2 underline-offset-4">
  Active filter
</button>
```

- Uses `--primary` (brand accent) — the one context where an underline earns the brand color; it is a deliberate selection state, not a hover affordance.
- Same 2px thickness and 4px offset as the active-nav state, but the accent color makes the active filter visually distinct from page-level navigation.

This is one of the few places `--primary` shows up in UI chrome. See [colors.md](colors.md) on the monochromatic-default rule.

## Editorial arrow links

Used for: regular page cross-links such as "Mehr erfahren", teaser links, feature links, related-term clusters, and section-to-detail links where the link is already isolated from body copy.

Do not use for: prose / MDX inline links, legal body links, email addresses, phone numbers, breadcrumbs, active nav, or active filters.

If the project provides shared editorial-link components, prefer those over retyping the full anchor pattern:

```astro
<Internal_Editorial_Link href={href}>
  Mehr erfahren
</Internal_Editorial_Link>
```

Example component path: `src/Components/Common/Links/Internal_Editorial_Link.astro`.

For external editorial links, use:

```astro
<External_Editorial_Link href={href}>
  External resource
</External_Editorial_Link>
```

Example component path: `src/Components/Common/Links/External_Editorial_Link.astro`.

```html
<a class="group inline-flex items-center gap-2 no-underline
          text-foreground/70 font-semibold
          hover:text-foreground focus-visible:text-foreground
          transition-colors duration-200">
  Mehr erfahren
  <Lucid_Arrow_Right
    class="size-4 text-muted-foreground opacity-70
           transition-all duration-300 ease-out
           group-hover:text-foreground group-hover:opacity-100 group-hover:translate-x-1"
    aria-hidden="true"
  />
</a>
```

- Resting: no underline or border. The arrow is the affordance.
- Text rests below full foreground, then steps up to full `foreground` on hover/focus.
- Internal links use `arrow-right`; external links use `arrow-up-right`.
- The arrow may translate slightly on hover. Do not move the text.
- External editorial links validate that the href leaves the current site/context and default to `rel="noopener"`.

### Touch-device handling

Editorial arrow links must not depend on hover for their basic affordance. On desktop, the quiet resting state plus hover lift is correct. On touch devices, hover is unavailable or unreliable, so the resting state should be closer to the desktop hover state:

```html
<a class="arrow-link-touch-text text-foreground/70 hover:text-foreground">
  Mehr erfahren
  <Lucid_Arrow_Right
    class="arrow-link-touch-icon text-muted-foreground opacity-70
           group-hover:text-foreground group-hover:opacity-100"
  />
</a>
```

Keep the movement itself hover/focus-only. The touch rule is about resting clarity, not simulating animation after tap. Do not add mobile-only underlines to editorial arrow links; the arrow remains the affordance. If the project has a shared interaction primitive stylesheet, put the `(hover: none)` behavior there and reuse a class instead of repeating arbitrary media variants in every component.

## Internal row links

Used for: full-row internal links where the entire row is the click target, such as "Weiter lesen" lists, glossary index rows, related services, or numbered service lists.

Do not use for: prose / MDX inline links, email addresses, phone numbers, breadcrumbs, active nav, active filters, or isolated "Mehr erfahren" links.

If the project provides shared row-link components, prefer those over retyping the full row pattern:

```astro
<Internal_Related_Row_Link
  href={href}
  title="Praxistour"
  description="Ein ruhiger Rundgang durch Empfang, Wege und Behandlungsraeume."
/>

<Internal_Indexed_Row_Link
  href={href}
  index="01"
  title="Zahnerhaltung"
  description="Moderne Fuellungstherapie und Wurzelkanalbehandlung..."
/>
```

Example component paths:

- `src/Components/Common/Links/Internal_Related_Row_Link.astro`
- `src/Components/Common/Links/Internal_Indexed_Row_Link.astro`

Rules:

- no underline or bottom border on the text itself
- row separators live in the parent list (`border-t`, `border-b`, or `divide-y`)
- the right arrow is always Lucide `arrow-right` for internal navigation
- compact related rows use title + optional description + arrow
- indexed rows use number + title + description + arrow
- text and arrow lift neutrally on hover/focus; do not use `primary`
- on touch devices, visible arrows may rest at full foreground/opacity so the row does not look inactive without hover

## Why these three and no fourth

A fourth underline pattern collapses the meaning of the existing three. If you're tempted to add one, it usually means one of:

- The existing inline link is being asked to do too much (e.g. "needs to read as more important here"). Solution: change the surrounding type weight or color, not the link decoration.
- You want a hover treatment to look different. Solution: use the inline pattern with a different hover destination (e.g. `hover:text-primary` for a deliberate accent moment).
- You're treating the link as a button. Solution: actually use a button, not a link.

## Exception — breadcrumb links

Breadcrumbs use color-only hover — no underline or border. Resting at `text-muted-foreground` (inherited), steps up to `text-foreground` on hover. The `border-b` inline pattern was intentionally dropped here:

- Breadcrumb chrome is compact and the underline reads as too heavy at `text-xs`.
- Multiple consecutive links separated by `/` get visually noisy with three or four underlines stacked.

This is the only place color-alone signals clickability. Don't extend the exception to other compact link clusters without re-deriving why it's safe there.
