# OG Image System

## Pieces

| Piece | Location | Role |
|---|---|---|
| SEO schema | `src/Scripts/Content_Schemas/schema_Seo.ts` | Defines `seo.og.image` frontmatter shape |
| Naming helpers | `src/Scripts/Astro_Frontmatter/2_Common/Og_Image/og_Img_Name.js` | Converts file paths to deterministic flat names (`en_blog_hello.webp`) |
| Manifest integration | `src/Scripts/Proc_Comm/Og_Img_Gen/manifest_Integration.js` | Astro hook — injects manifest route, moves JSON after build |
| Manifest worker | `src/Scripts/Proc_Comm/Og_Img_Gen/Manifest/worker_Mn.js` | Scans all collections, builds task list, emits `og_Manifest.json` |
| Theme helper | `src/Scripts/Proc_Comm/Og_Img_Gen/Manifest/theme_Helper_Mn.js` | Parses `daisy_Ui_Themes_Init.css`, supplies theme tokens to the manifest |
| Shared constants | `src/Scripts/Proc_Comm/Og_Img_Gen/shared_Consts.js` | Path constants shared between integration and worker |
| External generator | `02_Workfl_Comp/Og_Image` (separate repo) | Bun process — reads manifest, renders WebP via Takumi |

## Flow

```
MDX frontmatter (seo.og.image)
        ↓
  astro build
        ↓  (manifest_Integration hook)
  dist/og_Manifest.json  →  moved to  Cache/Og_Gen/og_Manifest.json
        ↓
  bun src/main.js --project_Root <path>
        ↓
  dist/Og_Gen/<og_img_name>.webp   (one file per opted-in page)
        ↓
  <meta property="og:image"> in layout (0_Init_Layout.astro)
```

## Key design decisions

- **Opt-in only** — only pages with `seo.og.image` in frontmatter get an OG image; others emit no `og:image` tag.
- **Deterministic naming** — OG image name is derived from the content file path, not a hash. Same file = same name across builds.
- **No project-specific fonts** — the generator uses its built-in Inter (Latin) and Noto Sans SC (Chinese) fallbacks. `lang_Font_Map` entries exist for all 5 supported langs with `null` primary/secondary/accent.
- **Themes** — `theme_Jav_0` and `theme_Jav_1` are whitelisted. Both must exist in `daisy_Ui_Themes_Init.css` or the build fails hard.
