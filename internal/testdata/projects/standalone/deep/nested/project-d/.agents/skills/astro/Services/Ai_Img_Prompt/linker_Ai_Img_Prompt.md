# AI Image Prompt

The AI-image workflow has two halves, and an agent typically owns one half per turn:

1. **Writing prompt briefs** — translate a surface gap (a missing image at a known consumer) into a copy/paste-ready prompt Markdown, with subject framing, hard-rule clauses, and target aspect ratio. The operator then runs the prompt through their image-generation tool.
2. **Converting delivered images** — once the operator hands back PNGs (often as numbered files like `0.png`, `1.png`), inspect each, infer which surface it belongs to from visible content + open prompt briefs, convert to WebP at high quality, and place it under `src/Assets/Imgs/<page>/<name>.webp`.

Both halves share the same universal content rules, listed once below so each leaf can stay focused on its workflow.

## Universal content rules (apply in both halves)

These come from the canonical Zetun policy at `03_Zetun_Page/Project_Manag/Docs/Offering/ai_Images.md`. They are not negotiable in either prompt-writing or post-generation verification.

### Never

1. **No faces of any living creature**, in any rendering style — foreground, background, blur, silhouette, all rejected.
2. **No photo-like real-people imagery.** No fake patients, staff, customers, treatments, or before/after composites.
3. **No fake actual-premises imagery.** Do not pretend to depict the client's real waiting room, storefront, treatment chair, or staff.
4. **No hands or fingers.** Even faceless. Listed in the negative-prompt set.

### Allowed when faceless

Plants, objects, architecture, empty rooms, abstract patterns, textures, silhouettes, shadows, back-turned figures, figures cropped above the shoulders, geometric dividers, and symbolic non-living imagery.

The body underneath a faceless figure can be photorealistic, illustrative, or abstract. The visible face is the line.

### Per-project flavor

The universal rules above are floor-only. Per-project flavor — aesthetic register, palette, light direction, industry-specific regulatory rules like German HWG — lives in each project's `Project_Manag/Docs/Brand/imagery_Brief_*.md`. Read that brief before writing prompts or accepting delivered images for that project.

## Outputs at a glance

| Phase | Output location | Purpose |
|---|---|---|
| Writing prompts | `Scratch/Img_Prompts/<surface>.md` | Markdown brief the operator copies into their generation tool. |
| Accepting delivered images | `src/Assets/Imgs/<page>/<name>.webp` | The final asset, consumed by an Astro page/component. |

## Leaves

- [Writing prompts](./writing_Prompts.md): how to compose a copy/paste-ready prompt-brief Markdown. Surface-notes block, aspect-ratio judgement, prompt-body framing, negative-prompt set, and the standard recommended structure. Read when the operator asks for an image and you need to brief them.
- [Image conversion and placement](./img_Conversion.md): what to do once the operator returns PNGs. Tool choice (`cwebp` as default, ImageMagick as preprocessor when the source needs reshape), the canonical `cwebp` command, folder discipline, how to infer which delivered file maps to which surface from visible content + open prompt briefs, post-conversion sanity checks, build verification.

## When to skip this skill

- The image is a small UI icon — use Lucide (see `.agents/skills/astro/Areas/icons.md`).
- The image is a real photograph from the client — no AI-generation policy applies; just convert to WebP per the conversion leaf and skip the policy verification.
- The image is purely decorative (a CSS pattern, a gradient, a generated SVG shape) — out of scope.
