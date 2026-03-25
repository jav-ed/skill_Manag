# Code Blocks Usage

## Fenced blocks in MDX

Standard backtick fences — expressive-code processes them automatically:

````md
```js
const foo = 'bar'
```
````

No import needed. Works in all `.mdx` files.

## `<Code>` component

For dynamically imported or runtime-generated code, use the component:

```mdx
import { Code } from 'astro-expressive-code/components';
import importedCode from "@content/500_Python_Series/en/Code/0004_init.py?raw";

<Code code={importedCode} lang="py" />
```

Note the `?raw` suffix on the import — required to get file contents as a string.

## Default props (global)

Set in `ec.config.mjs` under `defaultProps`. All code blocks inherit these unless overridden:

| Prop | Value | Effect |
|---|---|---|
| `wrap` | `true` | Long lines wrap instead of scroll |
| `showLineNumbers` | `true` | Line numbers shown on all blocks |
| `hangingIndent` | `4` | Wrapped continuation lines indented 4 columns |

## Per-block overrides

### Via fence meta

````md
```js wrap=false showLineNumbers=false
short snippet, no numbers needed
```
````

````md
```py hangingIndent=2
some_long_function(argument_one,
  argument_two)
```
````

### Via `<Code>` props

```mdx
<Code code={importedCode} lang="py" wrap={false} showLineNumbers={false} />
```

## `hangingIndent` behaviour

Only active when `wrap` is `true`. Adds N columns of indent to wrapped continuation lines — visual only, clipboard always gets the original unwrapped code.

With `preserveIndent` (default `true`): continuation lines get `original line indent + hangingIndent` columns.
