# Citations

Citations let a long-form blog post reference external sources inline and render them as a structured ledger at the bottom of the page. The system has three coupled pieces: `frontmatter.sources[]` (data), `<Cite n="N"/>` (inline marker), and `Ref_List` (auto-rendered renderer). They cooperate via stable `#source-{n}` anchors and a hash-driven open behavior on the collapsed list.

This is distinct from FAQ frontmatter (which drives FAQPage JSON-LD only — see [Structured blocks](structured_Blocks.md)) and from `AI_Disclosure` (an authorship note, not a citation).

## The three pieces

### 1. `frontmatter.sources[]` — the data

Each post lists its sources in frontmatter, validated by `schema_Blog_500.ts`:

```yaml
sources:
  - n: "1"
    title: "Bundeszahnärztekammer — Patientenleitfaden zur PZR"
    url: "https://www.bzaek.de/fuer-patienten.html"
  - n: "2"
    title: "DGZMK / DG PARO — Wissenschaftliche Stellungnahme"
    url: "https://www.dgzmk.de/"
```

`n` is a string — typically a number, but the schema allows `"a"`/`"b"` if a post ever wants letter footnotes. It is the stable identifier used both inline and in the `#source-{n}` anchor.

### 2. `<Cite n="N"/>` — the inline marker

Authors mark a citation in body prose with `<Cite n="2"/>`. The component renders a small superscript link to `#source-2` with a localized aria-label (`Quelle 2` / `Source 2` / `Fuente 2` per `t_Blog_Cite_Aria` in `blog_Ui_Txt.ts`).

```mdx
Die deutschen Fachgesellschaften empfehlen den ersten Termin
ab dem ersten Milchzahn.<Cite n="1"/>
```

Visually, `<Cite/>` is muted-foreground at rest and foreground on hover/focus. It does **not** pick up the `Inline_Link` `border-b` underline — citations are reference markers, not body destinations, and styling them like inline links would clutter the prose. This is deliberate; see Anti-patterns.

### 3. `Ref_List` — auto-rendered, collapsible

`Post_Layout.astro` auto-renders `<Ref_List sources={frontmatter.sources}/>` at the end of every post that has a `sources` array. Authors do not write `<Ref_List/>` inline for the post's own sources.

Each row (`Src_Item.astro`) renders as `[n] Title  domain ↗`, links out to the source URL on click, and carries `id="source-{n}"` so `<Cite n="N"/>` anchors resolve. Each row also has `scroll-mt-24` so hash-driven scrolls clear the sticky header.

The list is collapsible by default. The reader sees a header bar with a count chip (`4 Quellen`) and a chevron; clicking the header opens the accordion with a 300ms transition.

The list is rendered between the MDX content (so after the author's "Zu diesem Beitrag" disclaimer if one exists) and the `Post_Author_Note` byline. This order matters — the disclaimer's "Quellenangaben siehe Liste am Seitenende" promise must be fulfilled by what follows.

## Hash-driven opening

The collapsibility creates a UX problem: a `<Cite/>` click sets the URL hash to `#source-2`, but the row is hidden inside a collapsed accordion. Without intervention the browser would scroll to a zero-height element and the reader would see a closed list with no obvious connection to the citation they just clicked.

`Ref_List.astro`'s browser script handles this:

- On initial page load AND on `hashchange`, if the hash matches a `#source-N` row inside this list, open the accordion **instantly** (CSS transition temporarily disabled to skip the 300ms animation), then `requestAnimationFrame` → `scrollIntoView({block: "center"})`.
- `prefers-reduced-motion`: list opens by default at page load and the scroll uses `behavior: "auto"`.

The instant-open trick matters: a reader who followed a citation link wants to read the source immediately, not watch the accordion animate first.

## Multilingual

Every user-visible string flows through `blog_Ui_Txt.ts` and is asserted by `assert_All_Langs` at build time:

| Key | Used by | de | en | es |
|---|---|---|---|---|
| `t_Blog_Cite_Aria` | `Cite` aria-label | `Quelle` | `Source` | `Fuente` |
| `t_Blog_Refs_Label` | `Ref_List` header | `Quellen` | `Sources` | `Fuentes` |
| `t_Blog_Refs_Count` | `Ref_List` count chip | `Quellen` | `refs` | `fuentes` |

Add a new lang to any of these and the build fails until all three (and any other `t_Blog_*` keys) include it. Components pick their lang via `detect_Lang(Astro.url.pathname)` at render time — no `lang` prop threading needed.

## Anti-patterns

- **Raw Unicode superscripts (`¹` `²` `³`) in MDX.** They are plain glyphs — no link, no anchor, no screen-reader semantics, no hash routing. The disclaimer "Quellenangaben siehe Liste am Seitenende" reads as broken because nothing scrolls to the list. Always use `<Cite n="N"/>`. After authoring, sweep with `rg -n "¹|²|³|⁴|⁵|⁶|⁷|⁸|⁹|⁰" src/Content/500_Blogs` to confirm no glyphs slipped through.
- **`<Ref_List/>` inline when sources are in frontmatter.** `Post_Layout.astro` already auto-renders it. Adding a second copy creates two ledgers on the page. The legitimate exception is `<Ref_List headless sources={…}/>` mid-post under an author-supplied heading — different shape, deliberate.
- **Adding the inline-link `border-b` to citation styling.** Citations and inline body links are different visual classes. If a future author "fixes" the missing underline, they will be wrong.
- **Removing `scroll-mt-*` from `Src_Item`.** Without it, hash-driven scrolls land the target partially under the sticky page header.
- **Threading `lang` as a prop into `Cite` or `Ref_List`.** The components already self-detect via `detect_Lang(Astro.url.pathname)`. Adding a prop creates two sources of truth.

## File map

| File | Role |
|---|---|
| `src/Scripts/Content_Schemas/schema_Blog_500.ts` | `sources[]` Zod schema (`n`, `title`, `url`) |
| `src/Components/Mdx/Default/Inline/Cite.astro` | Inline `<sup><a href="#source-N">` marker |
| `src/Components/Mdx/Default/Lists/Ref_List/Ref_List.astro` | Collapsible accordion + hash-navigation script |
| `src/Components/Mdx/Default/Lists/Ref_List/Src_Item.astro` | Single source row with `id="source-{n}"` + scroll margin |
| `src/Layouts/Posts/Post_Layout.astro` | Auto-renders `<Ref_List/>` after `<slot name="main_text">` |
| `src/Scripts/Multi_Lang_Txts/Pages/blog_Ui_Txt.ts` | `t_Blog_Cite_Aria` + Ref_List label strings |
| `src/Components/Mdx/mdx_Components.ts` | Registers `Cite` (and `Ref_List` for the headless inline case) |

## Related docs

- [Component map](component_Map.md) — where `Cite` is registered.
- [Structured blocks](structured_Blocks.md) — sibling structured-data patterns (FAQ → JSON-LD, `Definitions`, `AI_Disclosure`).
- [`../../Content/schema_Patterns.md`](../../Content/schema_Patterns.md) — the `sources[]` field as an example of embedded structured frontmatter.
