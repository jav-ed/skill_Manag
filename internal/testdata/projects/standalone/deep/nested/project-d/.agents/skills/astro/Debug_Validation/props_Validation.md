# Props validation with ArkType

## Why this exists

Astro evaluates component frontmatter with no runtime prop checking. If a caller forgets to pass `lang`, the component receives `undefined`, keeps running, and crashes several lines later at the first place `lang` is actually used — pointing the stack trace at the wrong line and hiding the real cause. In a large component the actual bug can be very far from the actual error.

ArkType fixes this by validating `Astro.props` immediately at the top of the frontmatter. If a required prop is missing or the wrong type, it throws right there — at line 12, not at line 73 — with a message that names the component and the missing field.

The org does **not** use TypeScript `interface Props {}` blocks. TypeScript interfaces are compile-time only and Astro's build doesn't enforce them strictly at the prop-passing boundary. ArkType is a runtime check that actually crashes.

## The pattern

ArkType goes **first** — before every other import and every other line of code. A component's props are its public contract: inputs and outputs should be visible immediately, before any implementation detail.

```ts
import { type as ark_Type } from "arktype";

// Step 1 — declare the shape this component requires (only fields it actually uses)
const props_Schema = ark_Type({
  lang: "string",
});

// Step 2 — validate; returns data on success, ArkErrors object on failure (does not throw by itself)
const props_Result = props_Schema(Astro.props);
if (props_Result instanceof ark_Type.errors) throw new Error(`<ComponentName> props: ${props_Result.summary}`);

// Step 3 — destructure the validated props
const { lang } = props_Result;

// ... rest of imports and logic below
```

**Why the explicit error check?** ArkType does not throw automatically — it returns an `ArkErrors` object on failure. Without the `instanceof` check, `props_Result` would be the error object, destructuring `lang` from it gives `undefined`, and you're back to the silent-failure problem. The check is mandatory.

### Nested props

ArkType handles nested shapes naturally — declare the full structure you need:

```ts
import { type as ark_Type } from "arktype";

// Step 1 — declare shape
const props_Schema = ark_Type({
  mdx_Info: {
    lang: "string",
    seo: {
      title:       "string",
      description: "string",
    },
  },
});

// Step 2 — validate
const props_Result = props_Schema(Astro.props);
if (props_Result instanceof ark_Type.errors) throw new Error(`Card props: ${props_Result.summary}`);

// Step 3 — destructure
const { mdx_Info } = props_Result;
```

## Naming conventions

Following the org's `improved_Camel_Snake` convention:

| Name | Rule | Reason |
|---|---|---|
| `ark_Type` | verb `ark` lowercase, noun `Type` capitalized | Import alias for the `type` export from arktype |
| `props_Schema` | modifier `props` lowercase, noun `Schema` capitalized | Mirrors `faq_Schema`, `props_Type` patterns in the codebase |
| `props_Result` | modifier `props` lowercase, noun `Result` capitalized | The outcome of calling the schema on `Astro.props` |

Do not use `arkType` (camelCase — wrong convention), `safe_Checked_Props` (Bubble Tea's name — a different project), or `Props` (TypeScript interface style — not used here).

## Multi-prop components

Declare all props the component actually uses in one schema:

```ts
const props_Schema = ark_Type({
  lang:       "string",
  alternates: "Record<string, string> | null | undefined",
});
if (props_Result instanceof ark_Type.errors) throw new Error(`Header props: ${props_Result.summary}`);
const { lang, alternates } = props_Result;
```

Only list fields the component directly reads. Do not mirror the caller's full prop surface — if a parent passes 10 props and this component uses 2, declare only 2.

## Optional props

```ts
const props_Schema = ark_Type({
  lang:      "string",
  file_Path: "string | undefined",   // optional — omit is fine, undefined is fine
});
```

ArkType uses the `| undefined` union for optional fields, not `?`. Both work but `| undefined` is more explicit.

## When to apply

Apply to **every Astro component and layout** that receives props. No exceptions — even single-prop components like `<Draft_Notice lang={lang} />` benefit because the error message names the component.

Do **not** apply inside page files that read from `Astro.params` — params come from Astro's own routing, not from caller prop-passing, so the silent-undefined problem doesn't apply there.

## Error message format

Always include the component name in the thrown message:

```ts
throw new Error(`Footer props: ${props_Result.summary}`);
throw new Error(`Header props: ${props_Result.summary}`);
throw new Error(`Draft_Notice props: ${props_Result.summary}`);
```

This is the only context the developer has when the error surfaces in the terminal. Without it, a deep component tree gives you "must be a string (was missing)" with no indication of which of the 20 components on that page failed.

## Deliberate-crash test technique

When adding ArkType to a component for the first time, verify the error check actually fires before shipping:

1. Add a required field that no caller passes:
   ```ts
   const props_Schema = ark_Type({ lang: "string", deliberate_Crash: "string" });
   ```
2. Load any page that renders the component in the dev server.
3. Confirm the error message names the right component and field.
4. Remove `deliberate_Crash` and confirm the page renders normally.

This is a one-time sanity check — do it once per component, not on every change.
