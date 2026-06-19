# Dentist Typography

Premium dentist sites can benefit from a controlled serif + sans pairing. The goal is not "luxury serif everywhere"; the goal is to add human warmth and trust to large emotional messages while keeping every functional surface clear and medical.

This rule is dentist-specific. It fits calm, premium, healthcare-practice sites where the brand should feel personal, serious, and reassuring. It is not a default rule for all Astro projects or all premium websites.

## Recommended Split

Use:

| Role | Font Category | Reason |
|---|---|---|
| Large emotional headings | restrained serif | Adds warmth, trust, and a boutique-practice feeling. |
| Body, nav, buttons, forms, labels, cards, metadata | clean sans | Keeps the site legible, functional, and medically credible. |

In practice:

- `h1` can use the serif.
- Major `h2` section titles can use the serif when they carry emotional or editorial weight.
- Small headings inside cards, lists, forms, menus, and dense UI usually stay sans.
- Eyebrows, nav, buttons, form labels, helper text, and metadata stay sans.

## Good Use

- Hero headline: serif.
- Major section title with a calm emotional promise: serif.
- Body paragraph explaining treatment or process: sans.
- Service card title, CTA, nav item, form label, phone link: sans.

## Why This Fits Dentists

Only-sans can make a calm premium dentist page feel slightly too close to a SaaS or developer demo. A restrained serif on large emotional headings brings back boutique-practice warmth and trust. The sans still carries clarity, forms, navigation, and medical practicality.

This is different from a technical provider or agency site. A technical premium brand may need to stay mostly sans because precision, structure, and engineering reliability are the main signal.

## Avoid

- Serif in nav, buttons, forms, badges, chips, cards, footer links, or utility controls.
- Serif on every heading by default when the page has many compact sections.
- Delicate fashion or wedding-style serif choices that make the practice feel less medical.
- Old-fashioned bookish serif choices that make the practice feel dated.
- Treating serif as a decoration for otherwise weak hierarchy.
- Applying this dentist rule to Zetun Web, SaaS, technical, legal, or operations-heavy sites without a fresh brand argument.

## Choosing Families

This doc does not prescribe exact font families. Choose project fonts in the project brand docs and `src/Data/Common/font_Config.js`.

Useful selection criteria:

- The serif should be restrained, readable, and serious at display sizes.
- The sans should carry body copy and UI without calling attention to itself.
- The pair should not fight the dental imagery or make the site feel like a spa, fashion clinic, or SaaS dashboard.

For how to configure, preload, and debug the fonts, use [Fonts](../../Fonts/linker_Fonts.md). For which Tailwind font token to apply in markup, use the project's existing semantic font tokens rather than hard-coded family names.
