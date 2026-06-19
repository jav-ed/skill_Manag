# Post_Img

`Post_Img` is the MDX image helper for prose images, captioned editorial images, and click-to-zoom post media. It wraps Astro's `<Picture>` so authors can use one stable component while the project controls format output, responsive candidates, loading behavior, and lightbox wiring.

This component carries policy, not just markup. In Partner, inline article images intentionally request a higher-resolution candidate than their rendered CSS width when one is available, because visual testing showed the larger candidate looked sharper on DPR=1 laptop screens.

## Author Shape

```mdx
import hero from "./hero.png";

<Post_Img src={hero} alt="A clear and accurate description" />
```

Source images must be ES-imported. Do not pass string paths. Astro's image pipeline only optimizes imported assets at build time.

## Render Shapes

`<Post_Img src={img} alt="..." />` renders a neutral button wrapping `<Picture>`. The image participates in the site-wide lightbox through `data-lightbox-*` attributes.

`<Post_Img src={img} alt="..." caption="..." gallery="erstes-mal" aspect="aspect-[3/2]" />` renders a `<figure>` with `<figcaption>`, optional crop, and gallery grouping for previous/next lightbox navigation.

`<Post_Img src={img} alt="..." zoom={false} />` renders a bare `<Picture>` with no lightbox wrapping. Use this for diagrams, decorative thumbnails, or images where click-to-zoom adds nothing.

## Aspect Ratio

`Post_Img` does not crop by default. The default class uses `w-full h-auto`, so inline blog images preserve the source file's natural aspect ratio.

For AI-generated editorial inline blog photos, prefer a `4:3` source ratio unless the content clearly needs another shape. Partner's current inline blog photos use `1448x1086`, which is exactly `4:3`, and this reads well in the prose column.

Only pass `aspect="..."` when the author intentionally wants a cropped visual moment, such as a static-page image or a gallery-style figure. Once `aspect` is passed, the source image should be composed for that exact visible crop, with safe space around important subjects.

Do not force `4:3` onto diagrams, charts, screenshots, or technical images. Those assets should keep the ratio that makes the information legible.

## Formats

Use AVIF as the primary source format and WebP as the fallback:

```astro
formats={["avif"]}
fallbackFormat="webp"
```

Do not add JPEG or PNG fallbacks for normal photos. Global WebP support is sufficient, and extra fallbacks multiply generated files without meaningful browser coverage gains.

## Widths And Sizes

`widths` define which image candidates Astro can generate. `sizes` tells the browser how large the image is expected to render, and therefore which candidate to download for the current DPR.

Partner defaults:

```js
const default_Widths = [720, 1080, 1200];
const default_Sizes = "(min-width: 768px) 960px, 100vw";
```

The inline default deliberately overspecifies the desktop slot. It does not describe the exact CSS width; it biases the browser toward a larger real candidate so it can downsample to the rendered prose width.

## Sharpness Finding

Playwright check on `/de/blog/erster-zahnarztbesuch-kind/`, viewport `1418x807`, DPR `1`:

- Rendered inline image width: `672px`.
- `sizes="(min-width: 768px) 672px, 100vw"` made Chrome select the `720w` AVIF candidate.
- `sizes="(min-width: 768px) 960px, 100vw"` made Chrome select the `1080w` AVIF candidate.
- Manual visual comparison showed the `1080w` candidate looked better.

This is why inline `Post_Img` keeps the `960px` desktop hint even though the rendered image may be closer to `672px`.

## No-Upscale Guarantee

This policy is not upscaling. Astro refuses to generate image files wider than the original source image. The larger `sizes` hint only affects browser selection among real candidates Astro emitted.

If a source image is large enough, the browser can choose the higher-resolution candidate. If the source image is too small, Astro caps the candidate list at the source's real dimensions and the browser gets the largest real candidate available.

This means template authors can provide high-quality source images and let `Post_Img` use them, while small images are not artificially enlarged by the build pipeline.

## Author Overrides

Use `widths` only when an image appears at a substantially different size than the prose default, such as a small thumbnail or a source image with unusual dimensions.

Use `sizes` when you know the rendered layout differs from the component default. Keep `sizes` and `widths` aligned; `sizes` that points beyond every candidate simply makes the browser pick the largest available file.

Use `class` only for unusual layout cases. The default targets inline prose images with `w-full h-auto rounded-sm my-10`.

Use `loading="eager"` only for above-the-fold images. Normal post images stay `lazy`.

Use `zoom={false}` when the lightbox would be confusing or redundant.

## Related Docs

- [Images](../../Areas/image.md): when to reach for `<Picture>` vs `<Image>`, per-surface sizing policies, the four-knob loading playbook. Build-time machinery (breakpoints, layout, cache) lives in [image_Pipeline.md](../../Areas/image_Pipeline.md).
- [AI image prompt briefs](../../Services/Ai_Img_Prompt/linker_Ai_Img_Prompt.md): when generating an inline blog image via AI, write a prompt brief that embeds the chosen aspect ratio (typically `4:3` for prose images) and the universal content rules (no faces, no real-premises). The brief goes under `Scratch/Img_Prompts/`; the operator copies it into the image tool.
- [Post covers](../post_Covers.md): `16:9` cover images and blog-index thumbnails. Covers are a stricter contract than inline prose images.
- [Component map](component_Map.md): how `Post_Img` becomes available to MDX authors.
- [Blog overview](../linker_Blog.md): how post layout, prose, TOC, and MDX helpers fit together.
