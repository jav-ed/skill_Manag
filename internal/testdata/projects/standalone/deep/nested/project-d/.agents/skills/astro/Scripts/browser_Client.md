# Scripts/Browser_Client

`src/Scripts/Browser_Client/` holds all **client-side JavaScript** — code that runs in the user's browser, not at build time. Theme switchers, table-of-contents observers, dropdown menus, animations, form handlers, map mounting. Anything that touches `document`, `window`, or fires on user interaction belongs here.

This is the most active folder during page load. It splits **by English page name** so a developer working on the contact page knows to look in `Contact/`, a developer working on the header looks in `Header/`. Cross-page concerns live in `Common/`.

## What lives here

Three shapes:

| Shape | Lives in | Examples |
|---|---|---|
| Page-specific interactions | `<EnglishPageName>/` | `Contact/contact.ts`, `Blog/animated_Toc.js`, `Header/navMenu.ts` |
| Cross-page interactions | `Common/` | Theme switcher, tooltip engine, lazy loaders |
| Feature-specific helpers (used internally by the above) | Same subfolder, sibling files | `Blog/toc_Svg.js`, `Blog/toc_Clip.js` |

## Internal structure

```
Browser_Client/
├── Common/                    # cross-page interactions — always present in larger projects
│   ├── theme_Changer.js
│   └── tooltip_Engine.js
├── Header/                    # one folder per English page concept
│   ├── navMenu.ts
│   └── mobile_Menu.ts
├── Blog/
│   ├── animated_Toc.js
│   ├── reader.js
│   ├── toc_Observer.js
│   ├── toc_Svg.js             # helper, imported by toc_Observer.js
│   └── toc_Clip.js            # helper, imported by toc_Observer.js
├── Contact/
│   ├── contact.ts
│   └── load_Map.ts            # lazy-loaded from contact page
└── Landing/, About/, …
```

The folder list is **per project**. A clinic site has `Contact/` and possibly `Karriere/` (English: `Careers/`); a personal site has `Landing/`, `CV/`, `Blog/`. Pick the page concept the developer thinks of when navigating.

## English page names

Subfolder names use **English**, even when the customer-facing route is in another language:

| Route the customer sees | Folder in code |
|---|---|
| `/kontakt/` (German) | `Browser_Client/Contact/` |
| `/karriere/` (German) | `Browser_Client/Careers/` |
| `/uber-uns/` (German) | `Browser_Client/About/` |

The route translation lives in the i18n layer (slugs, page filenames, translation tables). The code layer stays English so non-German-speaking developers can navigate it.

## Wiring into `.astro` files

Three patterns, picked by use case:

**Side-effect import** — eagerly load the script when the component renders:

```astro
<script>
  import "../Scripts/Browser_Client/Contact/contact";
</script>
```

The script runs as soon as it's loaded. Use this for setup that needs to happen on every page render of the component (form binding, observer attachment).

**Explicit `<script src>` tag** — Astro's script bundler handles it; emits a hashed bundle and a `<script>` tag in the HTML:

```astro
<script src="../../Scripts/Browser_Client/Blog/animated_Toc.js"></script>
```

Use for scripts that should be visible as separate bundles in network waterfall (TOC, reader settings) — easier to debug.

**Dynamic `await import()`** — lazy-load on user action, keeps the initial bundle small:

```astro
<script>
  btn.addEventListener("click", async () => {
    const { mountMap } = await import("../Scripts/Browser_Client/Contact/load_Map.ts");
    mountMap();
  });
</script>
```

Use for heavy modules that aren't needed until interaction (map, charts, complex animations).

## Internal imports

Inside `Browser_Client/`, files import each other via **relative** paths:

```js
// inside toc_Observer.js
import { create_Svg, draw_Svg } from "./toc_Svg.js";
import { update_Clip }            from "./toc_Clip.js";
```

This keeps a feature folder portable — copying `Blog/` to another project doesn't need any path rewriting.

Browser_Client may import from `Utils/Common/` (locale detection, formatters). It should NOT import from `Astro_Frontmatter/` (different runtime) or pages/components (wrong direction).

## What does NOT belong here

| Pattern | Where it goes |
|---|---|
| A function called inside `.astro` frontmatter | `Scripts/Astro_Frontmatter/` |
| A function that takes inputs and returns outputs with no DOM access | `Utils/` (might be useful in browser AND in build) |
| A remark/rehype plugin | `Scripts/Build/MD_Plugins/` |
| Translation strings | `Scripts/Multi_Lang_Txts/` |

The test: **does the code run in the user's browser, after the HTML loads?** If yes, it belongs here. If it runs during `astro build`, it doesn't.

## Naming

- Folders: `PascalCase`, English page concept (`Header/`, `Contact/`, `Blog/`, `Common/`).
- Files: `snake_Case.ext`. Action handlers verb-first lowercase (`theme_Changer.js`, `mount_Map.ts`), data/state objects noun-first uppercase (`Reader_Settings.ts`).
- TypeScript preferred for new code; legacy `.js` is fine where it exists.

## Add or rename — checklist

1. One page → `<EnglishPageName>/`. Cross-page concern → `Common/`. Feature-internal helper → same folder as the entry file, sibling.
2. Name after what the code *does* (`theme_Changer.js`), not which page calls it first (`Landing_Theme.js`).
3. Wire into `.astro` via the matching pattern: eager `import`, `<script src>`, or `await import()`.
4. On rename, grep `.astro` files and `<script src>` tags. `refac-cli` won't touch these.
5. If the script grows to multiple files (TOC, with observer / svg / clip helpers), keep them in the same page folder, not a sub-subfolder.
