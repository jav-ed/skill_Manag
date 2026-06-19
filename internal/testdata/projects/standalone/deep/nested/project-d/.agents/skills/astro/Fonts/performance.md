# Fonts — Performance & Debugging

What `<Font />` emits in the page HTML, how to verify font loading in DevTools, what ends up in `dist/` after a build, how the cache behaves across rebuilds, and where the performance levers actually live. Open this when: a font isn't loading, the build emits unexpected files, an LCP audit fingers a font as the bottleneck, or a typeface flashes on first paint.

## What `<Font />` emits in the HTML

A single `<Font cssVariable="--astro_Fnt_<Family>" preload={[…]} />` in `<head>` expands to two pieces of HTML at build time:

1. **A `<style>` block** with one `@font-face` declaration per emitted variant, each pointing at a content-hashed file in `dist/_astro/`, plus a `:root { --astro_Fnt_<Family>: <FamilyName>, system-ui, sans-serif; }` rule that wires the cssVariable to the font family name.
2. **A `<link rel="preload">`** for every entry in `preload={[…]}` — `as="font"`, `type="font/woff2"`, `crossorigin`, pointing at the same content-hashed URL the `@font-face` references.

Variants **not** in `preload={[…]}` are still in the `@font-face` block but have no `<link rel="preload">`. The browser fetches those lazily on first use after CSS resolves them, which is the right behavior for non-LCP weights.

If a project has zero preload entries (no `preload={[…]}` prop on `<Font />`), the variant only fetches when CSS asks for it — usually fine for tertiary weights, never for body or H1.

## DevTools verification

The fastest end-to-end check is **DevTools → Network → filter `Font`** on a fresh load (clear cache or `Ctrl+Shift+R`):

- The preloaded variants should appear *very* early in the waterfall, typically before any image or non-critical script.
- Their `Priority` column should read `Highest` (the result of the `<link rel=preload>` hint).
- Non-preloaded variants either don't appear (not used yet) or appear later (used by a style applied below the fold).

If a preloaded variant doesn't show `Highest` priority, the `<link rel=preload>` is either missing the `as="font"` attribute or the `crossorigin` flag — Astro emits both correctly, so a missing one usually means a hand-written `<link>` is shadowing it.

**Coverage panel** (`DevTools → ... → More tools → Coverage` → reload) shows which font variants are *used* on the current page vs. just downloaded. A preloaded variant with 0% coverage means it was preloaded but never applied — either remove it from `preload` or fix the CSS that should be referencing it.

**Performance timeline** (record a load → look at the LCP marker) tells you whether the font finished loading before the LCP element painted. If LCP is text and the font hasn't loaded, the page either renders FOIT (invisible text) or FOUT (system fallback first) depending on `font-display`. `font-display: swap` is the Astro default — visitors see fallback text immediately, then swap when the font arrives.

**Application → Local Storage** — for Reader Settings projects only, the `rs-*` keys should appear after the user touches any control. If they don't, the persistence wiring is broken.

## `dist/_astro/` build output

After `bun run build`:

```
dist/_astro/
├── public-sans-latin-400-normal.<hash>.woff2
├── public-sans-latin-700-normal.<hash>.woff2
└── …
```

Filename pattern: `<family-slug>-<subset>-<weight>-<style>.<contentHash>.woff2`. The content hash is stable as long as the underlying file is — renaming the font, switching weights, or changing the Fontsource version invalidates it.

**Typical sizes** (Fontsource, Latin-only subset, one weight, woff2):

| Family class | Per-weight size |
|---|---|
| Modern variable (Public Sans, Inter, Geist) | 25–40 KB |
| Serif (Newsreader, Spectral, Lora) | 35–60 KB |
| Display (Recoleta, Cooper, custom)  | 50–80 KB |

A site with two families × four weights × Latin-only is typically under 250 KB total emitted font weight. CJK subsets are an order of magnitude larger (~1–4 MB per weight) — don't bundle them lightly.

The `_astro/` folder is shipped as-is; no further bundling/compression is applied at Astro's level. CDN-level brotli/gzip applies on transit and brings the wire size down further.

## Cache behavior across builds

