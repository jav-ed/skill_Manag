# Images

Astro's image pipeline (`astro:assets` + Sharp) generates responsive AVIF/WebP variants at build time. In this codebase, **`<Picture>` is the default** for almost every image — it emits multiple breakpoint variants and lets the browser pick the right one for the device. `<Image>` (single fixed size, no responsive variants) is a rare exception for tiny decorative assets — covered at the very bottom of this file.

This is the usage view: which wrapper to reach for, the decision flow, the raw-`<Picture>` checklist when no wrapper fits, the LCP playbook, the Partner audit, and common pitfalls. For *where* image files live on disk and how to import them, see [assets.md](./assets.md). For the build-time machinery (how `<Picture>` expands, breakpoints, Sharp behaviour) see [image_Pipeline.md](./image_Pipeline.md).

## The four picture wrappers

This codebase has four named picture wrappers, each owning a specific policy. **Prefer one of these over raw `<Picture>` whenever the surface matches.** Three live in `src/Components/Common/Pictures/`; the fourth lives in `src/Components/Mdx/Default/Media/` because it carries prose-specific extras (lightbox, figure/caption, gallery).

### `Hero_Picture` — generic design heroes

Location: `src/Components/Common/Pictures/Hero_Picture.astro`

For large design-led heroes and editorial panels where **the caller/layout owns the aspect ratio** and supplies an explicit `sizes` contract. The wrapper centralises AVIF/WebP output, hero-scale candidate widths, async decoding, and the LCP `priority` switch — but it stays aspect-ratio-agnostic so the caller decides whether to wrap with `aspect-[X/Y]` + `object-cover` (forced crop) or pass `w-full h-auto` (source-owned ratio).

Used by: `src/Components/Landing/Hero.astro` (landing diptych).

### `Blog_Cover_Picture` — blog post cover

Location: `src/Components/Common/Pictures/Blog_Cover_Picture.astro`

For the blog post cover specifically. Hard-coded as LCP-priority (`loading="eager"` + `fetchpriority="high"`) with a column-bounded `sizes` contract matching the article's max-w-3xl slot and the `[720, 960, 1280, 1600]` candidate ladder.

Expected source: **16:9, crop-safe** — the same source is reused by `Blog_Row_Thumbnail_Picture` at 4:3.

Used by: `src/Components/Posts/Cover_Hero.astro`.

### `Blog_Row_Thumbnail_Picture` — blog index thumbnail

Location: `src/Components/Common/Pictures/Blog_Row_Thumbnail_Picture.astro`

For the small thumbnail on the blog index row. Lazy by default, small candidate ladder (`[240, 360, 480, 720]`), `sizes` matched to the ~192px desktop slot. Reuses the cover source and crops it into 4:3 via `object-cover`.

Used by: `src/Components/Pages/Blog/Blog_Post_Row.astro`.

### `Post_Img` — MDX/prose images

Location: `src/Components/Mdx/Default/Media/Post_Img.astro`

For inline images inside MDX content — blog posts **and** static MDX pages like `first-visit.mdx`. Owns the prose-image policy: explicit candidates, AVIF/WebP, lightbox wiring, optional figure-with-caption mode, column-bounded rendering, and a deliberately generous desktop `sizes` hint for sharper DPR=1 downsampling. See [`../Blog/MDX_Components/post_Img.md`](../Blog/MDX_Components/post_Img.md).

Used by: any `.mdx` content file via the MDX component map.

## Which wrapper for which case

| Surface | Use | Notes |
|---|---|---|
| Inside `.mdx` content | `Post_Img` | Includes lightbox + figure/caption; stays bounded by the prose column. |
| Blog post cover | `Cover_Hero.astro` (wraps `Blog_Cover_Picture`) | Don't use raw `<Picture>` here. |
| Blog index row | `Blog_Post_Row.astro` (wraps `Blog_Row_Thumbnail_Picture`) | Don't use raw `<Picture>` here. |
| Design hero, editorial panel | `Hero_Picture` | Caller owns aspect via wrapper class and supplies `sizes`. |
| **Any other surface** | Inspect the consumer layout. Reuse an existing wrapper only if it matches the policy. Otherwise raw `<Picture>` with the checklist below. Create a new named wrapper only if the surface repeats (≥3 call sites with shared policy) or is high-impact. | |

