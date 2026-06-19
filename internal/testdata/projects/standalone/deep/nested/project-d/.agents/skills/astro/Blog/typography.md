# Typography

How blog prose is styled. Base layer: **`@tailwindcss/typography`** providing the `.prose` class. Project overrides live in `src/Styles/2_Pages/3_Posts/prose_Style.css`, wired into the page entry-chain via `posts_Main.css`.

The plugin gives sensible defaults for every prose element (paragraphs, headings, lists, blockquotes, code, tables) and exposes a `--tw-prose-*` CSS variable surface for per-token overrides. The org convention: keep the plugin's *structure* (spacing, list bullets, table treatment) and override only the *theme tokens* so prose inherits the project's design-system colours and the project's heading scale.

## Wire-up

### 1. Install the plugin

```sh
bun add -d @tailwindcss/typography
```

### 2. Load the plugin in base CSS

```css
/* src/Styles/0_Base/base_Main.css */
@import "tailwindcss";
@plugin "@tailwindcss/typography";
```

`@plugin` is the Tailwind v4 syntax. The plugin registers `.prose` and the responsive size variants (`prose-base`, `prose-lg`, `prose-xl`).

### 3. Apply `.prose` to the article on the post layout

```astro
<article
  class="
    prose
    sm:prose-base
    md:prose-lg
    xl:prose-xl

    sm:max-w-[53ch]
    lg:max-w-2xl
    xl:max-w-2xl
    2xl:max-w-3xl

    subpixel-antialiased
    text-pretty
    break-words
  "
  lang={lang}
>
  <slot name="main_text" />
</article>
```

Two things matter here:

- **`.prose` once, size modifier per breakpoint.** `sm:prose-base md:prose-lg xl:prose-xl` is the org's responsive size ladder. Smaller screens get tighter type for the limited width; large screens get more generous sizing.
- **`max-w-[…ch]`** caps the reading line length per breakpoint. Around 60–80 characters per line is the readability sweet spot; a `2xl:max-w-3xl` cap (~72ch) is comfortable on widescreen monitors.

### 4. Wire the override stylesheet via the page entry-chain

```css
/* src/Styles/2_Pages/3_Posts/posts_Main.css */
@import "../../0_Base/base_Main";

@import "./reg_Blog";          /* TOC scrollbar, bismillah, non-prose blog elements */
@import "./video_Tabs.css";    /* video tabs */
@import "./prose_Style.css";   /* THE prose override */
@import "./post_Progress.css"; /* reading progress bar */
@import "./prose_Reveal.css";  /* scroll-driven reveal */
```

The post layout imports `posts_Main.css`:

```astro
---
import "@styles/2_Pages/3_Posts/posts_Main.css";
---
```

Per the [Styles folder structure](../Styles/folder_Structure.md), each page concept has one entry CSS file that pulls everything it needs. The post layout's entry is `posts_Main.css`.

## The override pattern

Tailwind Typography exposes everything as `--tw-prose-*` CSS variables. Overriding is one selector + a handful of variable assignments:

```css
/* src/Styles/2_Pages/3_Posts/prose_Style.css */

[data-theme] {
  & .prose.prose {

    /* Body size — clamp to a comfortable range. 16px floor on small screens,
       18px ceiling on large. Matches Vercel/Stripe blog benchmarks. */
    font-size:   clamp(1rem, 2.5vw, 1.125rem);
    line-height: 1.75;
    font-family: var(--font-primary);

    /* Wire prose colours to project theme tokens. Body at 85% for comfortable
       reading; headings/links/bold/code at full foreground to stand out. */
    --tw-prose-body:     color-mix(in oklch, var(--foreground) 85%, transparent);
    --tw-prose-headings: var(--foreground);
    --tw-prose-links:    var(--foreground);
    --tw-prose-bold:     var(--foreground);
    --tw-prose-code:     var(--foreground);
    --tw-prose-quotes:   color-mix(in oklch, var(--foreground) 85%, transparent);

    /* Bold — weight 500 (medium), same as h2–h4.
       Differentiation from body comes from COLOR (full foreground vs body's 85%),
       not from heavy weight that would overpower headings. */
    & strong { font-weight: 500; }

    /* Inline code — single-backtick spans. Fenced blocks are handled by
       Expressive Code; this rule only affects `<code>` without `<pre>` parent. */
    & :not(pre) > code {
      background-color: var(--muted);
      border-radius:    var(--radius-xs);
      padding:          0.15em 0.35em;
      font-size:        0.65em;     /* Maple Mono runs large; reduce within prose */
      font-weight:      normal;     /* override the plugin's default 600 */
    }
  }
}
```