Astro caches font transforms under `.astro/` (in your repo) and `node_modules/.astro/` (Astro internals). Unchanged sources reuse the cached output — that's why incremental builds are fast.

Invalidation triggers (forces a re-emit):

- The family's package version in `package.json` changes (`@fontsource/public-sans` 5.0.0 → 5.1.0).
- The `weights: [...]` or `styles: [...]` array in `astro.config.mjs` changes.
- For local fonts, the `.woff2` file on disk changes (different mtime or content hash).
- `.astro/` directory is deleted.

**Forcing a clean transform run.** When debugging a font that "should be updated but isn't":

```bash
rm -rf .astro/ dist/
bun run build
```

The full re-emit takes seconds for Fontsource (no re-download needed in Astro 6 — packages are read from `node_modules`), longer for the image pipeline on the same build but that's separate.

The browser cache is content-hashed independently, so changing a font (and emitting a new hash) bypasses the visitor's cache without any manual cache-busting needed.

## Performance lever ranking

When a font is slow, the levers in order of impact:

1. **Preload (the LCP-critical variants).** The single highest-impact change. Without preload, fonts fetch *after* CSS resolves — typically 50–150 ms after page load on a fast connection, much worse on slow ones. With preload, the browser starts fetching during HTML parse. Set `preload={[…]}` to exactly the variants used above the fold.
2. **`font-display: swap`.** Astro's default. Means the browser renders text in the fallback face immediately and swaps in the real face when it arrives — visitors see content faster, at the cost of a visible flash. The alternative (`font-display: block`) hides text until the font arrives (FOIT) — almost always worse UX. Don't change the default unless a specific design genuinely needs FOIT.
3. **Subsetting.** Fontsource ships per-subset packages (`@fontsource/<family>` is Latin only; `@fontsource/<family>-cyrillic`, `-greek`, `-vietnamese`, etc. are separate). Astro doesn't auto-pick the right subset — the registration imports the package literally, so importing the Latin-only package keeps the build small. For multi-script content, register each subset explicitly.
4. **Weight count.** Each weight is a separate file. Going from `weights: [400, 500, 600, 700]` to `weights: [400, 700]` halves the font payload — at the cost of `font-medium` and `font-semibold` rendering as system fallback or fake-bold-faked-medium. Worth it on projects with strict performance budgets; not worth it on most.
5. **Format.** `woff2` is the universal default and what Astro emits. There is no realistic alternative — `woff1` is twice the size, ttf/otf are even larger and don't compress well.
6. **CDN vs self-host.** Astro self-hosts by design (privacy + GDPR), so "switching to a CDN for speed" is not a lever available in this org. Don't reach for it.

The first two levers are where 90% of the actual perf wins live. Subsetting and weight count are budget tools, not first-line optimisations.

## Common debugging scenarios

- **A font isn't loading at all.** Check `<Font cssVariable="..." />` is in the root Layout's `<head>` and the cssVariable name matches the `astro.config.mjs` registration exactly. Mismatched names silently produce no output.
- **Preloaded but never used.** Coverage panel will show 0%. Either a stylesheet references the wrong `cssVariable` / `--font-*` name, or the page doesn't actually use that weight. Remove the preload entry or fix the reference.
- **FOUT flash on every load.** `font-display: swap` is working as designed. To reduce flash duration, ensure the LCP-critical variants are preloaded (lever 1 above). Eliminating flash entirely requires `font-display: optional` or `block`, both of which have worse trade-offs.
- **Build emits 30+ font files.** Too many weights × subsets registered. Audit `astro.config.mjs` against the weight-usage grep in [setup.md](./setup.md), Weights gotcha section.
- **The variant changed but the browser shows the old one.** Browser cache holding the old `<hash>.woff2`. Content-hashed filenames mean Astro will emit a new file on weight/version change; force-refresh in DevTools (Network → Disable cache + reload). If the rebuild itself reused cached output, `rm -rf .astro/ dist/` + rebuild.
- **OG image renders the wrong font.** OG generation is a separate pipeline that reads from `node_modules/@fontsource/...` directly — it does not go through `<Font />`. Different fix path; see the OG_Images doc in the source repo.
