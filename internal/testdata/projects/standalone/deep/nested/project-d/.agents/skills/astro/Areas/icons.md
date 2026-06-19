# Icons

Icons in Astro projects in this org are SVG files imported directly as components. **Lucide is the only icon library.** Custom or brand SVGs that Lucide does not cover live in `src/Icons/`. Geometry computed from runtime data (progress bars, charts, language rings) uses inline `<svg>` directly. Astro treats `.svg` imports as components natively — tree-shaking is automatic and there is no runtime overhead.

This file owns the technical pipeline: aliases, imports, custom SVG placement, `src/Icons/` vs `public/`, SVGO, and build verification. For visual icon usage, arrow semantics, color, sizing, and Tier 3 restraint, use [Design/icons.md](../Design/icons.md).

## The rule, in one table

| Situation | What to use |
|---|---|
| Standard UI icon (arrow, chevron, menu, sun, shield, gem, baby, leaf…) | `import X from "@lucid/<icon-name>.svg"` |
| Brand / custom icon Lucide doesn't have (LinkedIn, Twitter, Java logo, partner logo…) | Add to `src/Icons/<Category>/<name>.svg`, import directly |
| SVG referenced by URL only (favicon, OG default placeholder) | `public/<name>.svg` — not optimized, served as-is |
| Geometry computed from runtime data (progress bar paths, charts) | Inline `<svg>` directly in the component — exempt from this system |

If in doubt, default to Lucide. If Lucide doesn't have it, search again — Lucide's catalog is large and many "obvious" custom requests already exist there.

## Wiring

Two places need a one-time setup per project.

**`tsconfig.json`** — defines the import alias for TypeScript:

```json
{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@lucid/*": ["node_modules/lucide-static/icons/*"]
    }
  }
}
```

**`astro.config.mjs`** — defines the same alias for Vite at build time:

```js
vite: {
  resolve: {
    alias: {
      "@lucid": path.resolve(__dirname, "./node_modules/lucide-static/icons"),
    },
  },
}
```

Both are required: tsconfig keeps the editor happy, the Vite alias makes the import actually resolve during the build. Partner has both already.

Package: `lucide-static` (already in `package.json` in Partner). The `-static` variant ships flat `.svg` files, which is what we want — the React/Vue variants ship JSX components we don't need.

## Import and render

```astro
---
import Lucid_Sun from "@lucid/sun.svg";
import Lucid_Moon from "@lucid/moon.svg";
import Lucid_Arrow_Right from "@lucid/arrow-right.svg";
---

<Lucid_Sun class="size-5 text-primary" />
<Lucid_Moon class="size-5 hidden group-hover:block" />
<Lucid_Arrow_Right class="size-4" />
```

The imported value is an Astro component. Astro inlines the SVG into the HTML at build time — no runtime, no fetch, no flash of unstyled icon.

## Naming convention

Component names follow the org's `Lucid_<Icon_Name>` pattern. The file name from Lucide maps directly with underscored PascalCase:

| Lucide file | Component name |
|---|---|
| `sun.svg` | `Lucid_Sun` |
| `arrow-right.svg` | `Lucid_Arrow_Right` |
| `chevrons-down.svg` | `Lucid_Chevrons_Down` |
| `file-text.svg` | `Lucid_File_Text` |
| `shield-check.svg` | `Lucid_Shield_Check` |

Always prefix with `Lucid_`. Even if the icon is used only once, the prefix makes it grep-friendly across the codebase and prevents collisions with project-specific component names (a `Sun` component would shadow `Lucid_Sun` otherwise).

## Sizing

Use Tailwind's `size-*` utility (sets both `width` and `height`):

| Pixel value | Tailwind class |
|---|---|
| 14px | `size-3.5` |
| 16px | `size-4` |
| 20px | `size-5` |
| 24px | `size-6` |
| 28px | `size-7` |
| 32px | `size-8` |
| 36px | `size-9` |
| 40px | `size-10` |

For odd sizes the design genuinely needs, use Tailwind's arbitrary value syntax: `size-[23px]`, `size-[42px]`. Don't reach for arbitrary values when a standard step is close enough — the standard set keeps the visual rhythm consistent across the site.

The SVG's intrinsic `width` / `height` attributes are removed by SVGO (`removeXMLNS` + viewBox preservation), so the Tailwind class is the only thing controlling size.

## Color

Lucide icons use `currentColor` for stroke and fill. Set color with Tailwind text utilities:

```astro
<Lucid_Sun class="size-5 text-primary" />
<Lucid_Shield_Check class="size-6 text-emerald-600" />
<Lucid_Arrow_Right class="size-4 text-muted-foreground" />
```

