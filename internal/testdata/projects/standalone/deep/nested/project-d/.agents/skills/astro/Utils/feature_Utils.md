# Utils/&lt;Feature&gt;

Feature-scoped Utils subfolders hold helpers and constants that only make sense within one feature of the site — the blog, the shop, an interactive map page, a course series. They sit alongside `Common/`, named after the feature concept (PascalCase), and contain only the helpers that 2+ files inside that feature share.

This is an *opt-in* pattern. A project does not need feature subfolders at all if it has no features that generate shared logic. Don't pre-create empty `<Feature>/` folders.

## When to lift code into a Feature subfolder

The trigger is **shared use within one feature**. Three rough thresholds:

1. **Two callers, one feature.** The same helper is needed in two files that both belong to the same feature (two blog page templates, two service-page components). Inline duplication has crossed into "should be shared."
2. **Helper feels too narrow for Common.** It would be misleading in `Common/` because nobody outside the feature would ever use it. (A `compute_Series_Nav()` only makes sense for blog series; nothing else on the site needs prev/next-in-series.)
3. **The feature has more than one such helper.** One isolated helper can sit inside `Common/` or stay inline. Two related helpers earn a folder.

If only one threshold fires, keep the code inline or in `Common/`. If two or three fire, lift to `Utils/<Feature>/`.

## Worked example — Jav_Web's `Blog/`

```
Utils/Blog/
├── blog_Linker.js          # prov_Blg_Link(file_Name, lang, collection_Key) — builds /<lang>/pub/<topic>/<slug>
└── series_Nav_Factory.js   # compute_Series_Nav({ collection_Key, entry, lang, ... }) — returns prev/next/total
```

Why this is a clean feature subfolder:

- Both files serve the blog and nothing else.
- They are *pure*: take inputs, return strings or nav objects. No DOM, no side-effects.
- They are *called by pages*: five series-entry page templates import `compute_Series_Nav`; the blog card component imports `prov_Blg_Link`.
- They satisfy the leaf-layer rule: `series_Nav_Factory` imports `blog_Linker` (same folder) and `astro:content`. Nothing pulls Utils → Scripts.

A new project adding a blog can follow the same shape: pick the feature name (`Blog/`), drop in the helpers as they accumulate, name them after what they do.

## Counter-example — Jav_Web's `Overview/`

Jav_Web has:

```
Utils/Overview/
├── AI/overview_Finder.js
├── Coding/overview_Finder.js
└── Digital_Solutions/overview_Finder.js
```

Each `overview_Finder.js` exports a `t_Combi` object — translation maps keyed by topic identifier. This is **not** a clean feature subfolder, for two reasons:

1. **Misnamed.** The files are called `overview_Finder.js` but contain no finder function — they are translation tables. Naming should follow what the file *does*; these should be `overview_Translations.js` or live in `Multi_Lang_Txts/` entirely.
2. **Wrong layer.** Translation strings belong in `src/Scripts/Multi_Lang_Txts/`, which exists for exactly this purpose. Putting them in Utils mixes concerns.

**Do not copy this structure** when bootstrapping a new project. It is Jav-specific (Jav has overview pages with custom translation lookups), and even within Jav it is arguably misplaced. If your project has analogous content, ask: is this translation data (→ `Multi_Lang_Txts/`) or is it a pure helper function (→ `Utils/<Feature>/`)? Pick one.

## Sub-subfolders inside `<Feature>/`

`Utils/<Feature>/` can have further subfolders when a feature has clearly distinguished sub-areas. Keep it shallow — at most one extra level. Beyond that, the feature is probably big enough that part of it belongs in `src/Scripts/` (build pipeline) or as its own page architecture.

Example shape (hypothetical, for a project with a complex shop feature):

```
Utils/Shop/
├── price_Format.ts
├── cart_Helpers.ts
└── Checkout/
    ├── address_Validation.ts
    └── shipping_Calculator.ts
```

If you find yourself going three levels deep, stop and consider whether the deepest level is actually pipeline code that belongs in `Scripts/`.

## Naming

- Folder: `PascalCase` named after the feature concept in English (e.g. `Blog/`, `Shop/`, `Booking/`), not after a German/local-language slug. See the org rule on English-first naming.
- Files: `snake_Case.ext`. Noun-first uppercase (`Blog_Linker.js`, `Cart_State.ts`) for data/feature names; verb-first lowercase (`compute_Series_Nav.js`, `format_Price.ts`) for action-named helpers.

## Cross-feature helpers

If a helper is needed by 2+ different features, it is no longer feature-scoped — lift it up into `Utils/Common/`. The reverse — pulling a `Common/` helper down into a single feature folder — is rare; do it only when the function genuinely turns out to be feature-specific and no longer used outside.

See [linker_Utils.md](./linker_Utils.md) for the universal Utils rules and the decision table.
