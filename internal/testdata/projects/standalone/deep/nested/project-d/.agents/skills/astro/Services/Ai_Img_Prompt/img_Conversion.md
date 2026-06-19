# Image Conversion and Placement

What to do when the operator returns generated images. Read this leaf when you find PNG (or JPEG) files in a delivery folder — most commonly `/home/jav/Downloads/AI_Imgs/` or a similar drop-zone — and need to bring them into the project.

For universal content rules and the writing-side workflow, see [`linker_Ai_Img_Prompt.md`](./linker_Ai_Img_Prompt.md) and [`writing_Prompts.md`](./writing_Prompts.md).

## Tool choice — `cwebp`, with ImageMagick as preprocessor

`cwebp` (from libwebp) and ImageMagick (`magick`) are both standard on Linux. They have different niches:

|  | `cwebp` | `magick` |
|---|---|---|
| What it is | Google's reference WebP encoder. Single-purpose. | Swiss-army format converter. WebP is one of hundreds. |
| WebP quality at equal `-q` | Best in the field — typically 5-15% smaller than `magick` at the same visual quality. Full libwebp tuning surface. | Uses libwebp under the hood, but through a wrapper. Slightly larger files. |
| Resize / crop / strip metadata | Minimal — has `-resize W H` only, no real crop control. | Full toolkit. |

**Default tool: `cwebp`.** The typical AI delivery is a PNG already at the right aspect and dimensions — pure format conversion is exactly the niche `cwebp` is built for.

**Fallback: ImageMagick** when the source needs reshaping before encoding (wrong aspect, oversized, colour profile junk). Chain `magick` → `cwebp`.

## Canonical `cwebp` command

```bash
cwebp -q 90 -m 6 -mt -sharp_yuv -metadata none \
  <input.png> -o <output.webp>
```

- `-q 90` — high quality. Source images get re-encoded downstream by Astro/Sharp; we want detail preserved here. 80-90 is the sweet spot; 75 starts to show artefacts on detailed scenes.
- `-m 6` — slowest, best compression method.
- `-mt` — multithread (uses all cores).
- `-sharp_yuv` — better luma/chroma sampling for crisp edges (matters on detailed editorial scenes with thin lines or text).
- `-metadata none` — strip EXIF (privacy + a few KB saved).

## ImageMagick preprocessing recipes (when needed)

### Source is wrong aspect — crop to fit

```bash
magick input.png -resize 2400x1800^ -gravity center -extent 2400x1800 -strip tmp.png \
  && cwebp -q 90 -m 6 -mt -sharp_yuv -metadata none tmp.png -o output.webp \
  && rm tmp.png
```

`-resize 2400x1800^` scales so the SHORT side hits the target, then `-extent` centre-crops the long side. Use when the AI returned a square or a different landscape ratio than the surface needs.

### Source is oversized — downscale to a reasonable source size

```bash
magick input.png -resize 2400x -strip tmp.png \
  && cwebp -q 90 -m 6 -mt -sharp_yuv -metadata none tmp.png -o output.webp \
  && rm tmp.png
```

Anything above ~2400px on the long side is overkill for an editorial source (Astro derives smaller variants from it at build time).

### Source is the right shape — skip ImageMagick entirely

If the PNG aspect and dimensions are already good, just run `cwebp` directly. Adding `magick` in front buys you nothing and risks subtle artefacts from the round-trip.

## Folder discipline

Images live under `src/Assets/Imgs/<page-or-feature>/<descriptive-name>.webp`. **Never in the bare `src/Assets/Imgs/` folder.** A new image either fits into an existing subfolder by topic, or warrants a new subfolder.

Current Partner subfolders (representative):

```text
src/Assets/Imgs/
  career/
  erstes_mal/
  featured/
  praxisphilosophie/
  praxistour/
  services/
  team/
```

Naming inside the subfolder:

- The production asset: `hero.webp` (when the consumer expects a single hero), or a descriptive name (`empfang.webp`, `wartebereich.webp`) when the folder holds multiple images.
- Use the [improved naming convention](../../../coding/languages/improved_Version.md): lowercase first letter for the file, nouns capitalised, underscores between (`empty_Treatment_Room.webp`, not `empty-treatment-room.webp` and not `EmptyTreatmentRoom.webp`).

One image per surface ships. Don't drop extra files in the production folder — `import.meta.glob('*.webp', { eager: true })` patterns used in landing components pull every match and emit unused responsive variants. If an extra image must live on disk (a regeneration, an older version), park it in `Scratch/AI_Imgs/<surface>/`.

## Inferring the destination when the operator named files `0.png`, `1.png`, ...

Operators usually drop files as `0.png`, `1.png`, `2.png` — sequential output from their generation tool. You have to figure out which goes where. Workflow:

