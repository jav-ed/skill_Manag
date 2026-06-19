# Dentist Tier 3

Use this only when a dentist project explicitly labels a page or preview as Tier 3, Partner, or premium dentist-preview work.

This file is a routing layer. It does not own the universal premium rules and it does not own exact project values.

## Apply In This Order

1. Start with [Design](../../../Design/linker_Design.md): token discipline, CTA scarcity, surfaces, link patterns, interactions, icons, radius, motion, and audit checklist.
2. Apply [Dentist Register](../register.md): calm medical trust, patient clarity, restrained color, legal-aware imagery, quiet interaction tone.
3. Apply [Dentist Typography](../typography.md): serif only for large emotional headings when the project direction supports it; sans for functional UI. This is dentist-specific, not a general Tier 3 rule.
4. Read the project Brand docs for exact commercial-tier decisions.

For this Partner repo, those project docs are:

- `Project_Manag/Docs/Brand/design_Tiers.md`
- `Project_Manag/Docs/Brand/tier_3_Partner.md`
- `Project_Manag/Docs/Reference_Site/tier_Demos.md`

## Tier 3 Design Posture

Tier 3 should feel more custom, restrained, and invested than lower dentist preview tiers on first paint.

Use:

- monochrome-first layout
- primary color only for true CTAs or explicitly approved accent states
- authored section rhythm instead of repeated template openers
- tonal lift, whitespace, hairlines, and type before color
- calm, finished dark mode when dark mode exists
- editorial or row-link patterns instead of primary-colored "read more" buttons everywhere

Avoid:

- treating every hover as a brand-color moment
- saturated color bands as section breaks
- decorative primary icons, bullets, numbers, chips, or eyebrows
- uniform card grids as the whole page grammar
- luxury-spa cues that weaken medical trust
- two-font typography applied mechanically to every heading

## Typography Reminder

Two fonts are not a general Tier 3 rule. For premium dentist sites, a restrained serif can work for large emotional headings, while body copy, nav, buttons, forms, cards, metadata, and utility controls stay sans.

If a compact section or repeated card grid gets busy, keep those headings sans unless there is a strong page-level editorial reason.

## Values And Code

Do not hardcode colors or font families here. Exact values live in the project brand docs, `src/Styles/Base/theme.css`, `src/Styles/Base/derived_Color_Vars.css`, and `src/Data/Common/font_Config.js`.
