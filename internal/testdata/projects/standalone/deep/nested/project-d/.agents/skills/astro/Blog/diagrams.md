# Diagrams

Static SVG diagrams authored as `.mmd` source files, rendered to `.svg` via a Bun CLI (`bun run mermaid:render`), embedded into MDX through a slot-wrapper component. **No mermaid runtime ships to the browser** — every diagram is a finished SVG by the time the page is built.

This is the author-facing view: how to add a diagram, update one, and what works automatically. For the pipeline internals see [diagrams_Pipeline.md](diagrams_Pipeline.md).

## When to reach for a diagram

| Need | Use |
|---|---|
| Flow / decision tree / sequence / state machine | `<Mermaid_Diagram>` (this doc) |
| Quantitative chart (bar, line, scatter, pie) | `<E_Chart>` — see [charts.md](charts.md) |
| Static illustration / photo / screenshot | `<Post_Img>` — see [Post_Img](MDX_Components/post_Img.md) |

Mermaid covers the structural diagrams that look hand-drawable in plain English (boxes-and-arrows). Anything driven by data goes through ECharts.

## Where source files live

The convention is to co-locate the `.mmd` source and its rendered `.svg` next to the post that uses them:

```
src/Content/<Collection>/Post_Assets/<post-folder>/<diagram>.mmd
src/Content/<Collection>/Post_Assets/<post-folder>/<diagram>.svg    # committed render output
```

Both files are committed. Authoring a new diagram = creating one new `.mmd`, rendering it, and committing the pair. The `.mmd` is the editable source; the `.svg` is the build artifact you embed.

## Authoring a `.mmd` file

A `.mmd` file is mermaid syntax with optional YAML frontmatter for layout knobs:

```mmd
---
flowchart:
  nodeSpacing: 30
  rankSpacing: 50
---
flowchart TD
    A[Start] -->|Path A| B{Decision}
    B -->|Yes| C[Outcome 1]
    B -->|No|  D[Outcome 2]
```

Supported frontmatter keys:

| Key | Effect |
|---|---|
| `flowchart.nodeSpacing` | gap between nodes within a rank |
| `flowchart.rankSpacing` | gap between ranks |
| `flowchart.componentSpacing` | gap between disconnected components |
| `flowchart.padding` | inner padding of node boxes |
| `padding` (top level) | shorthand for `flowchart.padding` |

Unknown keys generate a console warning during render and are silently ignored — the diagram still renders.

For the diagram body, use standard mermaid syntax. **Two real constraints:**

- **Plain text labels only.** No `<b>`, `<i>`, `<a>`, `<br>` inside node labels — mermaid's HTML-label path is disabled (an upstream `max-width: 200px` bug). See [diagrams_Pipeline § known bugs](diagrams_Pipeline.md).
- **Keep labels reasonably short.** Long labels wrap at `wrappingWidth: 400`. A diagram wider than the prose column scrolls horizontally rather than shrinking; there's no auto-scale because shrunk diagrams become illegible.

## Rendering to SVG

```sh
bun run mermaid:render <path>/<diagram>.mmd
# optional second arg: explicit output path
bun run mermaid:render <path>/<diagram>.mmd <path>/<custom-name>.svg
```

What the command does:

1. Reads the `.mmd` source.
2. Strips optional YAML frontmatter; translates flowchart keys into render options.
3. Renders to SVG via `beautiful-mermaid`.
4. Post-processes the SVG so it inherits the site's primary font and theme tokens at view time (font-family swap, theme-token rewrite, geometry recompute against the actual loaded font's metrics).
5. Writes the `.svg` next to the source (or at the explicit output path).
6. Also writes `Scratch/Blog/Mermaid/<name>.preview.html` — a side-by-side light/dark preview you can open in a browser to eyeball the result. `Scratch/` is gitignored end-to-end, so the preview is throwaway; regenerate it any time.

**First-time setup:** the renderer needs the project's primary font cached in `.astro/fonts/`. Run `bun run build` (or `bun run dev`) once before the first `mermaid:render` so the font cache is populated. After that, both commands work in any order.

## Embedding in MDX

In the MDX file:

