# Charts

Quantitative charts (bar, line, scatter, pie, etc.) rendered with [ECharts](https://echarts.apache.org/) and embedded into MDX through a single component. `<E_Chart>` is registered globally in `mdx_Components.ts`, so authors use it directly without a per-post import.

This is the author-facing view: how to add a chart, what props it takes, what works automatically, and how to nudge the defaults. For the internal contract (which CSS variables the chart reads, the theme and Reader-Settings hooks, the canvas color-conversion trick, porting to a new project) see [charts_Pipeline.md](charts_Pipeline.md).

## When to reach for a chart

| Need | Use |
|---|---|
| Quantitative data viz (bar, line, scatter, pie) | `<E_Chart>` (this doc) |
| Flow / decision tree / sequence / state machine | `<Mermaid_Diagram>` — see [diagrams.md](diagrams.md) |
| Static illustration / photo / screenshot | `<Post_Img>` — see [Post_Img](MDX_Components/post_Img.md) |

ECharts covers anything that needs axes, legends, tooltips, or programmatic series. Mermaid covers structural boxes-and-arrows.

## Where chart options live

The component takes an `options` prop — a plain ECharts option object. Two authoring patterns:

**Inline** for tiny datasets:

```mdx
<E_Chart
  height={280}
  options={{
    xAxis: { type: "category", data: ["Mon", "Tue", "Wed", "Thu", "Fri"] },
    yAxis: { type: "value" },
    series: [{ type: "bar", data: [12, 18, 9, 24, 15] }],
  }}
  caption="Weekly visit count."
/>
```

**Imported JSON** for anything larger or anything that benefits from being editable next to the post:

```mdx
import options from "../Post_Assets/<post-folder>/<chart>.json";

<E_Chart options={options} caption="2024 weekly visit volume." />
```

The `Post_Assets/` co-location mirrors the diagrams convention. The `.json` source and the `.mdx` that consumes it travel together.

**Constraint: the options object must be JSON-serialisable.** It is passed from the Astro frontmatter to the browser as a `data-echarts-options` attribute and parsed there with `JSON.parse`. Any ECharts feature that requires a function (custom `formatter`, `renderItem`, `tooltip.formatter` as a callback) does **not** work. If a chart needs a formatter, use ECharts' string-template form (`"{a}: {c}"` etc.) or rethink the visualisation.

## Props

| Prop | Type | Default | Purpose |
|---|---|---|---|
| `options` | ECharts option object | required | The chart definition. JSON-serialisable only. |
| `height` | number (px) | `300` | Chart container height. Width is always `100%`. |
| `id` | string | random | DOM id. Provide one only if you need to target the chart from CSS or scripts. |
| `caption` | string | none | Renders as `<figcaption>` below the chart. Same role as the caption on `Post_Img`. |

Markup is a `<figure class="echarts-figure">` containing the chart container and the optional caption. No special accessibility wiring beyond what authors put into `options.title` and `options.aria` (ECharts has native `aria` support).

## What happens automatically

Once a chart is embedded, several things resolve at runtime with no per-post wiring:

- **Theme adaptation.** Axes, tick labels, tooltip background, borders, and the default series colour all read from project theme tokens (`--border`, `--card`, `--muted-foreground`, `--foreground`, plus the `.prose`-scoped `--tw-prose-body`). Toggling `data-theme` on `<html>` re-renders the chart with no manual code.
- **Font inheritance.** Text inside the chart picks up the same font family the article body uses — headings, axis labels, legend, tooltip text, all consistent with the prose around them.
- **Reader-Settings reactivity.** When a reader switches font (sans/serif/mono) or size (s/m/l) in the Reader Settings panel, the chart re-renders with the new family and a proportionally scaled font size. Mono picks up an extra `0.9×` size multiplier so monospaced labels don't overpower the chart, matching the same compensation `reader.js` applies to body text.
- **View transitions.** After an Astro view-transition page swap (`astro:after-swap`), every chart re-initialises so the instance pool stays clean across navigation.

This is the same "free-once-embedded" story as Mermaid diagrams. Once the chart is in the page, runtime hooks keep it in sync with whatever the reader does.

## Default palette — monochromatic

Every series paints in `var(--muted-foreground)` by default. This matches the site's monochromatic-default rule: data viz is body content, not chrome, so it stays quiet unless the data needs colour.

**To introduce colour, set it per-series in your options:**

```js
series: [
  { type: "bar", name: "Visits",   data: [...], color: "#1e3a8a" },
  { type: "bar", name: "Bookings", data: [...], color: "#b45309" },
]
```

Use literal hex strings (or `rgb(...)`). The chart does **not** resolve `var(--*)` at the series level — only the theme-internal tokens go through the canvas converter. If a chart genuinely needs to follow a theme colour across light/dark, that's a dev-level concern; talk to the maintainer or see [charts_Pipeline.md](charts_Pipeline.md).

## External labels and chart margins

Setting `label.position` to `"right"`, `"left"`, `"top"`, or `"bottom"` on a series reserves a fixed 90px margin on that side of the chart and gives the labels `overflow: "break"` so long names wrap inside the reserved space:

```js
series: [{
  type: "bar",
  data: [...],
  label: { show: true, position: "right" },   // 90px auto-reserved on the right
}],
```

The chart area stays stable: long labels don't shrink the plot, they wrap. If you set an explicit `grid.right` (or `.left` / `.top` / `.bottom`) in your options, your value wins — the auto-reservation only fills in sides you haven't set.

For inline labels (`position: "inside"` or any default position), no margin is reserved and labels behave as normal ECharts text.

## Add a chart — checklist

1. Decide where the options live — inline in MDX for a tiny dataset, otherwise a `.json` file under `Post_Assets/<post-folder>/`.
2. Author the options object. JSON-serialisable only; no functions.
3. Embed via `<E_Chart options={...} caption="…" />`. Add `height` if 300px isn't right for this chart.
4. Set `color: "#…"` per series only if the chart genuinely needs colour. The default is monochromatic and that is usually correct.
5. If using external labels, set `position: "right"` / `"left"` / `"top"` / `"bottom"`. The 90px reserved margin appears automatically.
6. Preview locally (`bun run dev`). Toggle theme and Reader Settings to confirm the chart adapts.
7. Commit the `.json` (if used) alongside the `.mdx`.

## Common pitfalls

- **Chart shows nothing / blank container.** Usually the `options` object is invalid (a trailing comma in JSON, a function reference, a `Date` object). The component fails silently on `JSON.parse` of the serialised attribute. Check the browser console — ECharts also logs its own errors there.
- **Series colours all the same.** That's the default monochromatic palette doing its job. Override `color` per series.
- **External-label overflow off the side of the chart.** You used `position: "right"` (or another external side) but also set `grid.right` to something smaller than 90px. Drop the explicit `grid.right` or set it to at least 90px.
- **Font / theme don't update on theme toggle.** The pre-paint theme script (in `Top_Header_Main.astro`) has to be on the page setting `data-theme` on `<html>`. Blog pages always include it; if you're embedding `<E_Chart>` on a non-blog page that doesn't pull in the global header, the MutationObserver has nothing to watch.
- **Font doesn't follow Reader Settings.** The chart reads its font from the first `.prose` element on the page. On a blog post that's `article.prose`; on a non-article page it'll be whichever `.prose` element happens to exist (or fall back to system `sans-serif` if none). For full Reader-Settings reactivity, the chart should live inside the article.
- **A chart works locally but breaks after a build.** Almost always the JSON has something that survives `bun run dev`'s leniency but not the strict serialisation path — a Symbol, a Map, a circular reference. Re-author with plain object/array/string/number/boolean values.

## Things to NOT do

- Don't pass functions in `options` (custom formatters, callbacks). Use ECharts' string templates instead.
- Don't reach for raw colour values that pin to one theme. Either use the monochromatic default or pick a hex that reads acceptably on both light and dark surfaces.
- Don't set explicit `width` on the chart container — width is `100%` by design so the chart respects the prose column.
- Don't import `echarts` yourself in MDX. The component owns the import; pulling it in twice means two bundles.
