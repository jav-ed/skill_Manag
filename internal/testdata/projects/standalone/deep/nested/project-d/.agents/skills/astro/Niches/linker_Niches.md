# Niches

Niche guidance captures design judgment that is reusable for a specific industry, but not universal across Astro projects. Use this folder after the general [Design](../Design/linker_Design.md), [Styles](../Styles/linker_Styles.md), and [Fonts](../Fonts/linker_Fonts.md) rules are understood.

The niche layer must stay conceptual. It may describe why a domain usually benefits from a certain visual strategy, but it must not hard-code a client palette, exact font family, or component recipe. Actual base values belong in the project brand docs, `DESIGN.md`, `src/Styles/Base/theme.css`, and `src/Data/Common/font_Config.js`. Dependent section/surface colors belong in `src/Styles/Base/derived_Color_Vars.css` and must derive from existing theme variables.

If a niche rule needs copyable code, put that artifact under a `Templates/` folder and link to it from the niche doc. Do not bury reusable code blocks in prose.

## Available Niches

- [Dentist](Dentist/linker_Dentist.md): dentist-practice register, typography guidance, and dentist commercial-tier routing, with strict boundaries against applying those choices to every industry.

## Relationship To Other Skill Areas

- Use [Design](../Design/linker_Design.md) for general premium visual rules: token discipline, CTA scarcity, surface rhythm, link semantics, radius, motion, interactions, and color usage.
- Use [Styles](../Styles/linker_Styles.md) for implementation mechanics: Tailwind vs CSS, tokens, layers, breakpoints.
- Use [Fonts](../Fonts/linker_Fonts.md) for font loading, preloads, variable chains, and performance.
- Use project docs for brand decisions: actual colors, actual font families, photography direction, and commercial tier choices.
