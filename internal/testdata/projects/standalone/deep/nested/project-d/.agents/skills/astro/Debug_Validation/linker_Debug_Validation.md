# Debug / Validation

Cross-cutting patterns for catching errors early and hard in Astro projects. Astro's SSR pipeline silently accepts type mismatches — a missing prop becomes `undefined`, the component keeps rendering, and the actual crash happens lines later with a confusing stack trace pointing at the wrong place. The patterns here prevent that by crashing immediately at the point of failure with a clear message.

## Boundary against adjacent areas

- **Content collection schema validation** (Zod) → [`../Content/schema_Patterns.md`](../Content/schema_Patterns.md). That's for validating authored MDX frontmatter at build time. This folder is for validating runtime props passed between Astro components.
- **TypeScript interfaces for props** — the org does not use `interface Props {}` blocks in Astro components. ArkType replaces that pattern with runtime validation that actually crashes instead of silently accepting wrong shapes.

## Routing

- [Props validation with ArkType](props_Validation.md): why Astro props are silently unsafe, the org-standard ArkType pattern (`ark_Type` import, `props_Schema`, `props_Result`, explicit error check), naming conventions, when to apply, and the deliberate-crash test technique. Use when adding props to any Astro component or layout.
