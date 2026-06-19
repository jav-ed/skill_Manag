# Structured Blocks

Structured blocks are MDX helpers that carry content semantics beyond plain prose. They are usually explicit PascalCase components in the MDX map.

## FAQ (frontmatter → JSON-LD only)

There is no `FAQ` MDX component in the map. The `faq:` frontmatter exists only to drive **FAQPage JSON-LD** for search engines — `Post_Layout.astro` reads `frontmatter.faq?.items` and feeds it into `build_Faq_JsonLd` (see [`../../SEO/json_Ld.md`](../../SEO/json_Ld.md)).

```yaml
faq:
  items:
    - q: Question text?
      a: Answer text.
```

If a post needs a visible FAQ section in the body, write it as ordinary MDX (a heading plus paragraphs, or `<Definitions>` blocks below). The frontmatter handles the structured-data half; the visible half is the author's call. An earlier `FAQ.astro` MDX component was removed (it had no callers).

## Definitions And Def_Item

`Definitions` and `Def_Item` are in-body Q&A blocks. They are different from the FAQ frontmatter pattern: FAQ frontmatter drives JSON-LD only, while Definitions is a visible mid-flow free-form block.

Definitions can also be used for optional lookup or appendix-style terminology when the main article does not depend on the collapsed answers for the next paragraph, chart, diagram, or argument.

```mdx
<Definitions>
  <Def_Item q="What is the result of `1.0 / 0`?">
    Python raises `ZeroDivisionError`.
  </Def_Item>
</Definitions>
```

## Ref_List

`Ref_List` is the rendering half of the post citation system: a collapsible accordion that lists the post's `frontmatter.sources[]`. `Post_Layout.astro` auto-injects it at the end of every post that has sources — authors do not write `<Ref_List/>` inline for the post's own sources. The inline form is still supported for the rare `<Ref_List headless sources={…}/>` mid-post block under an author-supplied heading.

The full system — `<Cite n="N"/>` markers, `#source-{n}` anchor convention, multilingual aria-labels, and the hash-driven open behavior that keeps citation clicks from landing on a collapsed accordion — is documented separately in [Citations](citations.md).

## AI_Disclosure

`AI_Disclosure` renders a versioned authorship notice. It can pick text based on route/category and can support a pinned `variant` when an older post should keep its original disclosure.

## Related Docs

- [Citations](citations.md): full citation system — `<Cite n="N"/>` inline markers, `frontmatter.sources[]`, auto-rendered `Ref_List`, hash-driven accordion opening, and `t_Blog_Cite_Aria` multilingual labels.
- [`../../SEO/json_Ld.md`](../../SEO/json_Ld.md): FAQPage and other structured-data builders.
- [Component map](component_Map.md): registration and per-category component rules.
