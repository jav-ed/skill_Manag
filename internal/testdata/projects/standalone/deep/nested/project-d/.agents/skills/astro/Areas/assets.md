# Assets

Astro projects in this org keep binary source files (images, custom fonts) under `src/Assets/` with exactly two subfolders: `Imgs/` and `Fonts/`. Images participate in Astro's image pipeline — they are imported into `.astro` and `.mdx` modules and rendered through `<Picture>` / `<Image>` from `astro:assets`, which generates responsive AVIF/WebP variants at build time using the `image.breakpoints` config in `astro.config.mjs`.

Reference implementations: `src/Assets/` (this repo, current convention) and `01_Jav_Web/src/Assets/` (the pattern source — still ships `1_Images/` with a numeric prefix, which Partner deliberately dropped per the `file_Structure.md` rule "no numerics on code folders"; Partner also shortens `Images` → `Imgs`).

Note on Astro's default: out of the box, Astro scaffolds `src/assets/` (lowercase). The capital `Assets/` here is an org convention, mirroring how `Scripts/`, `Styles/`, `Content/`, and `Utils/` are also capitalised. The folder has no special meaning in Astro itself — only `public/` does, and that lives separately at the project root for unprocessed static files.

## Folder layout

```
src/Assets/
├── Imgs/                          # all source images
│   ├── <section>/                 # one folder per page or content section
│   │   └── <name>.webp
│   └── <root_image>.webp          # cross-section images can live at Imgs/ root
└── Fonts/
    └── <FontFamily>/              # one folder per family, files only
        └── <font_File>.woff2
```

No numeric prefixes. Folder names are PascalCase or lowercase-by-content-slug (`praxistour/`, `services/`, `team/`); image filenames are `snake_case.webp` or kebab when the slug is already kebab from a content collection.

`src/Assets/` itself is the convention boundary. If a binary doesn't belong under `Imgs/` or `Fonts/`, it likely belongs in `public/` (unprocessed, served as-is) — not as a third sibling of `Imgs/`.

## `Imgs/` — page/section subfolders

Group images by the page or content section that consumes them. The subfolder name matches the route slug or the conceptual section:

| Subfolder | Used by |
|---|---|
| `Imgs/team/` | `/team/` index + `/team/[slug]/` profile pages |
| `Imgs/services/` | `/leistungen/[slug]/` detail pages (hero images named `<slug>_hero.webp`) |
| `Imgs/praxistour/` | `/praxistour/` station gallery |
| `Imgs/praxisphilosophie/` | MDX content page (imported from `static_Pages/<lang>/praxisphilosophie.mdx`) |
| `Imgs/erstes_mal/` | MDX content page (`erstes-mal.mdx`) |
| `Imgs/featured/` | Homepage featured-service card |

Root-level files (`Imgs/hero_A.webp`, `Imgs/gallery_*.webp`) are reserved for cross-section imagery used directly by a top-level component like the homepage hero. When in doubt, prefer a subfolder.

**`_alt_imagery/` convention.** When alternate roundings of an image set are needed (e.g. a second design pass), nest them as `Imgs/_alt_imagery/<section>/` rather than as a sibling of `Imgs/`. The underscore prefix signals "not the primary set"; the structure mirrors the primary subfolder tree.

## `Fonts/` — where files live

`src/Assets/Fonts/<Family>/<weight_Style>.woff2` is the on-disk home for any font that cannot come from Fontsource (licensed commercial faces, custom in-house faces, niche scripts). Jav_Web ships `Assets/Fonts/Quran/Al_Qalam_Quran_Regular.woff2` for a Qur'anic Arabic typeface — Fontsource has no equivalent.

Folder shape:

```
src/Assets/Fonts/
└── <FamilyName>/                 # PascalCase, one folder per family
    └── <weight_Style>.woff2       # e.g. Al_Qalam_Quran_Regular.woff2
```

Files are wired into the build through `fontProviders.local()` in `astro.config.mjs`, not hand-written `@font-face`. **Never reach for Google as a fallback** — if Fontsource doesn't carry the face, it goes here as a local file. See [Fonts](../Fonts/linker_Fonts.md) for the full Fonts API config, provider choice, preload pattern, weights gotcha, and Tailwind hookup.

Partner has no `Fonts/` folder yet because every face it uses is on Fontsource. The folder appears the moment a typeface needs to be hand-licensed or is otherwise unavailable from Fontsource.

## How images are loaded into components

Three patterns, all going through `astro:assets`:

**1. Direct import (single known image)**

