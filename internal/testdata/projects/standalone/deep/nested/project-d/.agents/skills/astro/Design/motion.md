# Design — motion and elevation

Restraint rules for transitions, animations, and depth. The site is flat-by-default; motion confirms interaction rather than announcing the design.

## Reduced motion is mandatory

Every project ships a global `prefers-reduced-motion` override that zeroes out animation and transition durations. It lives in the final global Base stylesheet or equivalent override file so it wins against any per-component transitions:

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

Don't skip this. Reduced-motion users will see your hover transitions otherwise. The global override means every new interactive element automatically respects the preference — no per-component handling needed.

For animations that *must* keep some movement (e.g. a critical-path scroll-driven progress indicator), add an explicit per-component opt-out inside the `@media (prefers-reduced-motion: reduce)` block, not by setting a non-zero duration globally.

## Flat-by-default

Surfaces are flat at rest. Depth comes from tonal layering (background → card → muted surfaces), not from shadows.

- **No shadows on cards, list items, or inline elements.** Use background-tint or border for separation.
- **Shadows — if used at all — belong only on genuinely floating surfaces:** modals, dialogs, popovers, dropdowns. The shadow says "this is above the page," and most things are not above the page.
- **Hover state communicated by tonal shift, not lift.** A hovered card shifts hue (`bg-muted` or `bg-accent`), not brand color by default; it does not get `box-shadow` or `transform: translateY(-2px)`.

This keeps the design precise. A page where every card lifts on hover looks like a SaaS template; a page where hovered cards quietly tint reads as deliberate.

## Transition discipline

When you do animate, follow three rules.

1. **Short and immediate.** `150ms` is the default; `200ms` for color transitions; `250ms` for layout changes. Never longer for interactive feedback — users notice anything past 300ms as a delay.
2. **Ease in-out or linear, not bouncy.** No `cubic-bezier(...overshoot...)`, no spring physics on UI chrome. Bouncy easings call attention to themselves; the design's job is to recede.
3. **Animate one property, occasionally two.** Hover: `color`, `background-color`, optionally `border-color`. Don't animate transform + opacity + color + box-shadow together — that's how UI starts to feel restless.

```css
/* good */
a:hover {
  color: var(--foreground);
  transition: color 0.15s ease;
}

/* good */
.card:hover {
  background-color: var(--muted);
  border-color: var(--foreground);
  transition: background-color 0.2s ease, border-color 0.15s ease;
}

/* avoid */
.card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0,0,0,0.2);
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}
```

## Scroll-driven animation

Modern CSS scroll-driven animations (`animation-timeline: scroll()`) are fine for progress indicators (reading progress, header frost) when they:

- Have a non-animated fallback for browsers without support.
- Are wrapped in a `@supports (animation-timeline: scroll())` block.
- Respect `prefers-reduced-motion`.

Pattern from a transparent header:

```css
/* fallback — always frosted */
.header {
  background-color: color-mix(in oklch, var(--background) 92%, transparent);
  backdrop-filter: blur(12px);
}

/* scroll-driven enhancement */
@supports (animation-timeline: scroll()) {
  .header { animation: hdr-scroll linear both; animation-timeline: scroll(root block); }
}

/* reduced motion: skip the animation, stay frosted */
@media (prefers-reduced-motion: reduce) {
  .header { animation: none; }
}
```

The fallback is the load-bearing piece. If the enhancement disappears, the page still works.

## Don't decorate with animation

Two specific don'ts that come up often:

- **No staggered entry animations on page load.** A list of items fading in one-by-one looks impressive once and annoying every subsequent visit. Worse: it delays time-to-content.
- **No floating / pulsing / breathing animations on idle elements.** Anything that moves without user input is a visual interruption. Reserve motion for confirming the user's actions.

The narrow exception: a menu open/close animation (a deliberate user action). Even then, keep it under 200ms and use a tonal/opacity transition, not a translate-and-fade-and-scale.
