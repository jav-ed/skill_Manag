# Data

`src/Data/` holds the **infrastructure constants** of an Astro project — the values that change when the project's *shape* changes (a new language gets added, the domain moves, the brand contract for OG images is renegotiated), not when a feature evolves. It is a small folder by design. Most projects have `Common/` for always-on infrastructure and `Dev/` for dev-only feature flags. Projects with generated social cards also get `Og_Img/` for the OG image data contract.

This area is deliberately separate from [Utils](../Utils/linker_Utils.md). Both hold cross-cutting site-wide imports; the split below tells you which.

Adjacent question — Data vs MDX vs Multi_Lang_Txts: if you're unsure whether a value belongs here at all (versus a content collection or a translation table), see [Content vs Config vs Labels](../Content/content_Vs_Config_Vs_Labels.md). The short version: Data holds build-side config; site-wide UI labels go to Multi_Lang_Txts; page-bound content goes to MDX. The doc walks the gray-zone cases (opening hours, address block) where either Data or a sibling format is defensible.

## The mutability rule — Data vs Utils

| Question to ask | Goes in `src/Data/` | Goes in `src/Utils/` |
|---|---|---|
| Who edits this, and when? | Project engineers, when the project's structural shape changes (new lang, new tier, new domain) | Anyone, whenever they need a helper or a small per-feature lookup |
| Is it a constant, a contract, or a flag? | Yes | Sometimes (small lookup maps are fine) |
| Is it a function with logic? | No | Yes |
| Is it tied to a single feature? | No — site-wide | Often (feature subfolders allowed) |
| Does it change with content? | No — content lives in `src/Content/` | No |

If the answer to "who edits this?" is "Zetun engineers when the project's tier or language list changes" — it's `Data/`. If it's "any contributor adding a feature" — it's `Utils/`.

The line is not infinitely sharp. `mandatory_Inp.ts` declares raw infrastructure constants (`langs_Config`) *and* derives small helpers from them on the same import (`supported_Url_Langs = langs_Config.supported_Langs.map(...)`). That's fine — derived helpers built from Data's own constants stay in Data so the file is self-contained. A helper that *uses* a Data constant but adds independent logic belongs in `Utils/`.

## The two subfolders

```
src/Data/
├── Common/          # always-on infrastructure
│   └── mandatory_Inp.ts       # langs, BCP codes, domain, demo metadata, analytics IDs
├── Dev/             # dev-only feature flags
│   └── dev_Config.ts          # boolean flags gating dev tools — all false in production
└── Og_Img/          # OG image generator inputs
    ├── og_Image_Config.ts     # template catalogue + per-collection defaults
    ├── og_Asset_Config.ts     # project-root-relative reusable image/SVG assets
    ├── og_Icon_Config.ts      # explicit icon choices for split-icon templates
    ├── og_Output_Config.js    # output format, folders, manifest paths
    └── og_Theme_Contract.js   # hex-token contract for the OG image renderer
```