## Raw `<Picture>` is allowed — with a checklist

When none of the four wrappers fits, raw `<Picture>` from `astro:assets` is acceptable. The global config in `astro.config.mjs` already provides strong defaults — `layout: "constrained"`, the `breakpoints` ladder, and `responsiveStyles: false` so Tailwind owns sizing — so you don't need a custom wrapper to get sensible behaviour. But raw `<Picture>` is **not** a free pass. Run through this checklist at every call site:

- **`formats={["avif"]}` and `fallbackFormat="webp"`** — never `formats={["avif","webp"]}` (emits a redundant `<source>`), never JPEG fallbacks (universal WebP support since 2020).
- **`alt` is present** — empty `alt=""` is valid for purely decorative images; missing `alt` entirely is an a11y bug.
- **`loading` is deliberate** — `"eager"` only above-the-fold or for an LCP candidate; `"lazy"` for everything below the fold. Lazy on an above-the-fold image causes pop-in and hurts LCP.
- **`decoding="async"`** — safe everywhere. Only switch to `"sync"` if you've measured a paint stutter.
- **`sizes` is explicit when the slot is known** — split grids, fixed-width panels, cards in a known column, anything LCP-class. The global `image.breakpoints` in `astro.config.mjs` controls *which variants Astro emits*; `sizes` controls *which one the browser picks*. Setting `sizes` is the load-bearing lever for user-fetched bytes; setting `widths` is not. Astro's inferred `sizes` is sensible only for genuinely full-bleed / fluid surfaces.
- **`widths` is explicit only when the global `image.breakpoints` ladder doesn't fit your slot** — two situations qualify:
  1. **Floor:** the slot is smaller than the global floor (currently `640`) and needs smaller candidates generated (avatars, tiny thumbnails). Without explicit `widths`, the smallest candidate in the srcset is 640w and the browser will fetch that even for a 100px slot.
  2. **Ceiling cleanup:** the global ladder runs up to `6016`, so a small editorial panel still triggers emission of 1668/1920/.../6016 variants to dist. Browsers won't fetch them (assuming `sizes` is correct) but build time and dist size grow. Explicit `widths` caps the ceiling.

  For typical cards, panels, and heroes in the 400-2000px range the global ladder is already correct — skip `widths`.
- **Aspect matches the source** — if the consumer wraps with `aspect-[X/Y]` + `object-cover`, the source must be generated at that aspect or the crop loses the composition. See [AI image prompt](../Services/Ai_Img_Prompt/linker_Ai_Img_Prompt.md) for the prompt-brief workflow.
- **Comment beside the call site** explaining the rendered slot, when the policy is non-obvious.
- **Promote to a wrapper** if the surface repeats (≥3 call sites with shared policy) or is high-impact. Lift into `Common/Pictures/` and remove the duplication.

### Surface patterns for raw `<Picture>`

When raw `<Picture>` is the right choice, three repeating patterns deserve their own guidance.

**Cards, thumbnails, and portraits.** Always set a literal `sizes` — these surfaces have known column widths and the browser needs the hint to pick a small variant from the srcset. Only set explicit `widths` if the slot is below the global floor (`640`); otherwise the global ladder handles it. A portrait card displayed at 280px in a column needs `sizes="280px"` so the browser picks 640w from the existing ladder — it does *not* need explicit `widths` to generate sub-640 candidates that wouldn't help anyway. The exception is a true small thumbnail (e.g. an 80px avatar) where you do need `widths={[80, 96, 160, 192]}` to push below the global floor.

**Galleries and lightbox previews.** The visible preview and the lightbox source are separate concerns. The preview should use a `sizes` value for the visible tile; the lightbox can link to the original imported image via `data-lightbox-src={img.src}`. Don't make every preview heavy just because the enlarged lightbox should be high-quality.

