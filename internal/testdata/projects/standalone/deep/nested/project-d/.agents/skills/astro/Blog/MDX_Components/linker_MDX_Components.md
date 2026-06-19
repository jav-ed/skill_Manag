# MDX Components

The MDX component subsystem is how blog authors get a stable writing toolkit without per-file imports. A shared `mdx_Components.ts` map is registered on the post route/layout and passed to `<Content components={...} />`. Markdown element overrides such as `a`, `table`, and `code` become automatic everywhere; PascalCase helpers such as `Post_Img`, `E_Link_Card`, and `Definitions` are available by name when an author needs them.

The guiding split is: keep the map small enough to understand, but document each component family separately once behavior grows past a paragraph. `Post_Img` is the clearest example: it is not just a wrapper, it carries image-quality policy, lightbox protocol, and Astro responsive-image assumptions.

## Component Docs

- [Component map](component_Map.md): the shared map pattern, route wiring, lowercase element overrides versus PascalCase explicit helpers, per-category map spreading, and add-component checklists.

- [Post_Img](post_Img.md): the image component for MDX prose and static editorial pages. Use this when changing image sizes, formats, loading behavior, lightbox behavior, captions, or `widths`/`sizes` policy.

- [Links and cards](links_And_Cards.md): `Inline_Link` for localized internal links and secure external links, plus `E_Link_Card` and `I_Link_Card` for deliberate visual link beats.

- [Structured blocks](structured_Blocks.md): the FAQ frontmatter → FAQPage JSON-LD pattern (no MDX component), `Definitions`/`Def_Item` for in-body Q&A, the `Ref_List` collapsible sources renderer (auto-injected at the end of posts), and `AI_Disclosure` for versioned authorship notes.

- [Citations](citations.md): `<Cite n="N"/>` inline markers, the `frontmatter.sources[]` data shape, auto-rendered `Ref_List` ledger, multilingual aria-labels via `t_Blog_Cite_Aria`, and the hash-driven open behavior that keeps citation clicks from landing on a collapsed accordion. Open when adding or modifying citations.

- [Code and tables](code_And_Tables.md): `Table_Wrapper`, `Inline_Code`, and `Code` from `astro-expressive-code/components`.

- [Charts](../charts.md): `E_Chart`, author flow, theme inheritance, and SVG renderer behavior.

- [Diagrams](../diagrams.md): `Mermaid_Diagram`, static `.mmd` to `.svg` rendering, and no-runtime diagram policy.

## Routing Questions

- "How do authors get components without imports?" → [Component map](component_Map.md)
- "Why is a blog image requesting a larger candidate than its rendered width?" → [Post_Img](post_Img.md)
- "Where does FAQ frontmatter connect to FAQPage JSON-LD?" → [Structured blocks](structured_Blocks.md) + [`../../SEO/json_Ld.md`](../../SEO/json_Ld.md)
- "How do I add a citation / footnote to a blog post?" → [Citations](citations.md)
- "A citation link scrolls into a collapsed list and looks broken — what handles the open?" → [Citations § Hash-driven opening](citations.md)
- "How should a link become localised or open externally?" → [Links and cards](links_And_Cards.md)
- "How do tables avoid overflowing prose?" → [Code and tables](code_And_Tables.md)
- "How do I add a helper for only one post category?" → [Component map § Per-category extras](component_Map.md)
