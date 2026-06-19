# Design

Visual and interaction rules for premium Astro work in this org. This skill area is the *editorial discipline* layer: which token to reach for in markup, how text hierarchy works, how links signal their jobs, when motion is allowed and when it's noise. It does not own the token *system* — that lives in [Styles](../Styles/linker_Styles.md) — it owns the *usage* of those tokens.

The rules here are deliberately project-agnostic premium rules. They survive any brand identity: whether the accent is emerald, midnight blue, or rust, "never raw colors / monochromatic by default / three-tier text / font roles by job / CTA scarcity / derived surface roles / three underline patterns / editorial arrow links and row links / small radius scale / flat at rest" still hold. Project-specific decisions (actual oklch values, brand register, commercial tier discipline, exact font families) live in each project's own `Project_Manag/Docs/Brand/` or `DESIGN.md`.

This folder is not the home for industry-specific rules. Niche guidance captures domain-specific judgment that is reusable only inside that industry. For dental-practice work, use [Dentist Niche](../Niches/Dentist/linker_Dentist.md). For dentist preview tiering such as Partner / Tier 3, use [Dentist Tiers](../Niches/Dentist/Tiers/linker_Tiers.md) and the project Brand docs.

## Styling implementation route

Design decides what a UI state or surface should express. [Styles](../Styles/linker_Styles.md) decides whether that expression belongs in Tailwind utilities or a CSS primitive. The operating rule is: Tailwind by default for element-local styling; CSS only when it gives a real advantage such as reusable primitives, CSS-only behavior, reduced duplication, token/source-of-truth control, motion handling, or cross-component chrome.

