# TOC and chrome

The structural shell around the article body: table of contents, scroll-to-top/bottom icons, reading-progress bar, series-prev/next nav, breadcrumb, draft notice, work-with-me CTA. Each is independent — projects can ship the article without any of them — but together they make the difference between a readable post and a *navigable* one.

This doc covers the patterns. Jav_Web's internal docs ([Project_Manag/Docs/Architecture/Blog/toc.md](https://github.com/), `scroll_Icons.md`, `series_Nav.md`) have the full implementation detail file by file; the astro skill teaches the shape.

## Three folders cooperate

Every chrome element typically has three pieces:

| Piece | Lives in | Role |
|---|---|---|
| Markup | `src/Components/Posts/<Element>.astro` | The DOM the layout renders |
| Styles | `src/Styles/2_Pages/3_Posts/<element>.css` (or inside the entry chain) | Visual treatment |
| Behaviour | `src/Scripts/Browser_Client/Blog/<feature>.js` | Scroll observers, click handlers, state |

Splitting markup vs. behaviour is the leaf-layer rule applied at the page level — the Astro component is structurally pure (SSR-friendly, no DOM access), and the client script wires the live interaction once the page hits the browser. See [`../Scripts/browser_Client.md`](../Scripts/browser_Client.md) for the eager / `<script src>` / lazy import patterns.

## Blog chrome interaction contract

Icon-only controls in `src/Components/Posts/` use the shared compact-control primitives from [Design interactions](../Design/interactions.md):

```astro
<button type="button" class="icon-button surface-hover text-muted-foreground" aria-label={label}>
  <Lucid_List class="size-4" aria-hidden="true" />
</button>
```

Use a real `<button>` for panel toggles and close controls. Use an `<a>` for hash navigation controls such as scroll-to-top / scroll-to-bottom, with the same `icon-button surface-hover text-muted-foreground no-underline` class stack. Put the stable `id` and `aria-label` on the interactive element itself, not on the SVG. SVG icons stay presentational with `aria-hidden="true"`.

Do not put `role="button"` or `tabindex="0"` on Lucide SVGs. Do not use icon-scale hover for these compact controls; the filled `.surface-hover` hit area is the hover/focus confirmation.

## Table of contents — four variants

A single article serves multiple form factors: desktop sidebar that can carry rich visuals, inline mobile that's part of the article flow, and an overlay panel for mobile users who want to jump. Variants share one item component (`Toc_Header.astro` in Jav) so the per-item styling stays consistent.

### PC sidebar TOC (`lg+`)

Sticky in the right rail, always visible while scrolling. The rich one — supports active-state animation, vertical line fragments with bezier curves at depth transitions (clerk-style), and a scroll-icon pair plus a "work with me" tagline at the bottom.

**Architecture:**

- `Toc_Pc.astro` receives the raw flat `headings` array from Astro (the prop your page template gets from `entry.render()`) and filters to h1–h4. For each item it passes neighbour depths so the per-item component can draw the correct vertical-line fragment (a heading nested deeper than its predecessor draws a bezier curve down to its level).
- `Toc_Header.astro` is one item: link + depth-padded line fragment + `data-active` attribute.
- A client script (`animated_Toc.js` in Jav) lazy-loads the observer on first scroll — no JS until the user actually scrolls.
- The observer (`toc_Observer.js`) watches `article h1, article h2, article h3, article h4` with `IntersectionObserver` (threshold 0.98). Maintains an `active_ids` set; syncs `data-active` on the matching TOC links. Fallback: if nothing intersects, highlight the heading closest to the viewport top so *something* is always active.
- An SVG overlay (`toc_Svg.js` + `toc_Clip.js`) traces the full tree path using the same bezier geometry as the static per-item lines and uses `clip-path` to reveal only the active range. CSS transitions on the SVG animate movement between active states.

**Critical:** geometry constants (`getLineOffset`, `getItemOffset` with their depth-to-px tables) live in two files — the static markup component and the dynamic SVG script. They **must** stay in sync; a mismatch shows up as the SVG sliding off the per-item lines. Treat them as a single contract.

### Inline mobile TOC (`< lg`)

Lives inside the article flow, usually right after the post title card. No SVG — just a list of links with `data-active` driven by a lightweight observer (`mobile_Toc_Observer.js`) that doesn't carry the SVG-clip complexity. Same heading source, same `Toc_Header.astro` item component, simpler chrome.

### Mobile overlay panel (`< lg`)

A fixed full-screen panel toggled from a button in the header. Same heading data, same item component. Animation pattern: `data-open` attribute flips, CSS grid-row transitions from `0fr` → `1fr` to slide in (no `max-height` guesswork — `grid-template-rows: 0fr/1fr` animates cleanly to content height). Close on click-outside and on ESC.

### Header trigger button

Just an icon in the header that toggles the overlay panel. Hidden on `lg+` where the sticky sidebar makes the overlay redundant.

### Choosing variants

Not every project needs all four. The decision: how many simultaneous reading contexts does the project serve?

