# Post Covers

Blog cover images are frontmatter assets that can serve multiple blog surfaces from one source image. They are not ordinary inline MDX images, even when the current visual treatment is column-sized like prose media.

The cover contract is deliberately stricter than prose images: use one `16:9` landscape image that works as the post's main visual lead and as the blog-index row thumbnail source. This keeps customer templates simple because changing the cover image in frontmatter updates both surfaces without requiring a second asset. The row thumbnail may crop the source into a smaller slot, so keep important subject matter away from the extreme edges.

## Frontmatter Shape

Blog schemas should model the cover as a nested object under `mdx_Info`. Some projects may keep the schema field optional while the index is still being designed, but the current Partner index intentionally throws when a visible post has no cover:

```yaml
mdx_Info:
  cover:
    src: ../Post_Assets/0001-example/cover_example.webp
    alt: "Clear description of the cover image"
```

The `src` field should use Astro Content Collections `image()` resolution through the schema. Page routes must preserve the schema-resolved `ImageMetadata` object and must not overwrite it with raw remark frontmatter.

## Aspect Ratio

Use `16:9` for AI-generated cover images. The Partner reference source size is `1672x941`, which is effectively `16:9`.

Why `16:9`:

- It gives the post page a calm editorial lead image without turning the first viewport into a full-screen image wall.
- It lets one cover asset work in both `Cover_Hero.astro` and `Blog_Post_Row.astro`.
- It is easy to brief in AI generation tools and common enough for later manual cropping.

Do not generate a separate index crop unless the project has a specific visual reason. A second crop increases authoring overhead and makes template reuse weaker.

## Consumers

Current Partner consumers:

- `src/Components/Posts/Cover_Hero.astro`: renders the optional cover as the first child inside the article column on the post page.
- `src/Components/Pages/Blog/Blog_Post_Row.astro`: renders the same image as a small index thumbnail and throws when the cover is missing.
- Future OG image variants may also read the cover, but OG generation is a separate pipeline documented under Scripts.

## Component Boundary

`Cover_Hero.astro` is the semantic blog component. It owns the post-cover wrapper, spacing, `16:9` aspect slot, and article placement.

The current Partner layout keeps the cover inside `article.prose` as a `not-prose` first child so the image, title, subtitle, and body share one left edge beside the desktop TOC. This is intentionally calmer than the earlier contained-breakout version above `<main>`, where the cover could dominate the full first viewport on common laptop screens.

`Blog_Cover_Picture.astro` is the blog-cover picture primitive. It owns AVIF/WebP output, cover candidate widths, the column-bounded `sizes` contract, eager loading, async decoding, and `fetchpriority="high"`.

`Hero_Picture.astro` is only for design-led landing/page heroes that pass their own `sizes` contract. Do not route blog covers through it: blog covers are frontmatter-owned article media, not generic hero decoration.

Do not replace `Cover_Hero.astro` with `Post_Img`. `Post_Img` is for MDX-authored body images and defaults to lazy loading plus lightbox behavior. The cover is frontmatter-owned, appears before the article title, and stays `loading="eager"` with `fetchpriority="high"` through `Blog_Cover_Picture` because it can be the LCP candidate.

`Blog_Post_Row.astro` owns its own thumbnail slot through `Blog_Row_Thumbnail_Picture.astro`. Its small `4:3` crop is a scanning affordance on the index, not a second canonical cover ratio. Keep source images crop-safe rather than creating a separate thumbnail asset by default.

## Responsive Image Contract

For the current article-column cover, keep the browser `sizes` hint tied to the article slot, not the old breakout width:

```js
const cover_Sizes = "(min-width: 1024px) 768px, calc(100vw - 2rem)";
```

Keep the candidate widths at:

```astro
widths={[720, 960, 1280, 1600]}
```

Why this list:

- `720` and `960` cover mobile/tablet and avoid the global-breakpoint gap that previously made images look soft.
- `1280` covers larger low-DPR slots and gives browsers a middle choice.
- `1600` is still needed for high-density screens: a `768px` CSS slot at DPR=2 wants about `1536px`.
- `480` is intentionally omitted. The cover is rarely useful at that candidate size because common mobile DPR=2 screens choose `720` or `960`, and the project optimises this surface for sharpness rather than the smallest possible DPR=1 mobile file.

This is different from inline `Post_Img`, whose default sizes intentionally overspecify a prose image to improve perceived sharpness. The cover's `sizes` should describe the real column slot because its LCP status already gives it priority.

The blog-index thumbnail is a different surface. It should keep its own small candidate list, for example `[240, 360, 480, 720]`, with a `sizes` value tied to the rendered thumbnail slot. Here `480` is useful because a `192px` desktop thumbnail at DPR=2 wants about `384px`, and the browser needs a nearby candidate without jumping to a full cover-sized file.

## Generation Checklist

When generating a new blog cover image:

1. Generate a `16:9` landscape source.
2. Leave crop-safe space near the edges because the image may appear in both a wide cover slot and a smaller index thumbnail.
3. Export at least around `1600px` wide so the cover can serve a sharp `1280w` or `1600w` candidate.
4. Put the image under the post's `Post_Assets/<post-folder>/` folder.
5. Reference it from `mdx_Info.cover.src`, not from a hardcoded component import.

## Related Docs

- [Images](../Areas/image.md): when to reach for `<Picture>` vs `<Image>`, per-surface sizing policies, role contracts. The build-time machinery (breakpoints, layout, cache) sits in [image_Pipeline.md](../Areas/image_Pipeline.md).
- [AI image prompt briefs](../Services/Ai_Img_Prompt/linker_Ai_Img_Prompt.md): when generating the actual cover image via AI, write a copy/paste prompt brief that embeds the `16:9` requirement, the crop-safe note, and the universal content rules (no faces, no real-premises). The brief goes under `Scratch/Img_Prompts/`; the operator copies it into the image tool.
- [Blog overview](linker_Blog.md): how post layout, MDX, chrome, and SEO fit together.
- [Post_Img](MDX_Components/post_Img.md): inline blog image behavior. Inline photos usually use `4:3`, but they are not the cover contract.
