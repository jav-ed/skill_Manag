# Diagrams pipeline — deep dive

How the `.mmd` → committed `.svg` → page render chain works internally. Use this when modifying the pipeline, debugging an unexpected SVG output, or adapting the pattern to a new project. For the author-facing flow (write a diagram, embed it, update it) see [diagrams.md](diagrams.md).

## Architecture overview

Diagrams are **fully resolved at CLI time**. The browser never runs mermaid; it just paints a finished SVG that happens to be styled by CSS variables.

Three runtimes are involved:

| Runtime | Lives in | Role |
|---|---|---|
| Bun CLI | `Code/Mermaid/` + `Code/shared/` | Reads `.mmd` source → produces committed `.svg` |
| Astro build | `src/Components/Mdx/Import/Mermaid_Diagram.astro` | At build time, inlines the committed `.svg` into MDX-generated pages |
| Browser | The user's machine | Resolves CSS variables on the inlined SVG at paint time |

The runtime mermaid library (`mermaid`, `@mermaid-js/...`) is **not** a dependency. The CLI uses `beautiful-mermaid` instead, which produces SVGs that are more amenable to post-processing.

## File map

```
package.json
└── scripts."mermaid:render" → "bun Code/Mermaid/mermaid_Render.js"

Code/Mermaid/
├── mermaid_Render.js         # CLI entry point. Orchestration only.
├── mermaid_Pre_Process.js    # Parses optional YAML frontmatter; translates flowchart keys to beautiful-mermaid options.
└── mermaid_Post_Process.js   # Rewrites the raw SVG: font-family swap, theme tokens, geometry recompute, edge-label padding.

Code/shared/
├── fontMetrics.js            # fontkit-based text-width measurement; reads .woff2 files from .astro/fonts/.
└── yaml_Mini.js              # Minimal YAML parser used by mermaid_Pre_Process (no dep on a full YAML library).

src/Data/Common/font_Config.js
                              # The font-role registry. mermaid_Post_Process and fontMetrics both import primary_Font from here.

src/Components/Mdx/Import/Mermaid_Diagram.astro
                              # The <figure class="diagram-static"> slot wrapper used by MDX authors.

src/Styles/Pages/Posts/prose_Style.css
                              # The .prose .diagram-static block — overflow rules + the Reader-Settings insulation that
                              # pins .diagram-static descendants to var(--font-secondary).
```

## Stage 1 — CLI render (`mermaid_Render.js`)

Orchestration only; the heavy lifting is delegated:

1. Read the `.mmd` file from `process.argv[2]`. Resolve output path from `process.argv[3]` or default to the source path with `.svg` extension.
2. Call `preProcess(content)` (from `mermaid_Pre_Process.js`) — splits YAML frontmatter from body, translates flowchart config keys, warns on unknown keys.
3. Lazy-import `beautiful-mermaid` and call `renderMermaid(body, { ...DARK_TOKENS, transparent: true, ...layout })`. This produces a raw SVG with mermaid's internal styling — **including a hardcoded `font-family: 'Inter'`**.
4. Call `postProcess(svg)` (from `mermaid_Post_Process.js`) — rewrites the raw SVG into the project's idiom. Detailed in the next section.
5. Write the post-processed SVG to disk.
6. Generate `Scratch/Blog/Mermaid/<name>.preview.html` — a two-panel light/dark preview. `Scratch/` is gitignored end-to-end so the preview is throwaway.

`renderMermaid` is called **without a `fontFamily` option** because the font-family is rewritten in step 4. Whatever family beautiful-mermaid used internally is discarded.

## Stage 2 — post-processing (`mermaid_Post_Process.js`)

`postProcess(svg)` runs seven transformations in order. Each is a regex pass over the SVG string.

### 1. Strip inline `style=""` from the root `<svg>`

beautiful-mermaid sometimes emits a root `style="…"` attribute that conflicts with the project's CSS variables. Remove it; `prose_Style.css` rules drive layout instead.

### 2. Set natural width on the SVG

Read the `viewBox` to derive the SVG's intrinsic pixel width, then replace the auto-emitted `width="…"` / `height="…"` with `width="<natural>"` and `data-natural-width="…"`. If the viewBox can't be parsed, fall back to `width="100%"`.

Why natural width: diagram dimensions are baked at render time. In the browser, the prose container scrolls horizontally rather than shrinking the SVG, so labels stay legible at small viewports.

The `data-natural-width` attribute is also read by browser-side code (e.g. `src/Scripts/Browser_Client/Blog/reader.js`) to re-apply the width on load when SVG import strips inline width.

### 3. Replace the theme `<style>` block

beautiful-mermaid emits its own `svg { --bg: …; --fg: …; … }` style block with concrete hex colors. Replace it with `var(--color-bg)`, `var(--color-card)`, `var(--color-line)`, `var(--color-muted)`, etc. — references to the project's CSS-token chain.

When the page's `data-theme` toggles, these variables resolve to the dark or light theme automatically. No rerender.

### 4. Recompute edge-label backgrounds