- **Long technical posts read primarily on desktop.** PC sidebar + inline mobile. Skip overlay.
- **Frequent mobile reading on long posts.** PC sidebar + overlay + header trigger. Skip inline.
- **Short posts under ~4 H2 sections.** Drop the TOC entirely. The reader can scan.

## Scroll icons

A pair of arrows: "back to top" and "jump to bottom." Independent on desktop and mobile because the visual contexts differ.

### Desktop (`lg+`)

Lives inside the PC sidebar TOC, below the navigation list. Always visible while the sidebar is. Use two hash-link anchors styled as `icon-button surface-hover text-muted-foreground no-underline`, each wrapping a presentational Lucide arrow SVG (`arrow-up` / `arrow-down`). The `first_href` and `last_href` props come from the layout (first and last heading slugs), so the links work with normal URL/hash behaviour and still give the compact filled hover/focus surface.

### Mobile (`< lg`)

A floating pill at the bottom-centre of the viewport. Starts invisible (`opacity-0`, `pointer-events-none`); shows only when the reader scrolls upward, hides on downward scroll, and auto-hides after about 2 s of idle time. This keeps reading mode quiet: downward scroll usually means "keep reading", upward scroll usually means "I need navigation or context." An `IntersectionObserver` watches the footer — when the footer enters view, the pill hides immediately so it doesn't overlap. The pill can also carry the mobile overlay TOC trigger; that trigger is a real `<button>` using the same `icon-button surface-hover` primitive as the scroll anchors.

Wiring in `Post_Layout.astro`:

```astro
<div class="fixed bottom-3 left-1/2 -translate-x-1/2 z-40 lg:hidden …" id="post_scroll_icons">
  <Scroll_Icons_Small icon_id_up="…" icon_id_down="…" first_href={first_href} last_href={last_href} lang={lang} />
</div>

<script>
  // listen for scroll, show on upward intent, hide on downward scroll or idle
  // IntersectionObserver on footer to force-hide on overlap
</script>
```

The script is inline because it's a one-off behaviour tightly coupled to the layout's specific DOM (`#post_scroll_icons`, `<footer>`). Lifting it to `Browser_Client/Blog/` would force a contract for a script with one consumer — keep it inline. See [`../Scripts/browser_Client.md`](../Scripts/browser_Client.md) § three wiring patterns.

## Reading-progress bar

A thin bar pinned to the top of the article (or the viewport) that fills as the reader scrolls through the post. CSS-only via scroll-driven animations:

```css
@keyframes post_progress { from { transform: scaleX(0); } to { transform: scaleX(1); } }

.post-reading-progress {
  position: fixed;
  inset: 0 0 auto 0;
  height: 2px;
  background: var(--primary);
  transform-origin: left;
  animation: post_progress linear;
  animation-timeline: scroll();
  animation-range: 0 100%;
}
```

Place the `<div class="post-reading-progress" aria-hidden="true"></div>` in `Post_Layout.astro` right after `<Top_Header>`. Browsers without scroll-driven-animations support (older Safari) get a static empty bar — graceful degradation. Wrap the keyframes in `@supports (animation-timeline: scroll())` if you want to suppress the bar entirely on unsupported browsers. See [`../Design/`](../Design/linker_Design.md) § motion for the scroll-driven-animation pattern.

## Series navigation

End-card prev/next at the bottom of every post in a multi-part series. Always shows "back to series" link plus the prev and next post cards (with the next post on the right, prev on the left); cards are null at boundaries (first post has no prev; last has no next).

### Data flow

```
page component
  → compute_Series_Nav({ collection_Key, entry, lang, series_Url, series_Labels })
      → getCollection(collection_Key)
      → filter by lang prefix, sort by id (numeric NNNN_ prefix drives order)
      → find current index → slice prev / next
      → resolve prev/next URLs via resolve_Internal_Target (cached content route index)
      → return { prev, next, series_Url, series_Label, current_pos, total }
  → Post_Layout (series_nav prop)
    → Series_Nav component
```

`compute_Series_Nav` is a shared helper (Jav: `src/Utils/Blog/series_Nav_Factory.js`). It returns one `series_nav` object regardless of which series the post belongs to; the page template hands it to the layout as a prop.

`prev` and `next` are `{ title, subtitle, url }` or `null`. URLs are localised per-language because they come from the cached content route index (see [`../Scripts/astro_Frontmatter.md`](../Scripts/astro_Frontmatter.md)) — the same source the canonical-link resolver uses.

### View Transitions for post-to-post morphing

In supporting browsers, navigating between two posts in a series can morph the post title across the page load instead of jumping. Two pieces:

1. **Opt into cross-document View Transitions** in the post stylesheet:
   ```css
   @view-transition { navigation: auto; }
   ```
2. **Mark the title element with a transition name** on both the source card and the destination heading:
   ```astro
   <h1 style="view-transition-name: post-title">{title}</h1>
   ```

Chrome and Safari animate the title across the navigation; Firefox falls back to a standard page load (no morph, identical end-state). Custom timing in `series_Nav.css` under `::view-transition-old(post-title)` / `::view-transition-new(post-title)`.

### Adding a new series

