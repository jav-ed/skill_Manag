# MDX Components

Blog posts get custom components through a shared MDX component map. The page layout hands `src/Components/Mdx/mdx_Components.ts` to Astro's `<Content components={...} />`, so authors can write `<Post_Img />`, `<E_Link_Card />`, `<Definitions>`, and other helpers without importing them in every `.mdx` file.

This file is now only the entry point. The component set is large enough that the details live in a dedicated folder.

## Where to Go

- [MDX components overview](MDX_Components/linker_MDX_Components.md): the full navigation layer for the MDX component subsystem. Opens with the component-map mental model, then routes to leaf docs for the shared map, media, links, structured blocks, code, tables, charts, and diagrams.

- [Component map](MDX_Components/component_Map.md): how `mdx_Components.ts` is registered, why the map improves author velocity, how lowercase element overrides differ from PascalCase explicit components, and how per-category components are added with map spreading.

- [Post_Img](MDX_Components/post_Img.md): the blog image wrapper around Astro `<Picture>`. Covers imported image sources, lightbox behavior, AVIF/WebP output, `widths`/`sizes`, the deliberate desktop sharpness bias, the no-upscale guarantee, and when authors should override defaults.

- [Links and cards](MDX_Components/links_And_Cards.md): `Inline_Link`, `Fixed_Lang_Link`, `E_Link_Card`, and `I_Link_Card`.

- [Structured blocks](MDX_Components/structured_Blocks.md): `Definitions`, `Def_Item`, `Ref_List` (auto-rendered citation ledger), and `AI_Disclosure`. Also covers the FAQ frontmatter → FAQPage JSON-LD pattern (no MDX component for FAQ).

- [Citations](MDX_Components/citations.md): `<Cite n="N"/>` inline markers, `frontmatter.sources[]` data, auto-rendered `Ref_List`, hash-driven accordion opening, multilingual aria-labels.

- [Code and tables](MDX_Components/code_And_Tables.md): `Table_Wrapper`, `Inline_Code`, and `Code` from Expressive Code.
