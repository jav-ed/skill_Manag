# Styles — folder structure

Astro projects in this org organise CSS into a two-folder `src/Styles/` tree. `Base/` is shared chrome loaded on every page via `Layout.astro`. `Pages/<PageName>/` is page-scoped CSS that each layout pulls in directly, so off-page bundles do not pay for it. There is no `tailwind.config.js` — Tailwind v4 is configured CSS-natively, see [tokens.md](tokens.md).

Reference implementations: `src/Styles/` (current convention) and `01_Jav_Web/src/Styles/` (pattern source — still uses numeric prefixes `0_Base/` / `2_Pages/`, dropped here per the "no numerics on code folders" rule in [Areas/file_Structure.md](../Areas/file_Structure.md)).

## Tree

```
src/Styles/
├── Base/                       # loaded on every page via Layout.astro
│   ├── base_Main.css           # entry — imports tailwindcss + all siblings
│   ├── theme.css               # base semantic tokens + @theme inline exports
│   ├── derived_Color_Vars.css  # derived section/surface color roles
│   ├── fonts_Base.css          # font bridge vars + Tailwind font tokens
│   ├── root_Vars.css           # root globals + base element defaults
│   ├── interaction_Primitives.css
│   └── header_Css.css          # header, nav, and dropdown chrome
└── Pages/
    └── Posts/                  # long-form surfaces — opted in per layout
        ├── prose.css           # @layer components — prose-block / prose-editorial
        └── posts_Toc.css       # blog post TOC chrome
```

## Naming

- Folders: `PascalCase` (`Base`, `Pages`, `Posts`).
- Files: `snake_Case.css` (`base_Main.css`, `derived_Color_Vars.css`, `interaction_Primitives.css`). Matches the project-wide improved_Camel_Snake convention from the `coding` skill.
- No numeric prefixes on folders or files. Code folders are identified by name, not position.
- Per-page folder named after the surface concern (`Posts`, `Landing`, `About`), not the URL slug or layout filename.

## Entry stylesheet and import chain

`Base/base_Main.css` is the only file that imports `tailwindcss`. It chains the rest in cascade order:

```css
@import "tailwindcss";
@import "./theme.css";
@import "./derived_Color_Vars.css";
@import "./fonts_Base.css";
@import "./root_Vars.css";
@import "./interaction_Primitives.css";
@import "./header_Css.css";
```

Order matters — CSS `@import` rules must come before any other rule, and later files override earlier ones via the cascade. `theme.css` must come before `derived_Color_Vars.css` because derived values depend on base theme variables. Both files may export Tailwind variables through `@theme inline`.

`Layout.astro` imports `base_Main.css` once:

```ts
import "../Styles/Base/base_Main.css";
```

## Page-scoped pattern

When a feature is only used on one page or one layout, give it its own file under `Styles/Pages/<PageName>/` and import it from that layout. Don't add it to `base_Main.css` — the bundle then pays for it on every route.

Example from Partner's `Blog_Post_Layout.astro`:

```ts
import "../Styles/Pages/Posts/prose.css";
import "../Styles/Pages/Posts/posts_Toc.css";
```

`prose.css` is opted into by every long-form layout (blog posts, glossar entries, team detail, service detail, static editorial pages); `posts_Toc.css` only by `Blog_Post_Layout`.

If a page folder grows beyond one stylesheet, optionally add a `<page>_Main.css` that re-imports the others and have the layout import just that one file. Jav_Web does this (`posts_Main.css` re-imports `prose_Style.css`, `prose_Reveal.css`, etc.); Partner does not yet, because each page has at most one stylesheet so far.

## Where new things go

| What you're adding | Where it goes |
|---|---|
| New global element default (e.g. styling `<details>`) | `Base/root_Vars.css` under the existing base/global pattern |
| New reusable component class (`.callout`, `.tag`) | Existing shared component CSS under `Base/`, or page CSS if not global |
| New reusable interaction primitive | `Base/interaction_Primitives.css` |
| New menu/dropdown chrome | `Base/header_Css.css` or its imported header subfile |
| New base color/font/radius token | `Base/theme.css` or the relevant font base file (see [tokens.md](tokens.md)) |
| New derived section/surface color role | `Base/derived_Color_Vars.css`, deriving only from existing theme vars |
| New custom breakpoint variant | `Base/base_Main.css` (see [layers_And_Breakpoints.md](layers_And_Breakpoints.md)) |
| New page-only CSS | `Pages/<PageName>/<feature>.css`, imported by that layout |

For add/rename/move checklists, see [layers_And_Breakpoints.md](layers_And_Breakpoints.md).