For each `<g class="edge-label">` carrying `data-label="…"`:
- Find the inner `<rect>`'s `width` and `x`
- Add `EDGE_LABEL_PAD = 12` of padding on both sides
- Recenter the rect at the original midpoint

beautiful-mermaid sizes edge-label rects flush to the text. The pad gives labels visible breathing room.

### 5. Recompute node widths via `measureText`

For each `<g class="node">` carrying `data-label="…"`:
- Decode the label string
- Call `measureText(label, fontSize=14, weight=500, "sans")` from `fontMetrics.js`
- Compute `newW = measured + NODE_PAD * 2` (NODE_PAD = 14)
- Replace the rect's `x="…"` and `width="…"` to recenter at the original midpoint

Why recompute: beautiful-mermaid sizes its rects against Inter's glyph widths. The project's actual primary font has different metrics. Without this step, text either overflows the boxes or floats inside oversized boxes.

### 6. Recompute subgraph headers

Same pattern as nodes for `<g class="subgraph">` with `data-label="…"`, but using `FONT_METRICS.groupHeader: { size: 13, weight: 600 }` and `SUBGRAPH_LABEL_PAD = 16`. The first `<rect>` width is widened to fit; the `<text>` element is recentered with `text-anchor="middle"`.

If the existing rect is already wide enough, the pass is a no-op.

### 7. Rewrite `font-family`

```js
svg = svg.replace(
  /font-family: 'Inter'[^;]*;/g,
  `font-family: var(${primary_Font.css_Variable}, var(--font-sans, sans-serif));`,
);
```

This is what makes the SVG follow the project's font slot. Whatever family `font_Config.js` defines for the `primary` role at view time, the SVG picks up via the CSS-var chain.

### 8. Strip beautiful-mermaid's `@import url(...)`

beautiful-mermaid sometimes emits an `@import url('https://fonts.googleapis.com/...')` for Inter. Stripped, since the project bundles its own font via Astro Fonts API and shouldn't fetch from third-party CDNs.

## Stage 3 — font measurement (`fontMetrics.js`)

`measureText(text, fontSize, fontWeight, fontKey = "sans")` returns the rendered width of a string in the loaded primary font.

Internals:

1. **`bucket_Weight(fontWeight)`** maps the requested weight to one of `400 / 500 / 600 / 700`. The post-processor only ever asks for 500 (node labels) and 600 (subgraph headers); the bucket keeps the cache small.
2. **`find_Astro_Cached_Font_File(primary_Font, bucket)`** globs `.astro/fonts/` for files matching the role's `astro_Css_Variable` prefix at the requested weight, style, and subset:
   - **Exact match** for static-weight fonts: `<prefix>-<weight>-<style>-<subset>-<hash>.woff2`
   - **Variable-range match** for variable fonts: `<prefix>-<min>-<max>-<style>-<subset>-<hash>.woff2`
   - `uses_Variable_Font_Range(font_Config)` decides which pattern to prefer, by checking whether any entry in `font_Config.weights` is a space-separated range string.
3. **`font_File_Matches_Family(file, family_Name)`** opens each candidate with fontkit and checks that `font.familyName` (or `fullName`) starts with the configured family. This guards against picking up an orphan file from a previous font that's still in the Astro cache.
4. The matched font is opened with `fontkit.openSync` and cached per `(fontKey, bucket)` so repeated measurements don't re-parse the binary.
5. `measureText` calls `font.layout(text)`, sums `xAdvance` across the glyph run, and converts from font units to pixels: `(total / font.unitsPerEm) * fontSize`.

The cache is per-process; each CLI invocation starts fresh, so a font swap in `font_Config.js` automatically takes effect on the next render without any cache-busting step.

## Browser-time behavior

Once a `.svg` is committed and inlined into a built page, there is no JS involved in rendering it.

The committed SVG contains:

- **Geometry:** hardcoded `width="…"`, `x="…"`, `height="…"` from the post-processor's measurements
- **Font:** `font-family: var(--astro_Fnt_Primary, var(--font-sans, sans-serif));`
- **Colors:** `<style>` block referencing site CSS tokens (`var(--color-bg)`, `var(--card)`, `var(--border)`, etc.)

At paint time the browser:

1. Resolves `--astro_Fnt_Primary` from Astro's root `<style>` block — Astro Fonts API defines this variable based on `font_Config.js`'s primary role.
2. Resolves `var(--card)`, `var(--border)`, etc. from the page's theme variables (controlled by `data-theme` on `<html>`).
3. Lays out the SVG using the resolved font family at the hardcoded geometry.

**Reader Settings can't reach the diagram.** Two layers of insulation:

- The SVG references `--astro_Fnt_Primary` (the concrete-loaded-font variable, defined at `:root` level). Reader Settings overrides `--font-primary` (the semantic token, scoped to `article.prose`). They're different variables — the article-level override never reaches the variable the SVG actually reads.
- Belt: `prose_Style.css` declares `.prose .diagram-static { --font-primary: var(--font-secondary); }`, neutralising any `--font-primary` override on diagrams specifically. Even if a future change made the SVG go through `--font-primary`, this rule would catch it.

