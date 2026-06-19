# Design — CTAs

Rules for calls to action on premium Astro surfaces. These are general design rules, not dentist-specific rules. Exact labels, business priorities, and conversion paths belong in project docs.

## CTA Scarcity

A primary CTA is a scarce semantic signal. It tells the user "this is the main next action," not "this element is clickable."

Use primary CTA treatment for:

- the hero's main action
- the header's appointment/contact action when it is a true site-wide priority
- one strongest action inside a modal, form, or conversion panel
- a selected/filter state only when the product meaning justifies brand emphasis

Avoid primary CTA treatment for:

- every "read more" link
- nav hovers
- breadcrumbs
- menu rows
- ordinary cards
- decorative icons, bullets, numbers, chips, or eyebrows

If several CTAs compete on one viewport, decide which action is actually primary. The others should become secondary buttons, editorial links, or neutral row links.

## CTA Treatments

Prefer semantic variants over one-off styling:

- filled primary: true primary action
- foreground-on-background or inverted surface: strong action when the brand accent should stay restrained
- outlined or ghost: secondary action
- editorial arrow link: contextual "continue" action, not a conversion action

Header and sticky chrome should usually be quieter than the hero CTA. They may stay visible, but they should not make every viewport feel like a conversion funnel.

## Primary Color

Primary is not the default hover language. Hover confirms affordance through neutral tokens and interaction primitives. Primary appears when the element has primary semantic weight.

Solid saturated brand-color buttons can be correct for some brands or lower-tier work, but they are not the default answer for premium surfaces. If a primary button looks cheap in dark mode, fix the token value or use an inverted-surface treatment instead of adding more color elsewhere.

## Focus Rings

Focus is accessibility, not branding. Use `--outline` or a foreground-luminance focus ring by default.

Do not use direct primary focus rings except on a component whose whole meaning is primary action and whose contrast has been verified in both themes. If a project wants focus to feel brand-connected, set `--outline` deliberately in the token file and keep component markup on the outline role.
