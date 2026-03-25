# Icons Setup

## Approach

Icons are imported as SVG files directly from npm packages (`lucide-static`, `@phosphor-icons/core`, `heroicons`). No icon component library (`astro-icon`) is used — Astro treats `.svg` imports as components natively.

Why direct imports instead of astro-icon:

- Tree-shaking — only used icons are bundled
- No runtime overhead — SVGs are inlined at build time
- Standard Astro component syntax — no special `<Icon>` API to learn
- SVGO optimizes all imported SVGs automatically

## SVGO optimization

Astro's `experimental.svgo` runs [SVGO](https://svgo.dev/) on every imported SVG component during production builds. No optimization in dev mode — keeps rebuilds fast.

### Config location

```
src/Scripts/Astro_Frontmatter/Astro_Config/svgo_Config.ts
```

Imported in `astro.config.mjs`:

```js
import { svgo_Config } from "./src/Scripts/Astro_Frontmatter/Astro_Config/svgo_Config.ts";

export default defineConfig({
  experimental: {
    svgo: svgo_Config,
  },
});
```

### What it does

Starts from `preset-default` (SVGO's recommended baseline), then disables plugins that break icons:

| Plugin disabled | Why |
|---|---|
| `removeViewBox` | Icons cannot scale without viewBox |
| `cleanupIds` | Breaks clipPaths and masks in icon sets |
| `mergePaths` | Breaks duotone/layered icons (e.g. Phosphor Duotone) |
| `collapseGroups` | Destroys `<g>` layer structure in dual-tone icons |

Additional settings:

| Setting | Value | Why |
|---|---|---|
| `floatPrecision` | `2` | Reduces file size without visual harm |
| `multipass` | `true` | Multiple optimization passes for better output |
| `removeXMLNS` | enabled | Safe for inline HTML5 SVGs |
| `removeDimensions` | disabled | Kept off since viewBox handles scaling |

### Design principle

Safe optimizations only. If unsure whether a plugin could break an icon, disable it. Visual correctness over file-size savings.
