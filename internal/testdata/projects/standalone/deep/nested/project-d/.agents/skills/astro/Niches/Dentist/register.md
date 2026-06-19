# Dentist Register

Dentist guidance applies to dental-practice, orthodontic, oral-surgery, and similar healthcare-practice websites. It is not a general premium-web rule. A dentist site needs calm, medical trust, practical clarity, and human warmth.

Use this after the general [Design](../../Design/linker_Design.md), [Styles](../../Styles/linker_Styles.md), and [Fonts](../../Fonts/linker_Fonts.md) rules.

## What Must Read Dentist

A dentist site should feel:

- formal enough for healthcare
- warm enough for anxious patients
- precise enough for medical credibility
- calm enough that the user does not feel sold to
- practical enough that appointment, location, insurance, accessibility, and emergency information are easy to find

It should not read as:

- SaaS dashboard
- design-studio portfolio
- luxury fashion clinic
- spa or wellness retreat
- playful startup
- generic local-business template with dental words inserted

## Color Direction

Dentist sites may use a restrained primary color, but color should support trust rather than dominate the page.

Good primary directions often include:

- botanical green
- clinical blue
- mineral gray-blue
- warm neutral

Good secondary/accent directions can include muted brass or gold, but those should usually stay supporting accents rather than the main system color.

This doc does not prescribe exact values. Store actual colors in project brand docs and `src/Styles/Base/theme.css`. Store derived section and surface roles in `src/Styles/Base/derived_Color_Vars.css`, deriving from existing theme variables.

Avoid:

- electric agency-green unless the dentist brand explicitly owns it
- bright cosmetic-clinic gold as the main system color
- saturated section bands everywhere
- using primary for every icon, hover, link, chip, or service number
- color used to make weak hierarchy look designed

## Typography Direction

Dentist typography is allowed to be warmer than a pure SaaS or technical-provider register. Premium dentist sites can use a restrained serif for large emotional headings and a clean sans for functional UI. That rule is documented in [typography.md](typography.md).

Do not generalize the serif/sans rule to all premium websites. A technical premium brand may be stronger as mostly sans.

## Imagery Direction

Dentist imagery should help patients understand the practice without overpromising treatment outcomes.

Prefer:

- empty practice rooms
- reception, waiting areas, hallways, details, plants, instruments without treatment scenes
- architecture and light
- staff or patient imagery only when the project has verified rights and legal approval

Avoid:

- before/after treatment images unless legal review explicitly approves them
- fake patient or staff photography
- AI-generated real-looking people
- overlit stock-medical imagery
- luxury-spa cliches such as marble, champagne gold, or VIP cues

Project-specific image policy and legal floor belong in project docs.

## Interaction Tone

Interactions should be quiet and predictable:

- appointment CTA is clear but not aggressive
- forms feel calm and trustworthy
- hover states confirm affordance without turning colorful
- motion is minimal and never playful
- accessibility affordances are visible and serious

## Reuse Boundary

Use these rules for dentist and adjacent oral-healthcare sites. Do not copy them into legal, SaaS, restaurant, portfolio, or agency websites without a new design argument.
