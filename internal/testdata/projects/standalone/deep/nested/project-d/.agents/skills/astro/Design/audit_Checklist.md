# Design — audit checklist

Use this before calling a premium Astro surface finished. It works for browser review, screenshot review, or code audit.

## First-Pass Questions

- Does the page still read premium if primary color is almost absent?
- Is there one clear strongest CTA per context?
- Does the page rely on composition, type, spacing, and surface rhythm before color?
- Does dark mode look authored, not mechanically inverted?
- Do the rules still hold at mobile, tablet, and desktop widths?

## Color

Flag raw colors in templates, JS, and component CSS:

- `bg-white`, `text-black`, `text-green-*`, hex literals
- inline `oklch(...)`
- raw `stroke` / `fill` values on icons
- ad hoc opacity tiers such as `text-foreground/85`

Flag overuse of `primary` on:

- nav hovers
- breadcrumbs
- utility controls
- ordinary row links
- service icons
- decorative dots, bullets, and numbers
- section eyebrows
- chips that are not primary actions

Approve primary when it carries semantic weight: true CTA, approved selected/filter state, or a deliberate brand accent with a clear reason.

## CTAs

Check:

- the hero has the clearest action
- header/sticky CTAs do not overpower the hero
- secondary actions look secondary
- "read more" links are editorial links or row links, not primary buttons everywhere

If every section has a primary-colored CTA, the page is solving hierarchy with color instead of structure.

## Links And Nav

Check:

- prose links use the inline border pattern
- active nav uses neutral underline
- active filters use primary only when brand emphasis is meaningful
- editorial links use arrow-led affordance
- internal row links use row structure plus `arrow-right`
- breadcrumbs stay compact and use their documented exception

## Layout And Surfaces

Flag:

- repeated eyebrow + h2 + lead + grid section openers
- uniform card grids as the default grammar
- cards inside cards
- saturated color bands as section breaks
- decorative shadows or hover lift on ordinary cards
- `rounded-lg`, `rounded-xl`, `rounded-2xl`, or arbitrary radius in page work
- fresh component-local `oklch()` values

Prefer:

- tonal section lift
- hairlines
- stable surface roles from the token ladder
- row lists when comparison matters
- asymmetric or editorial layouts only when they clarify hierarchy

## Dark Mode

Check:

- primary does not become a saturated neon accent against neutral dark surfaces
- muted text still passes contrast where it carries content
- secondary surfaces do not create low-contrast prose
- focus rings are visible
- selected states do not depend on primary color unless explicitly approved
- inverted CTAs are readable and not disabled-looking

## Final Decision

If the page feels too plain, try this order before adding color:

1. stronger type hierarchy
2. clearer section rhythm
3. better image/content pairing
4. more precise spacing
5. hairline separators
6. improved CTA priority
7. one deliberate accent moment

Premium quality comes from confident restraint, not from making more things colored.
