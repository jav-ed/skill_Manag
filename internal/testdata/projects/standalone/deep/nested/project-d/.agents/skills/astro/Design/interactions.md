# Design — interactions

Rules for hover, focus, current, selected, disabled, and open states. This doc owns interaction-state semantics. Pair it with [colors.md](colors.md) for token hierarchy, [links.md](links.md) for link-specific underline patterns, and [motion.md](motion.md) for transition timing and animation limits.

## Default neutral surface hover

Compact controls that gain a filled hover/focus surface use the shared `.surface-hover` primitive. It lives in `src/Styles/Base/interaction_Primitives.css` and resolves to a foreground-relative tint:

```css
background: color-mix(in oklab, var(--color-foreground) 10%, transparent);
```

Use it for:

- menu rows
- icon buttons
- mobile-nav rows
- utility buttons
- compact controls where the user needs neutral hit-area confirmation

Do not use it for:

- prose links
- active nav underlines
- primary CTA buttons
- as the selected/filter marker itself
- card-level editorial affordances

The rule exists because `--muted` can sit too close to `--card` or `--background` in dark mode. Foreground-relative tinting gives the hover surface enough contrast without turning affordance into brand color.

## Reusable compact chrome

`src/Styles/Base/interaction_Primitives.css` also owns small reusable chrome classes that are shared across menus and utility controls:

- `.menu-root`: native `<details>` wrapper reset and positioning anchor
- `.menu-panel`: dropdown panel shell; use `--right`, `--compact`, and `--wide` modifiers for alignment and width
- `.menu-row`: shared row geometry for dropdown and mobile menu rows; pair it with `--compact`, `--mobile`, `--split`, or `--button` modifiers as needed
- `.menu-chev`: shared disclosure chevron rotation, keyed from `.menu-root[open]`
- `.utility-menu-trigger`: compact icon/text disclosure trigger, normally paired with `.surface-hover`
- `.icon-button`: square icon-only button, normally paired with `.surface-hover`

These primitives are examples of the [Styles Tailwind-vs-CSS rule](../Styles/linker_Styles.md#tailwind-vs-css): they are shared multi-property chrome, so one named CSS primitive is clearer than repeating a long utility cluster across menus. Element-local interaction styling should still stay in Tailwind utilities by default.

Use these when multiple components share the same physical interaction shape. Keep component-specific content styling local: language code chips, theme checkmarks, rich menu blurbs, overlay animation, and active-nav underline semantics stay with the component or link primitives.

CSS primitives must use the radius tokens from [radius.md](radius.md), not raw `border-radius` values. In practice: menu panels and row surfaces use `var(--radius-md)`, chips and compact badges use `var(--radius-sm)`, and tiny focus-outline rounding uses `var(--radius-xs)`.

## Focus states

Focus is an accessibility state, not decoration. Neutral focus rings use foreground-luminance through `--outline`:

```css
outline: 2px solid color-mix(in oklab, var(--color-outline) 60%, transparent);
```

`.surface-hover` includes this ring by default. Override only the offset through `--surface-focus-offset` when a compact control needs more room.

Do not use `primary` for ordinary focus rings. Brand-colored focus makes every keyboard stop look like a selected state.

## Current and selected states

Current/selected states are not hover states. They communicate orientation or selection, so the selected marker should be separate from the hover marker.

For compact menus, hover fill means "available alternative." Current/selected rows keep their selected marker stable and suppress the filled hover.

If the current row is only status text, do not attach `.surface-hover` at all:

```astro
<li class="nav-menu-lang-current" aria-current="true">
```

If the selected row must remain a focusable control, keep the shared row class for the group and suppress the selected row locally with Tailwind state variants instead of adding component CSS:

```astro
class="surface-hover aria-[checked=true]:pointer-events-none aria-[checked=true]:hover:bg-transparent aria-[checked=true]:focus-visible:bg-transparent"
```

The current value then gets its own marker:

- language selector current row: inverted language code chip and stronger text
- theme selector current row: checkmark and stronger text
- active nav link: active-nav underline from [links.md](links.md)

Unavailable rows also suppress hover. Do not put `.surface-hover` on disabled, missing, or unavailable rows.

For compact filled selected surfaces, use the shared selected primitive:

- `.surface-selectable`: state-driven selected row/control, keyed by `aria-current="page"`, `aria-selected="true"`, `aria-checked="true"`, or `aria-pressed="true"`

It uses foreground-relative fill, not `--muted`, so the selected surface stays legible in light and dark themes.

Only use filled selected surfaces when the fill is the clearest selected indicator. If the component already has a strong selected marker, keep that marker and suppress `.surface-hover` fill for the selected row:

- language selector current row: inverted language code chip and stronger text, not persistent row fill
- theme selector current row: checkmark and stronger text, not persistent row fill
- active nav link: active-nav underline unless the surface is explicitly a compact selectable control

Default neutral options:

- current page in nav: use the active-nav underline pattern from [links.md](links.md)
- selected compact row/control: use `.surface-selectable` only when a filled state is the intended indicator
- selected filter: use the active-filter pattern from [links.md](links.md) when brand emphasis is explicitly meaningful

Do not use `primary` for generic selected rows, current menu items, language rows, or theme rows. In premium surfaces, selected UI should usually stay neutral.

## Disabled and unavailable states

Disabled/unavailable states should reduce clarity without introducing a new color vocabulary.

Use:

- `text-muted-foreground`
- `cursor-not-allowed` for controls
- `aria-disabled="true"` when the element is still visible in a list
- optional strikethrough only when the unavailable state means "exists, but not here"

Do not add disabled-specific grays in component CSS. If a recurring disabled surface needs a new role, add it to the token system and document the role.

## Open disclosure states

Open dropdown/disclosure triggers may use the same neutral family as hover/focus, but open state should be slightly clearer than hover only when the menu remains visible.

Use foreground text plus a subtle surface. For example, a compact header can style `.menu-root[open] > .utility-menu-trigger` for disclosure controls.

Do not animate disclosure state beyond a short chevron rotation unless the motion clarifies structure.

## Links are separate

Link styling is intentionally not collapsed into this file. Text links carry meaning through underline/border patterns, not filled hover surfaces. Use [links.md](links.md) for inline links, active nav, active filters, and breadcrumb exceptions.

Follow-up: links need a real code audit. After auditing actual link usage across components and prose, update [links.md](links.md) with any missing reusable classes or migration notes instead of making ad hoc link exceptions here.