1. Add the collection key → canonical structural-segment array to `blog_Linker.js` (or equivalent). Values must be tuples of `route_Slugs` keys (no URL strings).
2. In the page route file, call `compute_Series_Nav` with `collection_Key`, `entry`, `lang`, `series_Url` (built via `build_Path_No_Trailing`), and `series_Labels`.
3. Pass `series_nav={series_nav}` to `<Post_Layout>`.

No layout or component changes needed.

## Other chrome bits

### Breadcrumb

Above the article. Renders the localised path from home → topic → subtopic → post title. Built from `route_Slugs.ts` (per-language labels for structural segments) and the post's title. See [`../Scripts/astro_Frontmatter.md`](../Scripts/astro_Frontmatter.md) § route_Slugs for the underlying map. The visible breadcrumb and the JSON-LD `BreadcrumbList` (see [`../SEO/json_Ld.md`](../SEO/json_Ld.md) § BreadcrumbList) should be driven by the same data — diverging them invites the JSON-LD to advertise different navigation than what the page shows, which Google flags.

### Draft notice

Frontmatter-driven. When the post's `status: "draft"`, the layout renders a translated banner above the body and the post's overview card carries a "draft" badge. Automatic — no MDX import needed. The translated strings live in `Multi_Lang_Txts/` per the project's localisation convention (see [`../Scripts/multi_Lang_Txts.md`](../Scripts/multi_Lang_Txts.md)).

### Work-with-me CTA

Optional editorial element after the article body. A short pitch + link to the contact / hire page. Crossfade animation (`work_Crossfade.js` in Jav) reveals it once the reader is past a certain scroll threshold — keeps it from competing with the article content. Project-dependent; skip if the blog isn't a lead source.

### Bismillah intro

Optional opening. Renders `بِسْمِ ٱللَّٰهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ` at the top of every post in a special font (Al Qalam). Pattern: a `.post-bismillah` element rendered unconditionally by the layout, styled with `font-family: var(--font-al_qalam)`. Project-dependent.

## Wiring everything in `Post_Layout.astro`

The layout assembles the pieces. Stripped to structure:

```astro
<Base_Layout og_Type="article" extra_Schemas={extra_Schemas}>
  <Top_Header url={url} path={path} lang={lang}>
    <Breadcrumb slot="breadcrumb" crumbs={post_crumbs} />
  </Top_Header>

  <div class="post-reading-progress" aria-hidden="true"></div>

  <main class="…">
    <article class="prose prose-base md:prose-lg xl:prose-xl …" lang={lang}>
      <Card mdx_Info={…} remark_info={…} />
      {is_Draft && <Draft_Notice lang={lang} />}
      <Toc_Small headings={headings} lang={lang} />
      <slot name="main_text" />
      {series_nav && <Series_Nav {...series_nav} lang={lang} />}
      <Work_With_Me lang={lang} />
    </article>

    {/* mobile scroll icons */}
    <div class="fixed bottom-3 …" id="post_scroll_icons">
      <Scroll_Icons_Small first_href={first_href} last_href={last_href} lang={lang} />
    </div>
    <script>{/* show/hide on scroll + footer-intersect */}</script>

    {/* mobile overlay TOC */}
    <Small_Side_Toc headings={headings} lang={lang} />

    {/* PC sidebar TOC */}
    <Toc_Pc headings={headings} first_href={first_href} last_href={last_href} lang={lang} />
  </main>

  <script src="src/Scripts/Browser_Client/Blog/work_Crossfade.js"></script>
  <Footer lang={lang} slot="footer" />
</Base_Layout>
```

The order is intentional: progress bar before `<main>` so it sits above everything; `<article>` carries the prose styling and contains body + draft notice + inline TOC + content + series nav + CTA; chrome elements (mobile icons, overlay TOC, PC sidebar TOC) sit as siblings of `<article>` because they're fixed-positioned and shouldn't inherit prose styles.

## Build vs ship — which chrome elements warrant which order

Per [Bootstrapping a blog](linker_Blog.md#bootstrapping-a-blog-on-a-new-astro-project) step 7, the article is readable without any chrome. Add elements in this rough order, dropping any the project doesn't need:

1. **TOC (one variant)** — the highest-leverage element. Pick PC sidebar if the project's audience is mostly desktop; pick inline mobile if mostly phone. Skip if posts are short.
2. **Breadcrumb** — cheap; nearly always worth it.
3. **Series nav** — required as soon as the project has a multi-part series. Skip until then.
4. **Reading-progress bar** — cheap. CSS-only, no JS, no observer.
5. **Mobile scroll icons** — only if mobile posts run long enough that scrolling far feels expensive.
6. **Mobile overlay TOC + header trigger** — only if mobile-first long posts are the norm.
7. **Draft notice** — needed as soon as the project drafts posts in repo before publishing. Cheap.
8. **Work-with-me CTA** — only on lead-generating blogs.
9. **Bismillah intro / project-specific intros** — only when the editorial frame calls for it.

Don't pre-build chrome the project doesn't have a use for. Each element carries maintenance cost (markup + style + script) that's hard to justify if no post benefits.
