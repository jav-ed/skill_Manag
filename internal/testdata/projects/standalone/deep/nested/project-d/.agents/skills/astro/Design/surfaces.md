# Design — surfaces

Rules for premium section rhythm, surface selection, borders, and tonal depth. This file explains which surface role to use; [Styles/tokens.md](../Styles/tokens.md) owns the token mechanics and `Base/derived_Color_Vars.css`.

## Surface Ladder

Premium pages use a small tonal ladder instead of one-off colors. The values can change per project, but the roles should stay stable:

| Role | Token examples | Use for |
|---|---|---|
| Page | `background`, `section-base` | Main page background and unlifted full-width chapters |
| Lifted section | `section-raised`, `muted` | Quiet chapter separation without making a card |
| Soft section | `section-soft` | Low-emphasis full-width blocks |
| Warm section | `section-warm` | Rare warm chapter shift derived from an existing accent role |
| Panel/card | `card`, `muted` | Framed content that truly needs containment |
| Hover/active surface | `accent`, `muted` | Neutral component hover and selected surfaces |
| Neutral block | `neutral` | Small controls, disabled states, compact selected states |
| Separator | `border`, `input`, `section-border` | Hairlines, dividers, input borders |

Do not introduce ad hoc `oklch()` values in components. If the ladder does not support a recurring surface, update the token system deliberately.

## Derived Color Vars

Dependent section colors belong in `src/Styles/Base/derived_Color_Vars.css` or the equivalent project derived-token file. They must derive from existing semantic variables such as `--background`, `--muted`, `--secondary-dim`, and `--border`.

Do not store independent palette values in a derived file. Base brand decisions live in `theme.css`; derived files express relationships between existing roles.

## Section Rhythm

Avoid repeating the same opener six times:

```txt
eyebrow
h2
lead paragraph
3-up grid
```

Used once, that pattern is fine. Repeated everywhere, it reads like a template. Alternate with:

- image-led editorial pairs
- split sections where text and media have unequal weight
- narrow prose chapters
- hairline-separated lists
- featured-plus-supporting compositions
- compact row indexes when comparison matters

Do not use saturated section bands as the default way to create rhythm. Prefer tonal lift, whitespace, typography, and hairlines.

## Cards And Grids

Uniform card grids are useful when users compare peers: services, team members, posts, or options. They should not become the whole page grammar.

When a page starts to feel card-heavy, try:

- a featured item plus supporting list
- row links with separators
- grouped prose and image chapters
- one strong editorial panel instead of several equal cards

Do not put cards inside cards. Page sections are full-width bands or unframed layouts; cards are for individual repeated items, modals, or genuinely framed tools.

## Borders

Use borders as quiet structure:

- section dividers
- row separators
- input boundaries
- card boundaries only when surface contrast is insufficient

Avoid:

- decorative side stripes
- heavy boxes around every block
- primary-colored hover borders on ordinary cards
- borders used to compensate for weak spacing

`--border` is intentionally subtle. Use `--input`, `--section-border`, `--muted-foreground`, or `--foreground` only when a visible indicator is genuinely needed.

## Radius

Use the shared radius discipline from [radius.md](radius.md): `xs`, `sm`, `md`, plus `full` for pills. Larger radius tokens exist for third-party internals, not page-level premium design.

## Placeholder Imagery

Letter monograms and tinted avatars read like missing content. Prefer:

- real approved imagery
- controlled placeholder blocks with clear alt text
- typography-led layouts that do not pretend an image exists

Do not use decorative primary monograms to fill content gaps.