For runtime-dynamic colors (CSS variable from data), use inline style:

```astro
<Lucid_Sun class="size-6" style={`color: var(--color-${theme_Tone})`} />
```

Never hard-code stroke / fill colors in the SVG file. The `currentColor` contract is what makes one icon file work everywhere.

## Custom SVGs — `src/Icons/`

For brand or custom icons Lucide does not carry — partner logos, social-media glyphs (when official mark is needed), tech-stack badges, regional symbols — ship the file under `src/Icons/<Category>/<name>.svg` and import like any Lucide icon:

```astro
---
import Lucid_Mail from "@lucid/mail.svg";
import Brand_Linkedin from "../Icons/Social/linkedin.svg";
---
<Brand_Linkedin class="size-5" />
```

### Folder shape

```
src/Icons/
├── Brand/                     # logos, marks, wordmarks
│   └── <name>.svg
├── Social/                    # platform glyphs (LinkedIn, X, Mastodon, Rumble)
│   └── <name>.svg
└── Tech/                      # technology badges (Java, Python, Matlab)
    └── <name>.svg
```

Conventions, all derived from the org-wide rules in [`file_Structure.md`](./file_Structure.md):

- **Capital `Icons/`** — matches `Assets/`, `Scripts/`, `Styles/`, `Content/`, `Utils/`. The casing is org convention; SVGO itself is folder-name-agnostic (see Rule 2 below), but the convention is non-optional for findability.
- **PascalCase category subfolders** (`Brand/`, `Social/`, `Tech/`), only when 3+ icons cluster naturally. With fewer, sit flat at the top of `Icons/`.
- **No numeric prefixes** on folders or files.
- **Lowercase file names**: `linkedin.svg`, `mastodon.svg`, `java.svg`. Match the Lucide convention so import names align (`Brand_Linkedin`, `Tech_Java`).

### Why `src/Icons/` and not `public/` — two separate rules

These two rules are often conflated. They are not the same.

**Rule 1 — `src/` vs `public/` decides whether SVGO runs.** SVGO is wired into Astro's import pipeline (see the next section). Anything imported from `src/` (or resolved through a Vite alias like `@lucid/*`) goes through that pipeline and gets optimized. Files in `public/` are copied verbatim into `dist/`, no transform, no SVGO. That's why a brand SVG belongs under `src/` — to inherit the optimization for free.

**Rule 2 — `src/Icons/` (specifically) is org convention, not SVGO requirement.** SVGO does not care about the folder name. It would optimize `src/Icons/foo.svg`, `src/Pretty_Pictures/foo.svg`, or a one-off `src/components/Logo.astro` neighbor file identically. We centralize under `Icons/` for human reasons: greppability, mirroring `Assets/` / `Styles/` / etc., preventing custom SVGs from scattering across the codebase as a hidden junk drawer.

So the decision tree is:

- Imported as a component, needs optimization → `src/Icons/` (Rule 1 + Rule 2)
- Imported through `node_modules` (Lucide via `@lucid/*`) → already optimized via Rule 1, no folder of ours needed
- Referenced by URL only (favicon, OG default placeholder) → `public/` — SVGO can't reach it, but those files need URL stability more than they need byte savings

`public/<name>.svg` is the right home for files that appear in `<link rel="icon" href="/favicon.svg">` or `<meta property="og:image">`. Never put a Lucide-replacement icon there.

### Naming the import

Mirror Lucide's `Lucid_<Name>` pattern with a category prefix instead:

| Subfolder | Prefix | Example |
|---|---|---|
| `Brand/` | `Brand_` | `Brand_Logo` |
| `Social/` | `Social_` | `Social_Linkedin` |
| `Tech/` | `Tech_` | `Tech_Java` |

Same goal as `Lucid_`: greppable, collision-proof, immediately recognisable.

## Programmatic / data-driven SVGs

Inline `<svg>` is **required**, not banned, when the geometry depends on runtime data. Two canonical patterns:

- **Progress bars** whose `<path d="..."` is written by JS as scroll progresses (Jav_Web's TOC progress bar is the reference).
- **Charts / language rings** whose `<circle>` or `<path>` attributes are computed from content-collection data (Jav_Web's CV language proficiency rings).

These are data visualisations, not icons. Do not move them into `src/Icons/`. Do not try to replace them with Lucide imports. They are exempt from the icon system on purpose.

## SVGO config

Astro's `experimental.svgo` runs [SVGO](https://svgo.dev/) on every imported `.svg` during production builds. Dev mode skips optimization to keep rebuilds fast.

### Wiring

The canonical config lives as a real TypeScript file in this skill: [`../Templates/svgo_Config.ts`](../Templates/svgo_Config.ts). Copy it verbatim into the project at:

```
src/Scripts/Astro_Frontmatter/Astro_Config/svgo_Config.ts
```

The template is the source of truth — if a real project needs to deviate, update the template first (with reasoning in the file header), then copy out. The shape, in summary:

```ts
export const svgo_Config: svgo_Config_Typ = {
  floatPrecision: 2,
  multipass: true,
  plugins: [
    "preset-default",
    { name: "removeViewBox",  active: false },   // see table below for why each is off
    { name: "cleanupIds",     active: false },
    { name: "mergePaths",     active: false },
    { name: "collapseGroups", active: false },
    "removeXMLNS",
    { name: "removeDimensions", active: false },
  ],
};
```

Then import it in `astro.config.mjs`:

```js
import { svgo_Config } from "./src/Scripts/Astro_Frontmatter/Astro_Config/svgo_Config.ts";

export default defineConfig({
  experimental: {
    svgo: svgo_Config,
  },
});
```

### What it does, and what it deliberately doesn't

Start from `preset-default` (SVGO's recommended safe baseline), then disable the few plugins that break icons:

| Plugin | Disabled | Why |
|---|---|---|
| `removeViewBox` | yes | Without `viewBox`, the icon can't scale — `size-5` would have no effect |
| `cleanupIds` | yes | Mangles IDs inside `<clipPath>` and `<mask>`, breaking many icon sets |
| `mergePaths` | yes | Flattens layered icons (duotone variants, animations) into a single broken path |
| `collapseGroups` | yes | Destroys `<g>` layer structure dual-tone icons depend on |
| `removeXMLNS` | enabled (custom) | Safe for inline HTML5 — saves a few bytes per icon |
| `removeDimensions` | disabled | Kept off; the `width`/`height` strip is unsafe in some renderers, viewBox alone is enough |

Plus `floatPrecision: 2` (cuts decimal precision below the visible threshold) and `multipass: true` (multiple optimisation passes catch reductions a single pass misses).

The design principle is **safe optimizations only**. If unsure whether a plugin could break an icon, disable it. The few bytes saved by a risky plugin are never worth a broken icon in production.

## Arrow conventions — cross-site hard rule

Arrows communicate link type. The mapping is fixed across every project in this org:

| Visual | Lucide file | Meaning | Examples |
|---|---|---|---|
| `→` (arrow-right) | `arrow-right.svg` | Internal navigation — stays on the site | "More about", blog-card "Read more", service-card links |
| `↗` (arrow-up-right) | `arrow-up-right.svg` | External link — leaves the site or context | `mailto:`, social profiles, third-party references |
| `↓` (chevron-down) | `chevron-down.svg` | Expand / collapse in place | FAQ accordion, disclosure toggles |

**Never substitute a text character** (`→`, `↗`, `↓`) for the icon. Text characters render with the font, which changes their weight, baseline, and metrics across faces — the SVG keeps the visual constant.

**Hover animation for `arrow-right`** — the arrow is hidden by default and slides in from the left on hover:

```html
<a class="group inline-flex items-center gap-2">
  Read more
  <Lucid_Arrow_Right class="size-4 opacity-0 -translate-x-1
                            transition-all duration-150
                            group-hover:opacity-70 group-hover:translate-x-0" />
</a>
```

## Adding a new icon — checklist

1. **Search Lucide first** at <https://lucide.dev/icons>. If it exists, use `import Lucid_<Name> from "@lucid/<file>.svg"`. Done.
2. **If Lucide doesn't have it** — drop the file under `src/Icons/<Category>/<name>.svg`. Confirm the file uses `currentColor` for stroke/fill (open it, search for hard-coded `fill="#..."` or `stroke="#..."`; replace with `currentColor`).
3. **Import** using the matching `<Category>_<Name>` component name.
4. **Render** with a `size-*` class and a `text-*` class for color.
5. **Verify** in `bun run dev` first, then `bun run build` — production builds will apply SVGO and any breakage shows up there, not in dev.

## Renaming / moving a custom SVG — checklist

1. Grep for the import path:
   ```bash
   grep -rn "Icons/<Old>" src/
   ```
2. Update imports in every consumer.
3. If renaming the file, also update any `aria-label` or alt-text that referenced the old name (rare for icons, common for illustrations).
4. Re-run `bun run build` and verify the SVGO-optimised output still renders correctly.