If you came here first while shaping a component, use the Design leaves below for the semantics, then check [Styles § Tailwind vs CSS](../Styles/linker_Styles.md#tailwind-vs-css) before creating a new class or moving utilities into CSS.

## The cardinal rule

**Never use raw color values in templates or JS.** No `bg-white`, `text-green-500`, `white/20`, hex literals, or inline `oklch(...)` in markup. Always reference a token.

```html
<!-- wrong -->
<div class="bg-white/20 text-green-500 outline-white">
<!-- right -->
<div class="bg-muted text-primary outline-foreground">
```

Same in JS: `el.style.background = 'var(--muted-foreground)'`, not `'white'`.

Every raw color breaks dark-mode and theme-flip behavior. Tokens pivot per theme; hardcoded values don't. This rule is the precondition for everything else in this folder — without it, the rest of the discipline is undermined by a single hardcoded `#fff`.

## The monochromatic-default rule

The site reads monochromatic unless proven otherwise. Don't reach for `--primary` or `--secondary` because an element is interactive — reach for `--foreground`, `--muted-foreground`, or `--muted`. Color enters the system when it carries **meaning** (active state, deliberate accent, callout), not when it confirms **affordance** (hover, focus, click target — those belong to neutral tokens).

`--primary` should appear on at most ~10% of any given screen surface. Its scarcity is its signal strength.

## The three-tier text rule

Body-heavy pages use exactly three text tiers. No intermediate opacities like `/85` or `/90`.

| Tier | Class | Used for |
|---|---|---|
| **Primary** | `text-foreground` | Headings, body paragraphs, pull-quotes, titles |
| **Secondary** | `text-foreground/70` | Descriptions, FAQ answers, supporting text |
| **Tertiary** | `text-muted-foreground` | Labels, footnotes, hints, nav links |

If a design seems to need a fourth tier, the structure is wrong — promote/demote an existing tier or introduce a token, don't invent an opacity step. See [colors.md](colors.md) for the full table and contrast traps. See [interactions.md](interactions.md) for `.surface-hover`, compact chrome primitives, hover/focus/current/selected/disabled rules, and how those states differ from links.

## The radius scale

Three steps plus pill — `xs` (2px), `sm` (4px), `md` (6px), `full`. Nothing else for page-level work. The larger tokens (`lg`, `xl`, `2xl`, `3xl`) exist for third-party component internals only. See [radius.md](radius.md).

## Link patterns

| Pattern | Purpose | Implementation |
|---|---|---|
| **Inline link** | "you can click this" | `border-b` with `muted-foreground/40` |
| **Active nav** | "you are here" | `text-decoration` with `foreground/40` |
| **Active filter** | "this filter is on" | `text-decoration` with `--primary` |
| **Editorial arrow link** | "continue to the related page" | no underline; `arrow-right` carries affordance |
| **Internal row link** | "this whole row opens a related page" | no underline; row layout plus `arrow-right` carries affordance |

Three underline patterns, no fourth underline. Editorial arrow links and internal row links are separate because the icon is the affordance. The differences are deliberate — a user can tell at a glance whether they're looking at prose clickability, page-orientation, a selected filter, or a premium regular-page cross-link. See [links.md](links.md).

## The motion baseline

- `prefers-reduced-motion` is mandatory — every project ships a global override that zeroes animation and transition durations.
- Surfaces are **flat by default**. Depth comes from tonal layering, not shadows. Hovered cards shift hue, not lift.
- Transitions are short (150–250ms), ease in-out (never bouncy), and animate one property at a time (or rarely two).

See [motion.md](motion.md) for transition discipline, the scroll-driven-animation pattern, and the specific anti-patterns (staggered entry, idle pulsing, decorative animations).

## Where does a new design rule go?

| What you're documenting | Where it goes |
|---|---|
| Which token to reach for in a specific UI context | [colors.md](colors.md) |
| Font-role strategy, display-vs-functional typography, or serif/sans boundaries | [typography.md](typography.md) |
| CTA priority, primary-button scarcity, or focus-ring color | [ctas.md](ctas.md) |
| Section rhythm, surface ladders, cards, borders, or derived section roles | [surfaces.md](surfaces.md) |
| When to use which radius step | [radius.md](radius.md) |
| A hover, focus, current, selected, disabled, or open-state rule | [interactions.md](interactions.md) |
| A link / interaction-affordance pattern | [links.md](links.md) |
| Icon usage, arrow semantics, icon color/sizing/hover behavior | [icons.md](icons.md) |
| Animation, transition, or elevation rule | [motion.md](motion.md) |
| A final browser/code pass before calling a premium surface done | [audit_Checklist.md](audit_Checklist.md) |
| Whether styling belongs in Tailwind utilities or CSS | [Styles § Tailwind vs CSS](../Styles/linker_Styles.md#tailwind-vs-css) |
| The token system mechanics (`@theme inline`, raw vars, name-reserved pattern) | [Styles/tokens.md](../Styles/tokens.md) — not Design |
| Derived color variables built from existing theme vars | [Styles/tokens.md](../Styles/tokens.md) and `Base/derived_Color_Vars.css` — not Design |
| Brand-specific value (the actual oklch for emerald) | Each project's own `Project_Manag/Docs/Brand/` or `DESIGN.md` |
| Dentist-specific design register, color direction, imagery, or typography judgment | [Niches/Dentist](../Niches/Dentist/linker_Dentist.md): niche-specific, not universal |
| Dentist commercial tier routing, including Partner / Tier 3 | [Niches/Dentist/Tiers](../Niches/Dentist/Tiers/linker_Tiers.md) plus project Brand docs |
| New universal rule that does not fit an existing Design leaf | Open a new leaf — but first ask whether it really belongs in one of the existing leaves |

## Deep-dives

- [colors.md](colors.md): the cardinal never-raw-colors rule, the monochromatic-default rule, the three-tier text hierarchy with worked examples, the full role-token reference table (surfaces / brand / muted / borders / semantic), and the three contrast traps (low-contrast `--border` by design, `--primary` is an emphasis tone not a hue-everywhere, chroma-at-high-lightness reads saturated). Use when deciding which color/token to apply, or auditing a page for hierarchy violations.
- [typography.md](typography.md): the general font-role system. Explains why serif/sans pairing is a project/niche strategy, not a universal premium rule; routes dentist-specific display-serif guidance to the Dentist niche; keeps exact font families in project font config and brand docs.
- [ctas.md](ctas.md): CTA scarcity, primary-button semantics, header/sticky CTA restraint, secondary action hierarchy, and focus-ring color discipline. Use when deciding whether a button, form action, or conversion link deserves primary treatment.
- [surfaces.md](surfaces.md): premium section rhythm, surface ladders, derived section roles, card-grid limits, borders, radius reminders, and placeholder-imagery rules. Use when shaping page rhythm or choosing a surface/background role.
- [interactions.md](interactions.md): hover, focus, current, selected, disabled, and open-state semantics. Owns the neutral `.surface-hover` primitive, compact menu/icon chrome primitives, when not to use them, selected/current-state boundaries, disabled-state guidance, and the note that links need a future audit/update pass. Use when designing or deduplicating component interaction states.
- [radius.md](radius.md): the three allowed steps (xs/sm/md) plus `full`, why only three (premium-brand restraint, element-size mapping), why `lg`/`xl`/`2xl`/`3xl` exist as tokens but aren't for page work (third-party component internals), and the rule for adding to the vocabulary (almost never; replace or push the scale outward, don't slot in between). Use when picking a radius, or pushing back on a design that wants `rounded-xl`.
- [links.md](links.md): the three underline patterns (inline `border-b`, active nav `text-decoration`, active filter `text-decoration` with `--primary`) plus the editorial arrow-link and internal row-link patterns for regular-page cross-links, why `border-b` beats `text-decoration` for prose inline links, why the active-filter case is the rare context where `--primary` belongs in chrome, and the breadcrumb exception (color-only hover) with its rationale. Use when adding a link, an active state, or a selected-filter indicator.
- [icons.md](icons.md): Lucide-only visual usage, fixed arrow semantics (`arrow-right` internal, `arrow-up-right` external, `chevron-down` disclosure), no text glyph arrows, icon sizing, currentColor coloring, hover behavior, and premium icon restraint. Use when choosing or styling icons in UI. For import/SVGO/custom-SVG mechanics, use [Areas/icons.md](../Areas/icons.md).
- [motion.md](motion.md): the mandatory global `prefers-reduced-motion` override, the flat-by-default rule for elevation (no shadows on cards/lists/inline; shadows only on truly floating surfaces), transition discipline (150–250ms, ease in-out, animate one property), the scroll-driven-animation pattern with non-animated fallback, and explicit anti-patterns (staggered entry, idle pulsing, decorative animations). Use when adding a transition, designing a hover state, or considering an animation.
- [audit_Checklist.md](audit_Checklist.md): final premium-surface audit for raw colors, primary overuse, CTA priority, link semantics, card/grid overuse, radius drift, surface mistakes, and dark-mode problems.
- [Dentist Niche](../Niches/Dentist/linker_Dentist.md): dentist-specific design judgment that must not be applied to every project. Currently covers register, serif-display plus sans-functional typography, and dentist commercial tier routing.
