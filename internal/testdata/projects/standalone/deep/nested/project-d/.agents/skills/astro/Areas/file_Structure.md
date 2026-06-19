# File Structure

Astro projects in this org organise non-template code into two top-level directories under `src/` — `Scripts/` for pipeline/infrastructure code and `Utils/` for leaf-layer shared data and helpers — plus a root `Code/` folder for manual CLI tooling that runs outside the Astro build. Reference implementations: `Partner/` (this repo) and `01_Jav_Web/` (the pattern source).

This file is the **src/ overview** and the home of the cross-cutting naming rules. Each of the three code directories has its own deep-dive doc — follow the links rather than expecting full coverage here.

Note on Jav_Web: it is the pattern source but **occasionally inconsistent** — it ships numeric prefixes on a few code folders (`0_Base/`, `2_Pages/` under Styles, `schema_Entries_5xx.ts` under Content_Schemas) and at the repo root uses lowercase `scripts/` instead of `Code/`. When Jav_Web's reality contradicts this doc, follow the doc. Newer projects deliberately strip those numerics and use `Code/` at the root.

## The three code locations

| Location | What lives here | Deep-dive |
|---|---|---|
| `src/Scripts/` | Pipeline/infrastructure — build plugins, frontmatter helpers, browser-boot scripts, content schemas, translations, OG image generation | [Scripts](../Scripts/linker_Scripts.md) |
| `src/Utils/` | Leaf-layer code — shared data and pure helpers, importable by anything | [Utils](../Utils/linker_Utils.md) |
| `Code/` (repo root) | Manual CLI tooling — diagram generators, doc generators. Run via `bun run <script>`; NOT part of `astro build` | this file (below) |

**The leaf-layer rule.** `Utils/` is the bottom of the import graph: pages, components, layouts, AND `Scripts/` may all import from `Utils/`, but `Utils/` does not import from any of them. The "if called by a page → Utils, if pipeline → Scripts" framing is a useful first pass but not strict — Scripts code (OG image generation, frontmatter helpers, translation validators) routinely imports Utils for shared data like language tables. See [Scripts](../Scripts/linker_Scripts.md) and [Utils](../Utils/linker_Utils.md) for the full split.

## Root `Code/` folder

Manual, one-off tooling that is NOT wired into `astro build`. Examples from current projects:

```
Code/
├── Mermaid/        # CLI wrapper for static Mermaid diagram rendering
└── shared/         # measurement helpers, YAML parser used by the CLI tools
```

Run via `bun run <script-name>` (registered in `package.json`). Differs from `src/Scripts/Build/`, which IS part of the Astro build pipeline. **Do not create a lowercase `scripts/` at the repo root** — the convention is capital `Code/`.

Sub-structure inside `Code/` is per project: one subfolder per tool. Shared helpers between tools go in `Code/shared/` (lowercase intentional — this folder mirrors a Node ecosystem convention for tool internals; the user-facing entry point folders are PascalCase).

## Naming conventions

Follow the project-wide `improved_Camel_Snake` convention from the [`coding` skill](../../coding/languages/improved_Version.md):

- **Folders:** PascalCase / underscore-separated, capitalised — `Browser_Client/`, `Astro_Frontmatter/`, `OG_Images/`, `Common/`.
- **Code files:** `snake_Case.ext` with noun-first uppercase / verb-first lowercase — `routing_Factory.ts`, `Blog_Linker.js`, `theme_Changer.js`, `minify_Html.js`.
- **Non-code files:** first letter lowercase, subsequent nouns capitalised — `doc_Start.md`, `linker_File_Structure.md`.

### Numeric prefixes — content yes, code no

| Context | Rule |
|---|---|
| Content collection folders (`500_Python_Series/`) | Keep numerics — the number is the stable collection key |
| Content entry files (`0001_No_Fear.mdx`) | Keep numerics — sort key for series ordering |
| Code folders (`Scripts/`, `Utils/`, `Browser_Client/`) | No numerics — folders identified by name, not position |
| Code files (`routing_Factory.ts`) | No numerics — file name is the identifier |

**The rule is recursive.** If a parent folder strips numerics, all its subfolders and files do too.

**Exception:** an externally-enforced execution order or identifier range that is not obvious from names alone justifies a numeric prefix — add a comment explaining why. Jav's `schema_Entries_5xx.ts` is an instance of this (the `5xx` encodes the collection's ID range).

### English in code, translated languages in slugs

All file, folder, function, variable, and type names use **English**, including page-name subfolders inside `Browser_Client/`, `Astro_Frontmatter/`, and `Multi_Lang_Txts/`. Customer-facing language (German, etc.) lives in slugs and translation files, handled at the i18n layer. This keeps the codebase legible to any developer regardless of the target market.

### Import paths in `.astro` files

Astro frontmatter uses bare `src/` paths (e.g. `import x from "src/Utils/Common/lang"`) and Vite aliases (`@components/`, `@layouts/`). Automated refactor tools (including `refac-cli`) do **not** reliably update bare `src/` paths or paths inside `.astro` frontmatter, `.mjs`, or `.mts` files. After a rename, grep manually:

```bash
grep -rn "old/path\|old_File_Name" src/ astro.config.mjs ec.config.mjs
```

## Where does a new file go?

High-level routing — for the full Scripts and Utils decision tables, see the respective deep-dives.

| What you are building | Where it goes | Deep-dive |
|---|---|---|
| A pipeline/infrastructure file (schema, plugin, frontmatter helper, browser JS, translation, OG glue) | `src/Scripts/` — pick the purpose subfolder | [Scripts](../Scripts/linker_Scripts.md) |
| A shared data file or pure helper called from anywhere | `src/Utils/` — `Common/` for site-wide, `<Feature>/` for feature-scoped | [Utils](../Utils/linker_Utils.md) |
| A manual CLI tool (NOT wired into `astro build`) | `Code/<Tool>/` at the repo root | this file (above) |
| A stylesheet | `src/Styles/` — see styling deep-dive | [Styles](../Styles/linker_Styles.md) |
| An image or font file | `src/Assets/` — see assets deep-dive | [Assets](./assets.md) |

## Adding a new file — checklist

1. Decide which top-level location fits — pipeline (`Scripts/`), leaf (`Utils/`), or manual tool (root `Code/`).
2. Follow the relevant deep-dive's decision table for the specific subfolder.
3. Name the file `snake_Case.ext` — noun-first uppercase, verb-first lowercase.
4. No numeric prefix unless an externally-enforced identifier or order requires it (then comment why).
5. Import using bare `src/` path from `.astro` frontmatter; if it's client-side JS, hook it in via `<script>`.

## Renaming or moving — checklist

1. Grep for the old name across `src/`, root config files (`astro.config.mjs`, `ec.config.mjs`), `package.json`, and `Project_Manag/Docs/`.
2. Update import paths in `.astro`, `.mjs`, `.mts`, and `.ts` files — `refac-cli` won't catch these.
3. Update doc comments that reference the file by name.
4. If you moved an entry in `package.json` scripts, update the script command.
5. Boot the dev server (`bun run dev`) and exercise the affected page before considering the rename done.
