# Utils

`src/Utils/` holds the **leaf-layer code** of an Astro project — shared data and pure helper functions that other layers (pages, components, layouts, the build pipeline) call into. It is the bottom of the dependency graph: anything may import from `Utils/`; `Utils/` should not import from `Scripts/`, pages, or components.

This folder typically stays small. Most projects need only a `Common/` subfolder for site-wide constants and helpers. Feature-scoped subfolders (e.g. `Blog/`, `Shop/`) appear naturally when a feature gathers more than one or two shared helpers.

## The core reason

Code in any non-trivial Astro project ends up needing three different kinds of things:

1. **Pipeline code** that runs during the build — schema definitions, frontmatter helpers, plugins. → `src/Scripts/` ([file_Structure.md](../Areas/file_Structure.md))
2. **Hydrated code** that runs in the browser — interaction handlers, animations. → `src/Scripts/Browser_Client/`
3. **Shared leaf code** that is just data or pure functions, callable from anywhere with no side-effects or environment assumptions. → **`src/Utils/`**

Utils exists so that the same constant (a language list, a contact email, a URL-building function) can be imported by a page template, a component, *and* a build-time script without anyone owning it more than anyone else. It is the org's answer to "where does this go when it doesn't belong to one place?"

## The leaf-layer rule

`Utils/` is the bottom of the import graph.

```
pages / components / layouts          ┐
                                      ├──► Utils/
src/Scripts/* (build, browser, etc.)  ┘
```

- Anything in `src/` MAY import from `Utils/`.
- `Utils/` SHOULD NOT import from `Scripts/`, `pages/`, `components/`, or `layouts/`.
- `Utils/` MAY import from `src/Data/` (raw config) if such a folder exists.

The Scripts-vs-Utils rule of thumb in [file_Structure.md](../Areas/file_Structure.md) — "if called *by* a page → Utils; if pipeline → Scripts" — is a useful first pass but **not strict**. In practice, Scripts code routinely imports Utils (e.g. an OG-image generator pulls in language tables). The strict rule is the leaf-layer one above.

## What lives in Utils — two kinds of content

Both belong; both are common.

**Data files.** Modules that export only constants and plain objects. Language tables, contact info, lookup maps. No functions, no side-effects.

**Pure helper functions.** Stateless functions: URL builders, formatters, deterministic computations. They take inputs and return outputs; they do not touch the DOM, the filesystem, or `astro:content` collections (collection access belongs in `Astro_Frontmatter/`, which builds the data that pages then consume).

If a file has both — a couple of constants and a small helper — that is fine. Split when the file passes ~150 lines or when the two halves clearly serve different consumers.

## Subfolder convention

Two patterns, both PascalCase:

| Subfolder | When it exists | Example |
|---|---|---|
| `Common/` | Always — every project will have site-wide constants and at least one shared helper | language tables, owner contact, locale detection |
| `<Feature>/` | When a feature accumulates more than ~2 shared helpers, lift them into a feature-named folder | `Blog/` for URL builders + series-nav factory |

`Common/` is the only universal subfolder. `<Feature>/` folders appear per project, named after the *concept* the helpers serve. Don't pre-create empty feature folders.

For deep-dives:

- [Common/](./common.md) — what site-wide content goes here, what doesn't (translation tables vs. language metadata, page-specific data vs. cross-cutting constants).
- [Feature subfolders](./feature_Utils.md) — when to lift a helper into its own `<Feature>/` folder, the worked Blog/ example, and an honest look at edge cases (Jav has an `Overview/` subfolder that is opinionated to its content model — your project may not need anything analogous).

## Naming

Follow the project-wide `improved_Camel_Snake` convention from the [coding skill](../../coding/languages/improved_Version.md):

- **Folders:** `PascalCase` — `Common/`, `Blog/`.
- **Files:** noun-first `Blog_Linker.js`, verb-first `compute_Series_Nav.js`. Use `.ts` for new code (matches modern Astro projects); legacy `.js` is fine when the project hasn't migrated.
- **Exports:** `noun_With_Capitals` for data (`SUPPORTED_LANGS`, `owner_Email`), `verb_Noun_Capitals()` for functions (`detect_Lang`, `compute_Series_Nav`).

No numeric prefixes on Utils folders or files.

## Where does a new file go?

| What you are building | Where it goes |
|---|---|
| A site-wide constant or lookup map (languages, contact, brand) | `Utils/Common/` |
| A pure helper used in 2+ unrelated places (URL builder, locale detector) | `Utils/Common/` |
| A helper specific to one feature and used in 2+ files within that feature | `Utils/<Feature>/` |
| A helper used in only one file | Keep it inline in that file; lift later if a second caller appears |
| A function that reads `astro:content` collections | `src/Scripts/Astro_Frontmatter/` — that is pipeline code, not Utils |
| A function that touches the DOM or `window` | `src/Scripts/Browser_Client/` — that is browser code, not Utils |
| Translation strings keyed by language | `src/Scripts/Multi_Lang_Txts/` — see [common.md](./common.md) for the distinction from language *metadata* |

## Add or rename — checklist

1. Pick `Common/` for cross-cutting, `<Feature>/` for feature-scoped, or stay inline if there's only one caller.
2. Name the file `snake_Case.ext` — noun-first uppercase, verb-first lowercase.
3. Import via bare `src/Utils/...` path from `.astro` frontmatter; relative path from another Utils file.
4. Do not add imports from `Scripts/`, pages, components, or layouts — if you need to, the function does not belong in Utils.
5. On rename, grep across `src/`, `astro.config.mjs`, `ec.config.mjs`, and `Project_Manag/Docs/` — `refac-cli` does not reliably update bare `src/` paths or `.astro` frontmatter.