1. **Inspect each image visually** — read it with the `Read` tool (multimodal image read).
2. **Cross-reference open prompt briefs** in `Scratch/Img_Prompts/`. Each brief names a consumer file, a surface, and a prompt. Match the visible subject to the brief. If multiple files are delivered for the same surface, ask the operator which to ship — don't pick on their behalf.
3. **Check open issues / recent work** if no prompt brief explains the delivery — `git log -p --since="1 week ago" -- src/Components/ src/Content/` and `Project_Manag/Live_Working/open_Issues.md` often point at the missing-image gap.
4. **Verify against per-project imagery brief** at `Project_Manag/Docs/Brand/imagery_Brief_*.md` — does the visible subject match a documented gap?
5. **When uncertain, ask the operator.** A wrong placement wastes the operator's generation budget; a confirmation question costs nothing.

The output filename is your decision, not the operator's — derive a descriptive name from the visible content + the prompt label, not from `0.png` / `1.png`.

## Aspect ratio sanity check

Before running `cwebp`, verify the source aspect matches what the consumer expects.

```bash
identify -format "%w × %h  (ratio %[fx:w/h])\n" input.png
```

Expected aspects by surface (Partner):

| Consumer | Expected aspect | Notes |
|---|---|---|
| Blog cover | 16:9 exactly | Hard rule. Source ≥ 2880×1620. |
| Inline `Post_Img` | 4:3 default | Source-owned for special content. |
| Landing teaser image (`*_Teaser.astro` with `aspect-[4/3]` wrapper) | 4:3 | Cropped via `object-cover` — match exactly or expect edge loss. |
| Hero (`Hero.astro`, `Hero_Picture`) | Per the layout — check the wrapper | No single answer. |
| Service hero, Featured-service portrait | 4:5 portrait | Per Tier-3 brief. Source ≥ 1800×2250. |
| Praxistour gallery item | 4:3 (default), 16:9 (`size: wide`) | Each station declares its size in `praxistour_Txt.ts`. |

If the source is, say, 1024×1024 (square) but the consumer expects 4:3, run the ImageMagick preprocessing recipe before `cwebp`. If the source is 1920×1080 (16:9) but the consumer expects 4:3, either ask the operator to regenerate or crop-fit with `magick`.

## File-size sanity check after conversion

```bash
stat -c "%n  %s bytes" output.webp
```

Rough budgets for a 1600px-wide source WebP at `-q 90`:

- Empty rooms, simple compositions: 80-200 KB
- Detailed editorial scenes with foliage and texture: 200-400 KB
- Anything > 600 KB at 1600px: investigate. Likely cause is an over-saturated, ultra-detailed AI output. Drop `-q` to 85 and re-encode, or accept the size.

## Source-PNG cleanup

After successful conversion and placement:

1. Verify with `Read` (multimodal) that the WebP renders correctly.
2. Move the source PNGs to a project-local archive or delete them. The drop-zone (`/home/jav/Downloads/AI_Imgs/`) is not a long-term home — it gets reused for every delivery.

```bash
# preferred: archive (per-delivery folder under Scratch/)
mkdir -p Scratch/AI_Imgs/<delivery_Date>_Source_Png/
mv /home/jav/Downloads/AI_Imgs/*.png Scratch/AI_Imgs/<delivery_Date>_Source_Png/

# or, when source isn't worth keeping
rm /home/jav/Downloads/AI_Imgs/*.png
```

Default to archive — keeping the source PNG means a later regeneration can pick up from the original, not a re-encode of the WebP.

## Verification before declaring done

Run these in order:

1. **Policy compliance per image** — re-`Read` the WebP and scan for faces, hands, real-premises pretence, electric green, medical-blue cast, watermarks. If anything slipped through, regenerate, do not ship.
2. **Aspect** — confirm it matches the consumer's wrapper.
3. **File size** — within budget per table above.
4. **Build** — `bun run build` and check that the consumer page generates without errors and the image appears in `dist/_astro/*.avif|webp` (Astro emits derivations).
5. **Visual check on the page** — open `dist/<lang>/<page>/index.html` in a browser, or run `bun run dev` and navigate to the surface. The "image renders correctly in isolation" check is not enough — the layout context matters (crop, dim, surrounding type).

If any step fails, fix before moving on.

## Related

- [`linker_Ai_Img_Prompt.md`](./linker_Ai_Img_Prompt.md) — entry point and universal content rules.
- [`writing_Prompts.md`](./writing_Prompts.md) — the writing-side workflow (creating prompt briefs).
- [`../../Areas/assets.md`](../../Areas/assets.md) — `src/Assets/Imgs/` folder layout and how images get imported into pages.
- [`../../Areas/image.md`](../../Areas/image.md) — consumer-side image wrappers (`<Picture>` vs `<Image>`, `widths` / `sizes` policy).
- [`../../Areas/image_Pipeline.md`](../../Areas/image_Pipeline.md) — pipeline internals: `<Picture>` expansion, breakpoints, Sharp behaviour.
- `Project_Manag/Docs/Brand/imagery_Brief_*.md` — project-specific subject briefs and the missing-image gap list.
