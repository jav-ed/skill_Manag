# Code And Tables

Code and table helpers are mostly automatic markdown overrides. Authors usually write normal markdown and let the MDX map replace the raw HTML output with safer project components.

## Table_Wrapper

`Table_Wrapper` overrides markdown tables and wraps them in an overflow container. Wide GFM pipe tables become horizontally scrollable on small screens instead of breaking the prose column.

Authors should still keep tables compact and meaningful. The wrapper prevents layout damage; it does not make huge tables easy to read on a phone.

## Inline_Code

Markdown's `code` mapping catches both inline code and fenced code internals. Expressive Code handles fenced blocks; `Inline_Code` is for single-backtick inline code.

Inline code should use the project mono font and muted chip styling. Mono fonts often run visually larger than prose text, so typography docs may define a smaller inline-code scale.

## Code

`Code` from `astro-expressive-code/components` renders imported source strings as highlighted blocks:

```mdx
import config from "../../../astro.config.mjs?raw";

<Code code={config} lang="js" title="astro.config.mjs" />
```

Normal fenced markdown blocks still use triple backticks. Use `Code` when the source lives in a file or variable rather than inline in MDX.

## Related Docs

- [Markdown pipeline](../markdown_Pipeline.md): Expressive Code config and markdown transforms.
- [Typography](../typography.md): prose styling and inline-code sizing.
