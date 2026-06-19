# Data — Dev/

`src/Data/Dev/` holds dev-only feature flags — boolean toggles that gate in-development tools (font switchers, debug overlays, theme-picker visibility, layout-grid overlays) so they show up while you're building but stay invisible to real users.

The whole subfolder exists for one reason: **audit boundary**. Every dev flag in one place, easy to grep, easy to wipe to `false` before a production deploy. A flag scattered across a component file is a flag that ships to production.

## The pattern

One file, one exported object:

```ts
// src/Data/Dev/dev_Config.ts
//
// Toggle dev tools on/off globally. Set all to false before production.

export const DEV = {
  fontSwitcher: false,    // floating font candidate picker (bottom-left)
  themeOverlay: false,    // hover the body to see theme-token-name overlay
  layoutGrid:   false,    // CSS grid overlay for layout debugging
  // readerSettings: false,  // add the flag here when the feature lands
};
```

Consumed wherever the feature is conditionally rendered:

```astro
---
import { DEV } from "../Data/Dev/dev_Config";
---
{DEV.fontSwitcher && <FontSwitcherDevPanel />}
```

Or in client JS:

```ts
import { DEV } from "../../Data/Dev/dev_Config";
if (DEV.layoutGrid) initLayoutGridOverlay();
```

## Why a separate `Dev/` subfolder

Three reasons, all about confidence at deploy time:

1. **Single file to audit.** Before a release, the question "are all dev tools off?" is answered by reading one file. No grep, no per-component check.
2. **Single import to find.** When the project grows and a flag turns into a real feature, every consumer of `DEV.x` is one grep away.
3. **Clear separation from real config.** `src/Data/Common/` values are *part of* production. `src/Data/Dev/` values are *gates against* production. Mixing them invites accidentally shipping a dev tool because someone mass-flipped the wrong object.

The same `DEV.x = false` discipline could live inside the component file, but then auditing means trusting every contributor remembers to gate their dev tools. The folder convention removes the question.

## The "all false before production" discipline

Three project-wide rules:

1. **Default to `false`.** New flags ship as `false`, even while the feature is being developed. Flip locally; don't commit the flip.
2. **CI check (optional but recommended).** A pre-deploy script that asserts every flag in `DEV` is `false` in the committed source. A simple `grep ": true"` against `dev_Config.ts` is enough.
3. **Audit comment.** When a flag becomes obsolete (the feature shipped properly, or the dev tool got deleted), remove the line entirely. Don't leave `// obsolete` comments — they accumulate and the file becomes a fossil.

## When NOT to use a Dev flag

Dev flags are for in-development tools that should disappear in production. They are not:

- **Real feature flags** (showing a beta to 10% of users) — those need a server-side flagging system (LaunchDarkly, GrowthBook, your own DB-backed toggle), not a const exported from a TS file.
- **Per-environment configuration** (different API URL in staging) — those belong in environment variables read at build time, not a hardcoded boolean.
- **Per-user preferences** (user's chosen theme, accessibility settings) — those belong in user settings persisted to localStorage / DB, not in source code.
- **Per-tenant toggles** (customer A gets features X+Y; customer B gets X only) — those belong in the per-tenant config (the customer-config sibling repo for multi-tenant projects, or the tenant's content collection entries).

If the toggle survives past local development — if it ever needs to vary by environment, user, or tenant — it doesn't belong in `dev_Config.ts`. Move it before it ships.

## Naming

- File: `dev_Config.ts` (`.js` only for legacy projects).
- Exported object: `DEV` — single uppercase noun, no prefix. Imports read as `DEV.fontSwitcher`, which scans well in component conditionals.
- Flag keys: `camelCase`, descriptive of the *tool*, not the feature it precedes. `fontSwitcher` (the dev tool that lets you cycle fonts), not `customFonts` (which sounds like the production feature).
- Inline comment on each flag describing what the tool does, where it appears, and the reminder that it must be `false` before production.

## Add or remove — checklist

**Adding a flag:**

1. Pick a `camelCase` key naming the dev tool itself.
2. Default to `false`.
3. Inline comment: what tool, where it appears, "set false before production".
4. Import `DEV` in the component / script that conditionally renders the tool.
5. Wrap the dev UI in `{DEV.flagName && ...}` (Astro) or `if (DEV.flagName) ...` (JS).

**Removing a flag (the tool shipped properly or got deleted):**

1. Delete the line from `dev_Config.ts`.
2. Grep `DEV.flagName` across `src/` and remove every conditional.
3. If the tool became a real feature, ensure it's now controlled by a real toggle (env var, user setting, tenant config) — not just always-on.