- **`Common/`** holds infrastructure imported across the site by everyone — pages, components, layouts, Scripts, Utils. See [common.md](common.md) for what belongs (and what doesn't).
- **`Dev/`** holds toggles for in-development tools (font switchers, debug overlays, theme picker visibility). All flags must default to `false` before a production deploy. See [dev.md](dev.md) for the pattern and the "when not to use a Dev flag" rules.
- **`Og_Img/`** holds the project-owned inputs for the OG image manifest and external renderer: template selection, reusable assets, theme contract, output paths, and output format. See [og_Img.md](og_Img.md) for the boundary against `src/Scripts/OG_Images/`.

## Customer / tenant state — route out

`src/Data/` does NOT hold per-customer or per-tenant state. Multi-tenant Astro projects in this org typically delegate that to:

- A sibling **customer-config repo** (per-tenant TOML/YAML files, sops-encrypted secrets) — read by deploy tooling, not by the Astro project directly.
- A sibling **backend service repo** that owns the runtime configuration (SMTP credentials, rate limits, audit settings) and the schema for what each tenant must provide.

When you're working on a customer-facing concern that needs tenant-specific data (a contact form, a per-clinic price page, branded OG cards):

1. **Don't add a customer field to `src/Data/`** — Data is for site infrastructure, not per-tenant state.
2. **Check your project's `Project_Manag/Docs/`** for a doc that routes to the relevant external repo (customer-config, backend-service). If your project doesn't have such a doc yet, add one — the route belongs at the project level, not in this skill.
3. **For Partner specifically:** per-tenant configs live in the sibling `07_Customers/` repo; backend form config lives in `05_Contact_Form/Project_Manag/Docs/Setup/`. Partner's own `tenant_Config/<TenantID>/` is local-dev wiring to the backend, not the source of truth.

Per-dentist or per-customer **frontend content** (practice name, address, team list, services) is a different concern. In Partner that lives behind `src/Data/Site_Config/site_Config.ts` with editable leaves in the same folder; in other forks it may live in a local customer facade or `src/Content/` collection entries. Do not add it to `Data/Common/`.

## Naming

Follow the project-wide `improved_Camel_Snake` convention from the [coding skill](../../coding/SKILL.md):

- **Folders:** `PascalCase` — `Common/`, `Dev/`.
- **Files:** `snake_Case.ts` (or `.js` for legacy) — `mandatory_Inp.ts`, `dev_Config.ts`, `og_Image_Config.ts`, `og_Output_Config.js`.
- **Exports:** `noun_With_Capitals` for data (`langs_Config`, `site_Domain`, `umami_Config`), `SCREAMING_SNAKE` only for "this is a hard-coded constant the agent should not touch lightly" (`AUTHOR_TITLE_SUFFIX`, `DEV`).

No numeric prefixes on Data folders or files.

## Where does a new constant go?

| What you're adding | Where it goes |
|---|---|
| New supported language or BCP code | `Data/Common/mandatory_Inp.ts` — extend `langs_Config.supported_Langs` |
| New site-wide URL or domain | `Data/Common/mandatory_Inp.ts` |
| New analytics or third-party service ID | `Data/Common/mandatory_Inp.ts` (`umami_Config` etc.) |
| New OG image template default, renderer asset, theme token, or output path | `Data/Og_Img/` — see [og_Img.md](og_Img.md) |
| New dev-only feature toggle (font switcher, debug overlay) | `Data/Dev/dev_Config.ts` — default `false`, audit before prod |
| New helper function (URL builder, formatter, computation) | `Utils/Common/` — not Data |
| New per-feature lookup map (e.g. service slug → schema.org type) | `Utils/<Feature>/` — not Data |
| New per-tenant or per-customer field | Not here — see "Customer / tenant state" above |
| New theme token (color, radius, font) | `Styles/themes.css` via [Styles/tokens.md](../Styles/tokens.md), not Data |

## Deep-dives

- [common.md](common.md): what site-wide infrastructure belongs in `Data/Common/` (language config, site domain, analytics IDs, demo/preview metadata) versus what does NOT (helpers → Utils, per-feature lookup maps → Utils, per-tenant state → external repos, OG image contracts → `Data/Og_Img/`), the derived-helpers-stay-in-Data exception, worked examples from `mandatory_Inp.ts`, and the rename/add checklist. Use when adding a site-wide constant, extending the language list, or extracting a value from a page that's getting reused.
- [dev.md](dev.md): the dev-only feature-flag pattern (boolean flags exported from `dev_Config.ts`, imported anywhere the feature is conditionally rendered), why a separate `Dev/` subfolder (audit boundary — every flag in one file, easy to grep, easy to wipe before prod), the "all false before production" discipline, when NOT to use a Dev flag (real feature flags belong in env vars or build modes; per-user toggles belong in user settings). Use when adding a development tool, hiding a feature behind a flag, or auditing flags before a deploy.
- [og_Img.md](og_Img.md): the OG image data contract under `Data/Og_Img/`: template catalogue, per-collection defaults, reusable assets, split-icon choices, theme contract, output format, and project-root-relative paths. Use when configuring which generated OG card a page uses or when changing values consumed by the external renderer.
