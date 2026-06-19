# Writing AI Image Prompts

Conventions for AI-generated source-image prompt briefs across every Astro project in this org.

This leaf is for agents writing **copy/paste-ready prompt Markdown**, not for agents directly generating images. The normal output is a `.md` file under `Scratch/Img_Prompts/`. The operator then copies those prompts into their image-generation tool.

For universal content rules (no faces, no hands, no fake-premises imagery), see [`linker_Ai_Img_Prompt.md`](./linker_Ai_Img_Prompt.md). For per-project aesthetic flavor (palette, register, light direction, industry-specific rules), see the project's `Project_Manag/Docs/Brand/imagery_Brief_*.md`.

## Output

Write prompt briefs to:

```text
Scratch/Img_Prompts/
```

Use the [improved naming convention](../../../coding/languages/improved_Version.md) for prompt-brief files: lower-case generic role first, then significant concepts in Pascal-style chunks separated by underscores. Examples:

```text
Scratch/Img_Prompts/blog_Cover_First_Dental_Visit.md
Scratch/Img_Prompts/service_Hero_Preventive_Care.md
Scratch/Img_Prompts/landing_Teaser_Career.md
```

Each prompt brief should be easy to copy from. Keep reasoning, surface notes, and implementation notes outside the prompt blocks.

Recommended structure:

````md
# Image Prompts — <Page / Surface>

## Surface

- Consumer: `<component or content file>`
- Output file: `<src/Assets/Imgs/... path>`
- Intended use: `<blog cover / inline blog image / service hero / landing teaser / ...>`
- Aspect ratio: `<16:9 / 4:3 / source-owned / ...>`
- Target dimensions: `<W × H, source ≥ 1.5× target>`
- Rationale: `<one short sentence for the user, not part of the prompt>`

## Prompt — <short label>

```text
<copy/paste prompt body>
```
````

The conversion leaf [`img_Conversion.md`](./img_Conversion.md) handles what to do once the operator returns generated images.

## Prompt clause

Every prompt needs clear subject framing plus the universal safety clause.

Good subject framing:

```text
An empty consultation room with morning light.
A wooden tray with dental instruments, no people.
A back-turned figure walking into a sunlit hallway, face not visible.
An abstract geometric pattern in warm earth tones.
```

Required clause (every prompt body, regardless of tool):

```text
No people, no animals, no faces, no human or animal figures with visible features. Subject must be non-living or fully faceless.
```

## Aspect ratios

The prompt author must know the target surface before writing the prompt. Do not hide this judgment inside the prompt body; state it in the Markdown surface notes so the user can sanity-check it.

### Hard rule

**Blog cover source images must be `16:9`.**

The same source is also cropped into a `4:3` slot on the blog index by `Blog_Post_Row.astro`, so the prompt should ask for crop-safe composition: keep important subject matter away from the extreme edges. The canonical cover rationale lives in [`../../Blog/post_Covers.md`](../../Blog/post_Covers.md).

### Soft default

For inline blog photos rendered with `Post_Img` and no forced `aspect` prop, `4:3` is a good default when the content permits it. It reads well in a prose column without dominating the article.

Deviate freely when the content demands a different shape: diagrams, wide comparison graphics, screenshots, tall step-by-step visuals, and source-owned editorial images can use a more suitable ratio.

### Everything else

For non-cover surfaces, inspect the consumer before choosing the aspect:

- **Forced crop slot:** wrapper has `aspect-[X/Y]` and the image uses `h-full w-full object-cover`. Generate for that slot or expect cropping.
- **Source-owned ratio:** image uses `w-full h-auto` and no aspect wrapper. Choose the ratio that serves the content.
- **Hero or panel:** check the actual layout width and composition needs. `Hero_Picture` does not decide the aspect ratio for you.

Always include the chosen aspect ratio in the copy/paste prompt body. The explanation for why that ratio was chosen belongs in the Markdown notes, not in the prompt.

## Example

````md
# Image Prompts — First Dental Visit Blog Cover

## Surface

- Consumer: `src/Content/500_Blogs/de/0001_first-dental-visit-child.mdx`
- Output file: `src/Content/500_Blogs/Post_Assets/0001_first-dental-visit-child/cover.webp`
- Rendered by: `Cover_Hero.astro` and `Blog_Post_Row.astro`
- Intended use: blog cover source, reused as blog index thumbnail
- Aspect ratio: `16:9`
- Target dimensions: 1920 × 1080 (source ≥ 2880 × 1620)
- Rationale: blog covers have a hard `16:9` source contract, and the index row crops the same source into `4:3`.

## Prompt — Calm Waiting Area Objects

```text
16:9 landscape image. A calm dental-practice waiting-area still life with a small wooden toy, a folded child-friendly brochure, and soft morning light on a neutral table. Premium, quiet, editorial, realistic but not identifiable as a real clinic. Crop-safe composition with important objects away from the extreme edges. No people, no animals, no faces, no human or animal figures with visible features. Subject must be non-living or fully faceless.
```
````

## Verification before shipping (writing-phase)

These checks belong to the prompt author, before handing the Markdown to the operator:

- Each prompt body includes the universal safety clause.
- Each prompt body declares the chosen aspect ratio explicitly.
- The Markdown surface notes name the consumer file, output file path, and target dimensions.

Post-generation verification (no faces appear in the rendered output, correct aspect, crop-safe composition) lives in the [conversion leaf](./img_Conversion.md).

## Related

- [Image conversion and placement](./img_Conversion.md) — what happens after the operator returns generated images.
- [`assets.md`](../../Areas/assets.md) — where image files live on disk and how to import them.
- [`image.md`](../../Areas/image.md) — consumer-side image wrappers, `widths` / `sizes` policy, loading attributes.
- [`image_Pipeline.md`](../../Areas/image_Pipeline.md) — pipeline internals: how `<Picture>` expands, breakpoints, Sharp behavior.
- [`../../Blog/post_Covers.md`](../../Blog/post_Covers.md) — blog cover contract.
- [`../../Blog/MDX_Components/post_Img.md`](../../Blog/MDX_Components/post_Img.md) — inline blog images.
- [`../../Scripts/og_Images.md`](../../Scripts/og_Images.md) — OG card generation pipeline.
- `Project_Manag/Docs/Brand/imagery_Brief_*.md` — project-specific subject briefs and visual flavor.