```mdx
import My_Diagram from "../Post_Assets/<post-folder>/<diagram>.svg";
import Mermaid_Diagram from "@components/Mdx/Import/Mermaid_Diagram.astro";

<Mermaid_Diagram alt="Short description of what the diagram shows.">
  <My_Diagram />
</Mermaid_Diagram>
```

How the wiring works:

- **`<My_Diagram />`** — Astro's SVG-as-component import. Vite resolves the `.svg` import as a component whose JSX form inlines the SVG into the page. No `<img src="…">` wrapper, no separate HTTP request — the SVG markup ends up directly inside the post's HTML, so it picks up the page's CSS context for free.
- **`<Mermaid_Diagram>`** — a `<figure class="diagram-static">` slot wrapper from `src/Components/Mdx/Import/Mermaid_Diagram.astro`. It accepts:
  - `alt` (required) — applied as `aria-label` to an inner `<div role="img">`. Always present, never relies on visible text alone.
  - `caption` (optional) — rendered as `<figcaption>` below the diagram. Use for credit lines or extended explanations the alt-text can't carry.

The component itself is generic — it works for any inlined SVG figure, not just mermaid output. If a project draws diagrams by hand in another tool, the same wrapper can host those SVGs.

## Updating an existing diagram

1. Edit the `.mmd` source.
2. Re-run `bun run mermaid:render <path>.mmd` — overwrites the existing `.svg`.
3. Commit both files.

If you commit only the `.mmd` and forget the `.svg`, the deployed site keeps showing the previous render. Treat the pair as source-of-truth together; the `.mmd` alone is incomplete.

## What happens automatically

Once a diagram is rendered and embedded, several things "just work" without any per-post wiring:

- **Theme adaptation.** Node fills, borders, text colors, edge lines, and subgraph backgrounds all resolve through site CSS tokens (`var(--card)`, `var(--border)`, `var(--foreground)`, `var(--muted-foreground)`, etc.). Toggling `data-theme` on the `<html>` element re-colors the diagram with no rerun and no JS.
- **Font inheritance.** The diagram's text uses `var(--astro_Fnt_Primary, …)` — whatever family the project's primary font role currently resolves to. Swapping the primary family in `src/Data/Common/font_Config.js` propagates to every existing diagram on next page load.
- **Reader Settings insulation.** Diagrams stay pinned to the site's primary font regardless of which option a reader picks in the Reader Settings panel. A reader switching the article body to a different face doesn't affect diagrams.

For the mechanism behind each of these, see [diagrams_Pipeline § browser-time behavior](diagrams_Pipeline.md).

## Common pitfalls

- **`No cached <family> font for weight=… in .astro/fonts/`** during render → `.astro/fonts/` hasn't been seeded. Run `bun run build` once, then re-run the render.
- **Text overflows the node boxes** after a font swap in `font_Config.js` → geometry was measured against the old font. Re-render each `.mmd` file; widths refresh automatically.
- **`config key "<key>" is not supported and will be ignored`** in the CLI output → the YAML frontmatter has a key outside the supported set. Either drop it or shape the diagram a different way.
- **Diagram renders too wide for mobile** → the SVG keeps its natural width on purpose. Either shorten labels, split the diagram, or accept the horizontal scroll behavior on narrow viewports.
- **MDX import path off by one** → the `.svg` import path is relative to the MDX file (`../Post_Assets/...`); the `Mermaid_Diagram` component goes through the project's path alias (`@components/...`). Both have to be correct.

## Add a new diagram — checklist

1. Create the `.mmd` source under `src/Content/<Collection>/Post_Assets/<post-folder>/`.
2. Write the mermaid syntax. Stick to plain text labels.
3. (Optional) Add frontmatter for spacing/padding tuning.
4. Run `bun run mermaid:render <path>/<diagram>.mmd`.
5. Eyeball the `Scratch/Blog/Mermaid/<name>.preview.html` preview.
6. In the MDX file, `import` both the `.svg` and `Mermaid_Diagram`, then wrap the inlined SVG in `<Mermaid_Diagram alt="…">`.
7. Commit the `.mmd` and `.svg` together.