**Wide editorial panels.** Use explicit `sizes` describing the panel width at each breakpoint. Slightly generous values are acceptable when the section is image-led and visual sharpness is part of the design. Document that choice beside the call site.

## Partner Picture Audit

Current picture surfaces outside `Post_Img`:

| Surface | Current state | Policy |
|---|---|---|
| `src/Components/Posts/Cover_Hero.astro` | Uses `Blog_Cover_Picture` with explicit column-bounded cover `sizes`, `[720, 960, 1280, 1600]` candidates, eager LCP, high priority. | Good. Keep the semantic cover wrapper, do not replace it with lazy `Post_Img`, and keep blog-cover picture policy independent from generic design heroes. |
| `src/Components/Pages/Blog/Blog_Post_Row.astro` | Uses `Blog_Row_Thumbnail_Picture` with small thumbnail candidates and a literal thumbnail `sizes` contract. Throws when the post has no cover. | Good. This is an index scan affordance, not the LCP cover or ordinary prose. |
| `src/Components/Landing/Hero.astro` | Uses `Hero_Picture` with explicit split-grid `sizes`; first image is priority. | Good. This is the reference wrapper pattern for new hero surfaces. |
| `src/Components/Landing/Gallery_Section.astro` | Direct `<Picture>` with Astro-inferred `sizes`. | Needs explicit `sizes` for the preview tile slot. Keep previews lighter than lightbox originals. `widths` not needed unless tiles are below 640px. |
| `src/Components/Landing/Featured_Service_Card.astro` | Direct `<Picture>` with Astro-inferred `sizes`. | Needs explicit `sizes` (5fr/7fr split grid in the shell). `widths` not needed — the card is in the global-ladder sweet spot. |
| `src/Components/Landing/Team_Grid.astro` | Direct `<Picture>` with Astro-inferred `sizes`. | Needs explicit `sizes` for the portrait card slot. `widths` not needed unless portraits render below 640px. |
| `src/Components/Landing/Career_Teaser.astro` | Direct `<Picture>` with Astro-inferred `sizes`. | Needs explicit `sizes` (7fr/5fr split grid). `widths` not needed — the slot is ~580px max, comfortably above the global floor. |
| `src/Components/Team/Team_List.astro` | Direct `<Picture>` with Astro-inferred `sizes`. | Needs explicit `sizes` for the portrait row slot. `widths` worth considering only if the row portrait is genuinely small. |
| `src/pages/[lang]/[team]/[doctor_Slug].astro` | Direct `<Picture>` with Astro-inferred `sizes`. | Needs explicit `sizes` for the detail portrait. `widths` not needed at hero-portrait size. |
| `src/pages/[lang]/[practice_tour].astro` | Direct `<Picture>` with Astro-inferred `sizes`. | Needs explicit `sizes` for gallery preview tiles. `widths` not needed unless tiles below 640px. |
| `src/pages/[lang]/[services]/[service_Slug].astro` | Direct `<Picture>` with Astro-inferred `sizes`. | Needs explicit `sizes` for the service hero/panel. `widths` not needed at hero size. |
| `src/pages/[lang]/[career].astro` (HR-contact avatar) | Direct `<Picture>`, **96/112px circular slot** — well below the global floor of 640. Currently no `sizes` / `widths`. | Needs explicit `sizes="(min-width: 768px) 112px, 96px"` AND explicit `widths={[96, 112, 192, 224]}`. This is one of the rare floor-violation cases the doc warns about — without small widths the smallest emitted candidate is 640w, which is ~6× oversampled for a 100px slot. |

## Format choice — AVIF primary, WebP fallback

Use:

```astro
<Picture
  src={img}
  formats={["avif"]}
  fallbackFormat="webp"
  alt="..."
/>
```

AVIF compresses roughly 7× smaller than WebP at comparable quality on large images. WebP shows bigger file-size jumps between breakpoints; AVIF stays flatter, which is what you want when you can't predict the visitor's viewport.

