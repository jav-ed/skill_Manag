# Reader Settings

A runtime panel that lets blog-post readers swap typography live — **font, font size, line height**. Scoped to article pages only; never site-wide. Settings persist to `localStorage` and re-apply on every load.

Reference implementation: `01_Jav_Web/src/Components/Posts/Reader_Settings/` (the UI) + `01_Jav_Web/src/Scripts/Browser_Client/Blog/reader.js` (the script). The patterns below come from that working code.

## What it controls — three knobs

| Control | Values | LocalStorage key | What it changes |
|---|---|---|---|
| **Size** | `s` / `m` / `l` | `rs-size` | `font-size` on the article element |
| **Spacing** | `compact` / `normal` / `loose` | `rs-line` | `line-height` on the article element |
| **Font** | project-defined (e.g. `sans` / `serif` / `mono`) | `rs-font` | `--font-primary` on the article element |

`m` size and `normal` spacing are the article's stylesheet defaults — selecting them removes the inline override rather than setting an equal value, so the CSS file remains the source of truth when nothing has been picked.

## How the swap actually works

This is the part most people get wrong on first design. There is **no `<style>` injection in `<head>`** and no `:root` override. The script writes inline styles on the article element directly:

```js
const article = document.querySelector('article.prose');

// Font swap — sets the semantic token on the article
article.style.setProperty('--font-primary', 'var(--font-secondary)');   // pick "sans"
// or to reset to stylesheet default:
article.style.removeProperty('--font-primary');

// Size — multiplier against the article's natural prose-class font-size
article.style.fontSize = `${Math.round(naturalSize * 1.2)}px`;          // "l"

// Spacing — direct CSS string
article.style.lineHeight = '2.15';                                      // "loose"
```

The CSS cascade does the rest: anything inside `article.prose` that references `var(--font-primary)` (headings, body prose) picks up the new value. No `!important`, no specificity battles, no rerender. Scoping to the article is the safety net — chrome, header, footer, and sidebar typography are not affected.

**Why this beats `<head>` injection.** Injecting a stylesheet means tracking and removing it on every change; inline styles are read/written in one DOM call and the browser handles cleanup. It also keeps Reader Settings strictly local to the article, so future per-section overrides (e.g. quotes, code blocks) work normally without fighting an injected rule.

## Size + font interaction — per-font scaling

Mono fonts visually appear larger than serif/sans at the same `px` value because every glyph occupies the same advance width. The script applies a per-font size multiplier on top of the user's size choice:

```js
const FONTS = {
  sans:  { css: 'var(--font-secondary)',       sizeScale: 1   },
  serif: { css: null,                          sizeScale: 1   },  // null = stylesheet default
  mono:  { css: 'var(--font-mono), monospace', sizeScale: 0.9 },
};

// Final px = natural × user_size_multiplier × font_size_scale
```

Without `sizeScale`, switching to mono at size `l` would make code-style prose feel oversized. The 0.9× compensates.

When porting to a new project: any font option needs a `sizeScale` value. Default to `1`; only deviate when the visual size of the family genuinely differs at the same px.

## The button protocol — `data-rs-*` attributes

Buttons are declarative. The script wires them by attribute, not by class or ID:

```astro
<button data-rs-size="s"      data-active="false" aria-label="...">A</button>
<button data-rs-line="compact" data-active="false" aria-label="...">...</button>
<button data-rs-font="serif"   data-active="false">Elegant</button>
```

- `data-rs-{size|line|font}="<value>"` — the control + value pair. The script picks these up with `querySelectorAll('[data-rs-size]')`.
- `data-active="true|false"` — managed by the script; styled via Tailwind's `data-[active=true]:` variant. **Not** `aria-pressed` (the buttons are not toggle buttons in the WAI-ARIA sense — they're radio-group-equivalents within each control row).
- `aria-label` — i18n string per button. Always present, never relies on visible text alone (the size buttons render only a glyph, the spacing buttons render only an SVG).

The font option's *label* (Modern / Elegant / Developer in Jav_Web) is project-named and i18n'd. The *key* (sans / serif / mono) is the technical identifier and stays in English in code.

### Control styling

The Reader Settings trigger and all option buttons are compact neutral controls, so they should use `.surface-hover` from [Design interactions](../Design/interactions.md):

```astro
class="... surface-hover [--surface-focus-offset:2px] ..."
```

Keep active state separate from hover state. The script owns `data-active`, and the visual marker should stay as stable text/border emphasis such as `data-[active=true]:text-foreground data-[active=true]:border-foreground/40`. Do not use `bg-muted` hover fills or icon/button scale effects here; the shared foreground-relative hover fill is the neutral affordance. Do not switch to `.surface-selectable` unless the selected state is meant to be a persistent filled surface.

## Panel toggle — open/close behavior

Three triggers close the panel:

1. **Click outside** anywhere not inside `[data-rs-panel]` or its button.
2. **`Escape` key** while focus is anywhere in the document.
3. **Re-click** of the toggle button.

State lives on the panel as `data-open="true|false"`; the button mirrors it via `aria-expanded`. Tailwind animates the open transition with grid rows (`grid-rows-[0fr]` → `data-[open=true]:grid-rows-[1fr]` + `transition-[grid-template-rows]`), which avoids the `height: auto` animation tax.

If a page has multiple `[data-rs]` wrappers (e.g. one for desktop in the TOC, one for mobile in a bottom sheet), all of them close together — opening any one closes the others first.

