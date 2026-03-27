# Styling

## Tailwind CSS v4

No `tailwind.config.js` — everything is CSS-native.

- **Entry stylesheet:** `src/Styles/0_Base/base_Main.css` — imports Tailwind, DaisyUI, fonts, and root variables. All other stylesheets are imported by it or import from it.
- Custom design tokens go in `@theme {}` blocks inside CSS files.
- Custom breakpoints are defined with `@custom-variant` in `base_Main.css`:

| Variant | Range | Device |
|---|---|---|
| `m_ss` | ≤ 375px | iPhone SE |
| `m_sm` | 376px – 416px | Samsung |
| `m_xs` | 417px – 639px | Small mobile |

- Page-level stylesheets live in `src/Styles/2_Pages/<PageName>/` and are imported in the `.astro` page file frontmatter.

## DaisyUI v5

Component library loaded via `@plugin "daisyui"` in `base_Main.css` with all themes enabled. Conventions and component usage to be documented.