Five conventions packed into this snippet:

1. **`[data-theme] & .prose.prose`** — the `[data-theme]` ancestor scopes the rule to the project's theme system (see [Styles tokens](../Styles/linker_Styles.md)). The doubled `.prose.prose` raises specificity above Tailwind's defaults without using `!important`.
2. **`color-mix(in oklch, var(--foreground) 85%, transparent)`** — opacity-against-the-current-theme via colour mixing. Works on both light and dark themes; no per-theme overrides needed.
3. **`var(--font-primary)`** for the body font — the project's font registry (see [`../Fonts/setup.md`](../Fonts/setup.md)) exposes role-named variables; prose reads them, not Tailwind's defaults.
4. **`var(--muted)`, `var(--radius-xs)`** — design tokens from the project's token catalogue. Never hardcode colours or radii in `prose_Style.css`.
5. **Inline-code carve-out.** The plugin styles all `<code>` with weight 600 and quote marks. Override both for single-backtick spans; fenced blocks go through Expressive Code (see [Markdown pipeline](markdown_Pipeline.md)) which renders its own `<pre>` chrome.

## Heading scale

Tailwind Typography's heading sizes are generic. Override per project to match the editorial register. The org's reference scale (from Jav_Web's `prose_Style.css`):

| Level | Size | Line-height | Weight | Letter-spacing | Notes |
|---|---|---|---|---|---|
| h1 | 2.25rem (36px) | 1.11 | 600 | -0.02em | Tight tracking, semi-bold — the title-card scale, rarely used in body |
| h2 | 1.5rem (24px) | 1.33 | 500 | -0.01em | Border-bottom carries the visual hierarchy |
| h3 | 1.25rem (20px) | 1.6  | 500 | 0.02em  | Open tracking signals subhead |
| h4 | 1rem (16px)    | 1.5  | 500 | -0.01em | Same size as body — weight + spacing differentiates |

Apply via the same `[data-theme] & .prose.prose` scope:

```css
[data-theme] {
  & .prose.prose {
    & h1 {
      font-size:   2.25rem;
      line-height: 1.11;
      font-weight: 600;
      letter-spacing: -0.02em;
    }
    & h2 {
      font-size:   1.5rem;
      line-height: 1.33;
      font-weight: 500;
      letter-spacing: -0.01em;
    }
    /* …h3, h4… */
  }
}
```

The reasoning behind the scale:

- **h1 is often the post title** rendered by the layout's `Card` (not by markdown). When h1 appears in body, it's a rare top-of-section marker; the larger size signals that weight.
- **h2 is the primary structural unit** of the body — every blog post will have many. Pair the size override with a border-bottom or generous top margin to give the section change visual weight.
- **h3 / h4 use the same weight (500) as h2.** Differentiation comes from size + letter-spacing, not weight. Three different weights (700 / 600 / 500) for three nested levels would feel chaotic; one consistent weight with size deltas reads cleaner.
- **h4 matches body size.** Same `1rem`, differentiated by weight (500 vs body's 400 effective) and slight negative tracking. Use sparingly — most posts shouldn't nest four heading levels deep.

## What the plugin handles well (don't override)

- **Vertical rhythm.** `margin-top` / `margin-bottom` for every element type. The plugin's spacing is well-tuned; overriding piecemeal makes the post feel uneven.
- **List markers.** Bullet shape, indent, marker colour.
- **Tables.** Cell padding, header weight, border treatment.
- **`blockquote`.** The default left-border + italic. Override only if the project has a stronger editorial register for quotes (Jav adds a `.c_Blockquote` class with a leading `"` glyph).
- **`hr`.** Centered, full-width with a colour.

These all read from `--tw-prose-*` colour variables, so the colour-token wiring above flows into them automatically.

## What the plugin handles poorly (override)

- **Heading scale.** Generic. Always override per project.
- **`strong` weight.** Defaults to 700 (browser-bold). With headings at 500–600, body bold at 700 overpowers them. Drop to 500 and let colour differentiate.
- **Inline code.** Defaults to weight 600 with surrounding quote marks. Always reset weight and remove the quotes; pair with a small chip background.
- **Link underlines.** The plugin underlines all links with a default offset. The org's [Design § links](../Design/linker_Design.md) has stronger opinions about link affordance — apply those rules in `prose_Style.css` or in `Design/`-area utilities that the prose inherits.
- **Code-block frames.** The plugin's `<pre>` styling collides with Expressive Code. EC renders its own frame; reset the plugin's `<pre>` styles to avoid double borders / padding.

## Responsive size ladder

The size modifier per breakpoint changes both base font-size and the `--tw-prose-*` size variables. Combined with the body-size `clamp()` (which sets a min and max regardless of breakpoint), the result is a smooth scale:

| Breakpoint | Modifier | Body size (clamp + plugin) |
|---|---|---|
| `< sm` | `prose` | 16px (clamp floor) |
| `sm+` | `prose-base` | 16px (still on floor at sm width) |
| `md+` | `prose-lg` | ~17px (clamp interpolating) |
| `xl+` | `prose-xl` | 18px (clamp ceiling) |

The clamp is what makes the scale feel continuous instead of stepping. Pick `clamp(1rem, 2.5vw, 1.125rem)` for blogs or adjust per project — the floor/ceiling values should sit comfortably either side of the 16/18px reading sweet spot.

## Reader Settings interactions

The runtime Aa panel (see [`../Fonts/reader_Settings.md`](../Fonts/reader_Settings.md)) overrides `font-size` and `line-height` on `article.prose` via inline styles. The override pattern in this doc must coexist:

- **Don't `!important` the body size in `prose_Style.css`** — inline styles already win against any normal-specificity selector; `!important` here would break the Aa panel.
- **Per-font size correction.** The reader-settings panel applies a `sizeScale` multiplier per font choice (mono fonts run large). The Aa panel writes the final computed size to the inline style; `prose_Style.css` provides the *base* the panel reads via `getComputedStyle()`.

If the Aa panel reads the wrong base, it's because the override stylesheet's `font-size` rule isn't visible in computed styles — usually a specificity issue or a missing `@import` in `posts_Main.css`.

## Add a new prose override — checklist

1. **Is it a colour override?** Add a `--tw-prose-*` variable assignment in `prose_Style.css`. Source the value from the project's design tokens (`var(--foreground)`, `var(--muted)`, `color-mix(...)`). Never hardcode.
2. **Is it a per-element override (size, weight, spacing)?** Nest under `[data-theme] & .prose.prose & <element> { … }`. Specificity wins, no `!important`.
3. **Is it a structural change (margins, layout)?** Likely a smell. Tailwind Typography's spacing rhythm is tuned; per-project overrides usually mean the project should reconsider the override, not the rhythm.
4. **Verify against both themes.** `bun run dev`, toggle dark/light, check the override holds. Colour overrides via `color-mix(in oklch, var(--foreground) …)` should work without per-theme blocks; if you wrote `[data-theme="dark"] & .prose { … }` you probably should have used a token.
5. **Verify against the Aa panel.** Apply Small / Medium / Large via the panel; the size should change. If it doesn't, the inline style isn't winning — check the override specificity.

## Anti-patterns

- **Hardcoded colours in `prose_Style.css`.** Always tokens. The whole reason `@tailwindcss/typography` exposes `--tw-prose-*` is to make this clean.
- **`!important` on body size.** Breaks the Aa panel. Specificity (`.prose.prose`) is the right lever.
- **One `prose_Style.css` per language.** The plugin is locale-agnostic by design. Differences (CJK weight ceilings, Arabic font, etc.) belong in the project's font-system rules (see [`../Fonts/cjk.md`](../Fonts/cjk.md)), not in prose.
- **Replicating the plugin's spacing.** If you find yourself writing margin rules for every prose element, you've overridden the wrong thing. Step back and decide whether the plugin's spacing is wrong (rare) or whether a different size modifier (`prose-base` vs `prose-lg`) is the lever you actually want.
- **Skipping the `.prose.prose` double-class.** Tailwind Typography defaults are at `.prose` specificity; your single-class overrides will lose to them in places. The doubled selector wins cleanly without `!important`.