```ts
import { Picture } from "astro:assets";
import heroA from "../Assets/Imgs/hero_A.webp";
```
```astro
<Picture src={heroA} formats={["avif"]} fallbackFormat="webp" alt="..." />
```

The import returns an `ImageMetadata` object (width, height, format, src). `<Picture>` produces responsive AVIF + WebP variants using the breakpoints in `astro.config.mjs`. Always provide `alt`. Use `loading="eager" fetchpriority="high"` for the LCP candidate; `loading="lazy"` otherwise.

**2. Dynamic import via `import.meta.glob` (data-driven sets)**

When the image to render is chosen at runtime from a data array — services, team members, gallery stations — eagerly glob the whole subfolder and key into the map:

```ts
import type { ImageMetadata } from "astro";

const teamPhotos = import.meta.glob<{ default: ImageMetadata }>(
  "../Assets/Imgs/team/*.webp",
  { eager: true },
);

const key = `../Assets/Imgs/team/${member.photo}.webp`;
const photo = teamPhotos[key]?.default;
```

The glob path is relative to the file containing it. Vite resolves it at build time, so the path must be a string literal — you can't `${variable}` your way inside the glob argument, only inside the key lookup.

**3. MDX import**

Inside an `.mdx` file, the import statement sits at the top above the body. The imported `ImageMetadata` is passed as a prop to a wrapper component like `Lightbox_Figure`:

```mdx
import Lightbox_Figure from "../../../components/Mdx/Lightbox_Figure.astro";
import philosophyA from "../../../Assets/Imgs/praxisphilosophie/ansatz.webp";

<Lightbox_Figure src={philosophyA} alt="..." caption="..." gallery="philosophy" />
```

The wrapper internally uses `<Picture>`, so MDX inherits the same optimisation pipeline.

## Adding a new image — checklist

1. Decide the section: pick or create `src/Assets/Imgs/<section>/`. Cross-section image → `Imgs/` root.
2. Save as `.webp` (or `.avif` if source quality demands it). Use lowercase descriptive names: `kinder_hero.webp`, `gallery_reception.webp`.
3. Import it in the consuming `.astro` / `.mdx` file using a relative `../Assets/Imgs/...` path.
4. Render via `<Picture>` from `astro:assets` (or `<Image>` for non-responsive cases). Provide `alt`. Set `loading` and `fetchpriority` based on whether it is the LCP candidate.
5. Verify the dev server picks it up (`bun run dev` → hit the page). The first transform takes a moment; subsequent loads hit the `_astro/` cache.

## Renaming or moving — checklist

Astro asset paths are resolved relative to the importing file, so a move/rename touches every importer. Renaming `src/assets/` → `src/Assets/Imgs/` in Partner touched:

- Component imports (`Hero.astro`, `GallerySection.astro`, `FeaturedServiceCard.astro`, `TeamGrid.astro`, `TeamList.astro`, `Mdx/Lightbox_Figure.astro`)
- Page-level glob patterns + key templates (`pages/[lang]/team/[slug].astro`, `pages/[lang]/leistungen/[slug].astro`, `pages/[lang]/praxistour.astro`)
- All MDX content imports (`static_Pages/{de,en,es}/praxisphilosophie.mdx`, `erstes-mal.mdx`)
- The build-time `og_Imgs_Rel_Path` constant in `src/Data/Og_Img/og_Output_Config.js` — **load-bearing**: the OG manifest plugin uses it to locate images. A path rename that misses this leaves OG card generation broken silently (build succeeds, manifest entries point nowhere).
- Doc references in `DESIGN.md` and `PRODUCT.md`

Steps:

1. Grep for both the folder name and the literal subpaths used in imports:
   ```bash
   grep -rn "Assets/\|assets/" src/ astro.config.mjs DESIGN.md PRODUCT.md
   ```
2. Update relative import paths in `.astro`, `.mdx`, `.ts`, `.js`, `.mjs` files. Refac tools miss `.astro` frontmatter and `import.meta.glob` string literals — do these by hand.
3. Update glob patterns AND the key template that addresses them — they must be kept in lockstep:
   ```ts
   import.meta.glob("../Assets/Imgs/team/*.webp", ...)   // pattern
   const key = `../Assets/Imgs/team/${slug}.webp`;       // key — must match
   ```
4. Update doc comments and example code blocks that reference the old path.
5. Run `bun run build` (not just `bun run dev`) before declaring the rename done. Build catches missing imports and fails loudly; the dev server sometimes papers over them via HMR cache.