**Theme switch is automatic.** Toggling `data-theme` re-resolves the `var(--color-*)` references in the SVG's `<style>` block. No JS, no rerender.

## The build-first dependency

`fontMetrics.js` reads `.astro/fonts/`, which is populated by Astro's Fonts API during `bun run build` or `bun run dev`. The strict order for a fresh checkout is:

1. Configure (or change) the family in `src/Data/Common/font_Config.js`.
2. **Run `bun run build` (or `bun run dev`) once** — this downloads the `.woff2` files into `.astro/fonts/`.
3. **Then** `bun run mermaid:render <path>.mmd` — fontMetrics finds the file and measures with it.

A fresh clone with no prior build will fail step 3 with `No cached <family> font for weight=… in .astro/fonts/`. The error is loud, not silent.

After the first build, the dependency is one-directional and idempotent: subsequent renders work regardless of whether you rebuild between them.

## Font-swap maintenance

When `font_Config.js`'s primary family changes:

- **Font face follows automatically.** Every committed SVG renders in the new family on next page load. The `var(--astro_Fnt_Primary, …)` reference does the work — no SVG edit needed.
- **Geometry does NOT follow.** Node-box widths, label x-positions, subgraph header widths, and edge-label backgrounds were measured against the old font's `.woff2`. If the new font has noticeably different glyph widths (sans ↔ serif, condensed ↔ wide), text may overflow boxes or float inside them.

Rule of thumb:

- Same-style swap (one sans for another sans of similar metrics) → geometry usually still fits, no re-render needed.
- Cross-style swap (sans ↔ serif, narrow ↔ wide) → re-run `bun run mermaid:render` on every `.mmd` to refresh geometry.

The committed SVGs are not auto-regenerated on build. Re-rendering is a manual maintenance step. A CI-side guard (compare `.mmd` mtime to `.svg` mtime, or hash-compare) is a defensible addition if the project ships diagrams frequently.

## Known mermaid bugs we work around

**htmlLabels `max-width: 200px`** (mermaid-js/mermaid #3525, #7354, #6424, #6437): mermaid's HTML-label path wraps node text in `<foreignObject>` + HTML `<div>` with a hardcoded `max-width: 200px`, truncating wider labels. No upstream fix as of mermaid v11.14.

The pipeline uses `htmlLabels: false` everywhere — labels render as native SVG `<text>`. Trade-off: no inline formatting in node labels (`<b>`, `<i>`, `<a>`, `<br>` don't work). Plain text labels only.

## Historical note — why not a runtime mermaid library

An earlier shape of this pipeline used a runtime two-part pattern: a `remark_Mermaid` build-time plugin that converted ```` ```mermaid ```` fences to `<pre class="mermaid">`, plus a client-side `Mermaid_Init.astro` component that lazy-loaded mermaid and rendered the blocks to SVG in the browser.

That pattern was removed. The static CLI is preferred because:

- **No per-page JS cost.** The mermaid runtime library is ~1.5 MB; a static SVG is typically a few KB.
- **Render-once correctness.** Runtime mermaid measures text in JS before drawing the SVG. That measurement runs before CSS can restyle the page, so the font often hasn't loaded yet → node boxes sized for the fallback font → real font renders wider → labels clip. The static CLI measures with fontkit against the actual `.woff2`, no race.
- **No CSS-vs-SVG-attribute fight.** Runtime mermaid wants to set its own theme via JS at init time; the static pipeline writes CSS-var references straight into the SVG and lets the cascade handle theming.
- **Same authoring tooling.** The author still writes mermaid syntax in a `.mmd` file; only the render step moves out of the browser.

If a future project genuinely needs runtime mermaid (e.g. user-editable diagrams in a CMS, or live previews while editing), the runtime pattern would need to be reintroduced as a parallel option — not as a replacement.

## Adapt the pattern to a new project — checklist

1. Add `beautiful-mermaid` and `fontkit` to `devDependencies`; add a `mermaid:render` script in `package.json` pointing at `Code/Mermaid/mermaid_Render.js`.
2. Copy `Code/Mermaid/{mermaid_Render,Pre_Process,Post_Process}.js` and `Code/shared/{fontMetrics,yaml_Mini}.js` into the new project.
3. Ensure `src/Data/Common/font_Config.js` exists with a `primary` role. The post-processor and fontMetrics both import `primary_Font` from there.
4. Add `src/Components/Mdx/Import/Mermaid_Diagram.astro` — a `<figure class="diagram-static">` slot wrapper with `alt` (required) and `caption` (optional) props.
5. Add the `.prose .diagram-static` layout + font-pin rules in the project's prose stylesheet (`overflow-x: auto;` on the figure, `display: block; margin: 0 auto;` on the inner `svg`, and `--font-primary: var(--font-secondary);` to insulate from Reader Settings if the project ships one).
6. Add `Scratch/` to `.gitignore` (the previews land there).
7. Run `bun run build` once to seed `.astro/fonts/`. Render one test `.mmd`. Eyeball the preview HTML.
