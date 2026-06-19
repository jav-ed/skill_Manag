# Image pipeline — deep dive

How Astro's `astro:assets` + Sharp integration turns a source import into a responsive `<picture>` element at build time: what `<Picture>` actually expands to, how the breakpoints list shapes browser candidate selection, the rationale behind `layout: "constrained"` + `responsiveStyles: false`, and how the content-hashed cache under `.astro/` works. Use this when modifying breakpoints, debugging an unexpected emit, or porting the pattern to a new project. For the author-facing usage (when to reach for `<Picture>`, per-surface sizing policies, the four-knob loading playbook, common mistakes), see [image.md](./image.md).

## What `<Picture>` produces

A single `<Picture src={img} formats={["avif"]} fallbackFormat="webp" alt="..." />` expands at build time to:

1. **One or more `<source>` elements** — one per format in `formats={[...]}` (here: avif), each with a `srcset` listing every breakpoint variant.
2. **An `<img>` element** — the `fallbackFormat` (here: webp), also with a `srcset` of breakpoint variants. Browsers that can't handle any `<source>` format fall back to this.
3. **A `sizes` attribute** — Astro infers it from `layout` + the breakpoints. The browser uses it to decide which size in the srcset to download.

The browser walks the `<source>` list, takes the first format it supports, reads `sizes`, multiplies that layout width by the device DPR, and picks a matching candidate from `srcset`. Visitors with modern browsers get AVIF; legacy browsers get WebP. Whether the downloaded candidate is byte-minimal depends on whether `sizes` is literal or intentionally generous.

## Breakpoints config

Configured in `astro.config.mjs`:

```js
image: {
  responsiveStyles: false,
  layout: "constrained",
  breakpoints: [640, 1280, 1668, 1920, 2048, 2560, 3200, 3840, 4480, 5120, 6016],
}
```

Each number is a pixel width Astro generates a variant for. The 640 entry covers mobile; the upper end (2560+) covers retina desktop and the rare 4K case. Astro skips breakpoints **larger than the source image** — a 768px source gets just 640w + 768w variants per format, not the full list.

This means `sizes` can bias the browser toward a larger candidate, but it cannot create fake pixels. If a component asks for a larger candidate and the source image is not large enough, Astro emits only the real source-capped candidates and the browser picks from those. Use that fact deliberately for editorial images: a blog `Post_Img` can overspecify its desktop `sizes` hint to get sharper downsampling when a larger real variant exists, while still staying bounded by the imported source image. The blog-specific policy is documented in [`../Blog/MDX_Components/post_Img.md`](../Blog/MDX_Components/post_Img.md).

**Tuning the list.** More breakpoints = finer granularity but more build time and more cached files in `dist/_astro/`. Partner uses the 11-entry editorial list because it serves higher-resolution photography and benefits from finer mobile→retina gradation. Smaller content sites can use a shorter list, but don't extend or shrink it without a concrete reason; every entry changes build time, CDN cache size, and the browser's available candidates.

The single highest-value entry is **640** — savings between full-size and 640px are large on mobile, where the bulk of traffic sits. Never remove this one.

## Layout modes and the Tailwind interaction

`layout: "constrained"` — images scale within their container but never exceed source dimensions. Best general default for content pages.

Other Astro layouts (`responsive`, `full-width`, `fixed`) exist but aren't used in this org's projects today. Pick `constrained` unless a specific page has hero/full-bleed semantics that the layout explicitly handles.

`responsiveStyles: false` — Astro normally emits a small inline style block to enforce its layout semantics. We disable this because **Tailwind handles all image sizing** via utility classes (`w-full h-full object-cover` etc.) and the framework styles fight each other. Without `responsiveStyles: false`, expect specificity bugs around `width: 100%` and `aspect-ratio`.

## Cache and rebuild

Astro caches transforms under `.astro/` (and `node_modules/.astro/`). Variants are content-hashed (`hero_A.Q4jJdgDX_25nHMW.avif`), so renaming the source or changing the binary forces a fresh emit; renaming around them in source code does not. To force a clean transform run, delete `.astro/` and re-build.

`bun run dev` does on-demand transforms and may mask cases where a missing import would fail in production. Run `bun run build` after image work; the output lists every emitted variant.

## Related pipelines

OG image generation is a **separate** pipeline (manifest builder in `src/Scripts/OG_Images/` + external Bun renderer) — it reads from `src/Assets/Imgs/` but does not go through this `<Picture>` pipeline. For the build-time internals of that pipeline see [`../Scripts/og_Images_Pipeline.md`](../Scripts/og_Images_Pipeline.md); for the usage view see [`../Scripts/og_Images.md`](../Scripts/og_Images.md).
