# Styles

`src/Styles/` holds the project's CSS — every stylesheet that ships with the site, organised so the load order is obvious and the cascade is predictable. This skill area covers *how* the CSS is structured (folders, files, imports, layers, tokens, breakpoints). The companion [Design](../Design/linker_Design.md) area covers *which* token or pattern to reach for when actually styling something.

The architecture is built on Tailwind CSS v4, configured CSS-natively (no `tailwind.config.js`). Tokens are CSS custom properties surfaced to Tailwind through `@theme inline`, so utility classes like `bg-primary` and `text-foreground` resolve to themable vars at runtime — the same utility flips with the `[data-theme]` attribute, with no rebuild.

## Tailwind vs CSS

Tailwind is the default for element-local styling. Reach for utilities when the styling belongs to the element in front of you: layout, spacing, text treatment, responsive behavior, simple state variants, and one-off composition. This keeps the component readable at the call site and keeps authors inside one token vocabulary.

Regular CSS must earn its place. Use it when it gives a real system advantage:

- reusable multi-property primitives used across components
- CSS-only selectors or browser behavior (`details[open]`, pseudo-elements, ancestor state, scroll/reveal effects)
- lower duplication or smaller markup when a long utility cluster would be repeated many times
- token or source-of-truth primitives that should be changed in one place
- animation, reduced-motion handling, or cross-component chrome

CSS is not a dumping ground for arbitrary local styling. When CSS is added, prefer a named primitive with a clear design meaning, document where to use it in [Design](../Design/linker_Design.md), and keep component-specific content styling close to the component.

## The two-folder split

The whole system rests on one decision: every stylesheet is either chrome-that-loads-everywhere or chrome-that-only-some-pages-need. That split mirrors how the bundle should behave.

```
src/Styles/
├── Base/                       # loaded on every page via Layout.astro
└── Pages/<PageName>/           # loaded only by the layouts that opt in
```

- **`Base/`** holds shared chrome: tokens, element-level defaults, reusable interaction primitives, reusable components, menus, dark-mode overrides. `Layout.astro` imports `Base/base_Main.css` once; every page inherits it.
- **`Pages/<PageName>/`** holds page-scoped CSS. The relevant page layout imports its files directly (`import "../Styles/Pages/Posts/prose.css"`). Other pages never pay for it.

The split is not theoretical — it shows up in real bundle size when a long-form-only stylesheet (prose typography, blog TOC chrome) stops loading on the homepage. See [folder_Structure.md](folder_Structure.md) for the full tree, naming rules, entry chain, and page-scoping mechanics.

## The token system

The token system has three responsibilities:

1. **Base semantic values** live in `Base/theme.css`: actual theme values for `--background`, `--foreground`, `--primary`, `--secondary`, `--border`, radius, and similar core roles.
2. **Derived color roles** live in `Base/derived_Color_Vars.css`: section and surface roles built only from the base theme variables. Do not add standalone palette values there.
3. **`@theme inline { ... }` exports** surface raw vars and derived vars into Tailwind's namespace (`--color-primary`, `--color-section-raised`, `--radius-md`, `--font-sans`) so utilities resolve at runtime.

The canonical pattern is to **declare every token your project might need in `@theme inline`, even if some don't have raw backing yet**. Tokens without raw backing are *name-reserved* — `bg-popover` drops silently today, but adding the backing var later lights it up site-wide. No renames, no codebase-wide search. See [tokens.md](tokens.md) for the mechanics, the full token catalogue (surfaces, brand, muted, semantic states, derived section colors, radius, fonts), and why oklch is the default color space.

## Cascade layers and breakpoints

Three explicit buckets:

| Layer | Holds | File |
|---|---|---|
| `@layer base` | Element-level defaults (`html`, `body`, headings, links) | `Base/root_Vars.css` or equivalent base file |
| `@layer components` | Reusable patterns (`.shell`, interaction primitives, callouts, menus, prose) | `Base/interaction_Primitives.css`, header CSS, `Pages/<Page>/<feature>.css` |
| _unlayered_ | Overrides that must beat utility classes | A final base override file if the project has one |

Custom breakpoints are declared in `Base/base_Main.css` via `@custom-variant` and used as Tailwind variants (`m_ss:hidden`, `m_xs:text-sm`). See [layers_And_Breakpoints.md](layers_And_Breakpoints.md) for layer semantics, breakpoint conventions, and the checklists for adding, renaming, or moving a stylesheet.

## Naming and conventions

- Folders: `PascalCase` (`Base/`, `Pages/`, `Posts/`).
- Files: `snake_Case.css` matching the project-wide improved_Camel_Snake convention from the [coding skill](../../coding/SKILL.md) (`base_Main.css`, `derived_Color_Vars.css`, `interaction_Primitives.css`).
- No numeric prefixes on folders or files — code folders are identified by name, not position. See [Areas/file_Structure.md](../Areas/file_Structure.md) on the wider rule.
- Per-page folder is named after the surface concern (`Posts/`, `Landing/`, `About/`), not the layout filename.

## Where does a new stylesheet go?

| What you're adding | Where it goes |
|---|---|
| New element default (e.g. styling `<details>`) | `Base/root_Vars.css` or equivalent base file under `@layer base` |
| New reusable component class (`.callout`, `.tag`) | Existing shared component CSS under `Base/`, or page CSS if not global |
| New reusable interaction primitive (`.surface-hover`, `.menu-panel`, `.icon-button`) | `Base/interaction_Primitives.css` under `@layer components`; document when to use it in [Design/](../Design/linker_Design.md) |
| New nav / menu / dropdown chrome | Header CSS or a dedicated Base chrome file |
| New override that must beat utility classes | Final Base override file, only after tokenized markup is not practical |
| New base color, font, or radius token | `Base/theme.css` (see [tokens.md](tokens.md)) |
| New derived section/surface color role | `Base/derived_Color_Vars.css`, deriving only from existing theme vars |
| New custom breakpoint variant | `Base/base_Main.css` (see [layers_And_Breakpoints.md](layers_And_Breakpoints.md)) |
| New CSS used by only one page or layout | `Pages/<PageName>/<feature>.css`, imported by that layout |
| New design rule about *which token to use when* | Not a stylesheet — belongs in [Design/](../Design/linker_Design.md) |

## Deep-dives

- [folder_Structure.md](folder_Structure.md): two-folder split with the full file tree, snake_Case + PascalCase naming, the entry-chain mechanics (`base_Main.css` → `@import` siblings → `Layout.astro`), the page-scoping pattern with worked examples from Partner. Use when adding, moving, or renaming a stylesheet, or when bootstrapping a new repo's Styles layer.
- [tokens.md](tokens.md): the `@theme inline` + raw vars + derived color vars + `[data-theme="dark"]` system, name-reserved-tokens pattern (declare everything you might need, fill in raw vars over time), the full canonical token catalogue (surfaces, brand, muted, borders, semantic states, derived section colors, radius, fonts), pre-paint theme-attribute script to avoid flash-of-wrong-theme, why oklch is mandatory. Use when adding a token, changing a color, wiring a new font into Tailwind, or porting tokens cross-project.
- [layers_And_Breakpoints.md](layers_And_Breakpoints.md): `@layer base` / `@layer components` / unlayered discipline (when each is correct, when unlayered with `!important` is a band-aid for token-bypassing markup), `@custom-variant` breakpoint conventions, and the three operational checklists — adding a stylesheet, renaming/moving a stylesheet, lifting CSS from `Base/` to `Pages/<PageName>/`. Use when you're about to touch the cascade or run a stylesheet rename across the codebase.