Don't include JPEG/PNG fallbacks — global WebP support has been universal since 2020. Adding `jpeg` to `formats` doubles the emitted file count for a fallback nobody hits.

For images that need transparency (logos, glyph art), use AVIF + WebP both with transparency. Don't reach for PNG.

## Loading attributes — the four-knob playbook

Every `<Picture>` should set:

| Knob | Above the fold (LCP) | Below the fold |
|---|---|---|
| `loading` | `"eager"` | `"lazy"` |
| `decoding` | `"sync"` *or* `"async"` (default) | `"async"` |
| `fetchpriority` | `"high"` | omit |
| Preload? | `<link rel="preload" as="image">` in `<head>` | no |

```astro
{/* LCP candidate */}
<Picture
  src={heroImg}
  formats={["avif"]}
  fallbackFormat="webp"
  alt={heroAlt}
  loading="eager"
  fetchpriority="high"
  decoding="async"
/>

{/* anything else */}
<Picture
  src={img}
  formats={["avif"]}
  fallbackFormat="webp"
  alt={alt}
  loading="lazy"
  decoding="async"
/>
```

`fetchpriority="high"` is the single biggest LCP lever — the browser otherwise defers image fetches behind script and stylesheet downloads. Set it on **exactly one image per page** (the LCP candidate). Setting it on multiple defeats the prioritization.

`decoding="async"` everywhere is safe — modern browsers handle it well and it avoids blocking the main thread on decode. Only switch to `sync` if you've measured a paint stutter on a specific image.

## LCP-critical images — extra steps

For the hero or any image confirmed by Lighthouse / WebPageTest as the LCP element:

1. Set `loading="eager"` + `fetchpriority="high"` on the `<Picture>`.
2. Add a `<link rel="preload" as="image" imagesrcset="..." imagesizes="..." type="image/avif">` to `<head>` so the browser starts fetching before the HTML even parses the `<picture>` element.
3. Confirm `image: { layout: "constrained" }` in `astro.config.mjs` — the right `sizes` attribute is part of an accurate LCP fetch.
4. Verify in DevTools → Performance → LCP element + LCP request. If the request waterfall shows the image fetching after stylesheets, your priority hints aren't sticking.

## Common mistakes

- **Forgetting `alt`.** Empty alt (`alt=""`) is the correct choice for purely decorative images; omitting `alt` entirely is an a11y bug.
- **Same image rendered at hugely different sizes across pages** with one source. Astro picks variants from the breakpoint list, but the source resolution caps the upper end. For a hero used both as 1920px banner and 200px thumbnail, the thumbnail wastes bytes if you don't add a `widths={[200, 400]}` prop or use `<Image>` for the small case.
- **`formats={["avif","webp"]}` instead of `formats={["avif"]} fallbackFormat="webp"`.** The former emits two `<source>` blocks where one is redundant; the latter is the idiomatic Astro 6 pattern.
- **`fetchpriority="high"` on more than one image.** Picks the wrong winner. Score one image at a time.
- **Skipping the build.** `bun run dev` does on-demand transforms and may mask cases where a missing import would fail in production. Run `bun run build` after image work; the output lists every emitted variant.

## `<Image>` for fixed small assets

`<Image>` (singular, no responsive variants) is rare in this codebase. Use it only when an image genuinely doesn't benefit from responsive sizing:

- Small icons that aren't from the Lucide set (see [`icons.md`](./icons.md) — Lucide icons go through `Lucid_<Name>` components, not `<Image>`).
- Fixed-size avatars.
- Decorative graphics ≤ 200px on the longest side.

For everything else — content photography, heroes, editorial panels, cards, thumbnails, anything responsive — use `<Picture>` via one of the four wrappers, or raw `<Picture>` with the checklist above.

---

OG image generation is a separate pipeline (in-repo manifest builder in `src/Scripts/OG_Images/` + external Bun renderer) — it reads from `src/Assets/Imgs/` but does not go through this `<Picture>` pipeline. See [`../Scripts/og_Images.md`](../Scripts/og_Images.md) for that flow.
