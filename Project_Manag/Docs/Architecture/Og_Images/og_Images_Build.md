# OG Image Build Sequence

Two steps, always in this order.

## Step 1 — Astro build

```bash
./Scripts/Steps/build.sh
```

Runs `bun run build`. As a side effect, the manifest integration hook fires and writes `Cache/Og_Gen/og_Manifest.json`.

## Step 2 — OG image generation

```bash
./Scripts/Steps/og_images.sh
```

`cd`s into the external generator (`02_Workfl_Comp/Og_Image`) and runs:

```bash
bun src/main.js --project_Root <project_root>
```

Reads `Cache/Og_Gen/og_Manifest.json`, renders one WebP per task, writes to `dist/Og_Gen/`.

## Scripts structure

```
Scripts/
  Config/env.sh      — CI_PROJECT_DIR (git root), CI_OG_IMAGE_DIR (generator path)
  Lib/colors.sh      — terminal output helpers
  Steps/build.sh
  Steps/og_images.sh
```

## Notes

- `dist/Og_Gen/` is recreated on every build — Step 2 must always run after Step 1.
- If no pages have `seo.og.image` in frontmatter, the manifest has zero tasks and the generator exits cleanly.
- The generator path in `Scripts/Config/env.sh` is an absolute path — update it if the generator is moved.