## Persistence — localStorage keys

Three keys, all under the `rs-` prefix:

| Key | Default if unset |
|---|---|
| `rs-size` | `m` |
| `rs-line` | `normal` |
| `rs-font` | matches the body font's category (see below) |

On every page load, the script reads all three and re-applies them. The size and spacing defaults are stable: `m` and `normal` correspond to the article's stylesheet defaults, so selecting them is a no-op and the inline-style overrides only appear when the user has explicitly chosen non-default.

The font default is project-specific. Under the slot=category convention (see [`bootstrap.md`](./bootstrap.md)), the script's fallback for `rs-font` should equal the slot key whose chain currently resolves to the body font: `sans` when the body is in the Modern slot, `serif` when the body is in the Elegant slot, `mono` when the body is in the Precise slot. Aligning the fallback this way keeps first-load font-family identical to the article's stylesheet default, so the inline override is a no-op until the user explicitly picks something else. Mismatching it (defaulting `rs-font` to `sans` when the body sits in the Elegant slot) force-switches every first-time reader from the body font to a system fallback, which is a regression.

Settings sync across tabs only on focus / explicit re-read, not via the `storage` event — Jav_Web's implementation doesn't subscribe. Add a `window.addEventListener('storage', …)` listener if cross-tab sync is required.

## Accessibility expectations

- **All controls labelled.** Size and spacing buttons render only a glyph or SVG, so `aria-label` is mandatory.
- **The panel is `role="region"`** with an `aria-label` describing what it is ("Reader settings" / translated equivalent).
- **The toggle button uses `aria-expanded`** mirroring `data-open` on the panel.
- **Active state is `data-active`**, not `aria-pressed`. Per-control rows behave like radio groups, but they aren't marked up with `role="radiogroup"` — that would conflict with the visual button styling. If a project's a11y audit pushes for full radio semantics, that's a defensible upgrade.
- **Escape closes** — the keyboard expectation for any disclosure panel.
- **Focus** stays where the user puts it; the panel does not trap focus (it is a transient adjustment, not a modal).

## Where it gets mounted

In Jav_Web:

- **Desktop:** in the TOC header (`Toc_Pc.astro`), so the panel hangs off the right-rail TOC.
- **Mobile:** in the reader-settings sheet that opens from the sticky bottom bar.

For a project porting this, the desktop mount point can be anywhere that's anchored above the article (TOC header, article header, floating action). The mobile mount typically lives in the same sheet system as other "page tools".

## What `--font-primary` overrides actually need

This is the place the four-layer chain pays off (see [setup.md](./setup.md), Variable chain section).

The font-swap inline style writes `--font-primary: var(--font-secondary)` (or another `var(--font-*)`). The browser then resolves the chain — `--font-secondary` → `--fnt_<Body_Font>` → `--astro_Fnt_<Body_Font>` → an actual font name — when applying styles. **CSS-only swap works without the bridge layer**, because the browser does chain resolution.

**JS that reads back the active font name needs the bridge.** If Reader Settings ever needs to extract the resolved font name (to feed it to a chart library or render a preview canvas), `getPropertyValue('--font-primary')` returns the literal string `var(--font-secondary)`, not the resolved font. Call `getPropertyValue('--fnt_<Body_Font>')` instead — the bridge var resolves to an actual font name string, which is what JS needs.

Jav_Web's `reader.js` doesn't currently need this, but the chain is in place so the day it does (for example, when adding a font preview that draws to canvas), no refactor is needed.

## Porting checklist

Bringing Reader Settings to a new project:

1. **Pick the controls.** Size + spacing + font is the established set. Adding a fourth (e.g. column width) means new `data-rs-<name>` + new key + new apply function in the script.
2. **Decide which slots to fill.** The `FONTS` table in `reader.js` is fixed by convention: `sans → var(--font-modern)`, `serif → var(--font-elegant)`, `mono → var(--font-precise)`. It is part of the slot-category contract, not a project knob, so leave it alone. The per-project decision is which physical font feeds each slot, and that wiring lives in `src/Styles/Base/fonts_Base.css`. See [`bootstrap.md`](./bootstrap.md) for the slot-routing rules and the single-bundled-font / two-or-more-font playbooks. The only legitimate reason to edit the `FONTS` table is to tune `sizeScale` for a specific bundled face (the default 0.9 on `mono` compensates monospaced glyphs visually overstating their size).
3. **Build the Panel component** with the `data-rs-*` button protocol and i18n'd `aria-label`s on every control.
4. **Mount the Entry component** in the post layout — desktop and mobile entry points are separate components in Jav_Web (`Entry.astro` + a mobile sheet trigger).
5. **Wire the script.** A `Posts` layout (or the post page itself) imports `reader.js` via `<script>`. The script self-initialises on `DOMContentLoaded`.
6. **Confirm the article element selector.** `reader.js` uses `document.querySelector('article.prose')`. If the project's post template uses a different element/class, update the selector.
7. **Test the panel toggle** — click-outside, ESC, re-click — and verify state persists across reloads.

## Things to NOT do

- Don't inject the font swap into `<head>` (mistake from the earlier version of this doc). Use inline styles on the article.
- Don't use `!important` on the inline override. The cascade handles it.
- Don't scope to `:root` — that bleeds into chrome.
- Don't subscribe to `storage` events for cross-tab sync without a real need; the surprise of typography changing while the reader is mid-paragraph is jarring.
- Don't sync settings to a server / account. This is a local preference, not user data.
