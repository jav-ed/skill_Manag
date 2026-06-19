# Links And Cards

Links in MDX have two layers: the automatic `a` override for prose links, and explicit card components for visual link moments. Keep ordinary links inline. Use cards only when the link is the content beat.

## Inline_Link

`Inline_Link` overrides markdown's raw `<a>` output. It is one of the highest-leverage entries in the MDX map because every prose link passes through it.

Internal links are authored as canonical paths and resolved to the current language through the site's route helpers. That lets authors write one path shape without maintaining per-language link tables.

Canonical internal links are English route targets, not localized URLs. In MDX, write links like `/contact`, `/services/preventive-care`, `/blog/professional-dental-cleaning`, or `/glossary/caries`. Do not write `/de/...`, `/en/...`, or `/es/...` in normal MDX prose.

`Inline_Link` must call the shared `resolve_Site_Link()` helper for internal links. Unknown canonical targets and raw localized internal paths should crash the build with the authored href and the source page path in the error. This is intentional: a broken internal link should be discovered during `bun run build`, not after publish.

External links get `target="_blank"` and `rel="noopener noreferrer"`. Projects may add a small visual external-link indicator, but the security attributes are the required behavior.

Without this override, markdown links are raw HTML anchors with no localization and no external-link guardrails.

## Fixed_Lang_Link

`Fixed_Lang_Link` is the narrow escape hatch for links that must deliberately point at one language, for example an English privacy page linking to the legally binding German imprint. It takes `lang` and `to`, where `to` is still a canonical route target:

```mdx
<Fixed_Lang_Link lang="de" to="/legal-notice">German imprint</Fixed_Lang_Link>
```

Do not use `Fixed_Lang_Link` for ordinary cross-page navigation. If the target should follow the reader's language, use a normal markdown link or `I_Link_Card`.

## Link Cards

`E_Link_Card` is for external destinations. It takes a raw URL, title, and description.

`I_Link_Card` is for internal destinations. It takes a canonical internal target and resolves the localized URL through the same route machinery as `Inline_Link`.

Use cards for deliberate navigational moments: source repositories, previous/next context, canonical references, or a related post that deserves visual weight. Do not scatter cards through normal prose.

## Related Docs

- [Component map](component_Map.md): how `a`, `E_Link_Card`, and `I_Link_Card` are registered.
- [`../../Scripts/astro_Frontmatter.md`](../../Scripts/astro_Frontmatter.md): route helpers and build-time page utilities.
