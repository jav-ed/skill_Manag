# Utils/Common

`Utils/Common/` is the one Utils subfolder that exists in every Astro project in this org. It holds the constants and helpers that any layer of the site might need — language metadata, owner contact, locale detection, brand-level lookups. Anything that does not belong to a single feature lives here.

## What belongs in Common

**Site-wide constants and lookup maps.** Things that describe the *site itself*, not its content. Examples from the org's repos:

```
Utils/Common/
├── helper.js       # SUPPORTED_LANGS, textDirections, lang_Local_Names
├── site_Owner.js   # owner_Email, owner_Phone
└── lang.ts         # detect_Lang(pathname), with_Lang(pathname, lang)
```

(Names like `helper.js` are historical and not great — `helper` says nothing about what is inside. Prefer descriptive names for new files: `language_Tables.ts`, `owner_Contact.ts`. Rename existing ones when convenient.)

**Pure cross-cutting helpers.** Functions used in 2+ unrelated places that do not belong to a single feature: locale detection from a URL path, theme-token resolver, debounce/throttle utilities, anything site-shape.

**Format conventions.** Date formatters, slugifiers, normalisers that the whole site agrees on. If only one feature uses it, it belongs in that feature's subfolder instead.

## What does NOT belong in Common

**Translation strings keyed by language.** UI labels like *"Read more"* in 5 languages belong in `src/Scripts/Multi_Lang_Txts/`, which exists precisely for that purpose. The distinction:

| Goes in `Utils/Common/` | Goes in `Scripts/Multi_Lang_Txts/` |
|---|---|
| Language *metadata*: which langs are supported, their text directions, their codes, their local-script names | Translation *strings*: the actual words shown to users in each language |

**Page-specific content.** If a constant is only used on one page, keep it on that page or in its content collection. Common is not a junk drawer for everything that is `export const`.

**Functions that touch `astro:content`.** Reading collections is build-time pipeline work — that belongs in `src/Scripts/Astro_Frontmatter/`.

**Functions that touch the DOM, `window`, or `document`.** Those are hydrated browser code — `src/Scripts/Browser_Client/`.

## File patterns

One file per concern. Keep files small: when a file exceeds ~150 lines or starts mixing concerns (language metadata next to contact info next to formatters), split.

Each file exports related members:

```ts
// language_Tables.ts
export const SUPPORTED_LANGS = ['en', 'de', 'fr', 'es', 'zh'];
export const textDirections = { en: 'ltr', ar: 'rtl', /* ... */ };
export const lang_Local_Names = { /* ... */ };
```

```ts
// lang.ts (locale detection from URL)
import { SUPPORTED_LANGS } from './language_Tables';
export type Lang = (typeof SUPPORTED_LANGS)[number];
export function detect_Lang(pathname: string): Lang { /* ... */ }
export function with_Lang(pathname: string, lang: Lang): string { /* ... */ }
```

Import these from anywhere with bare `src/` paths:

```ts
import { SUPPORTED_LANGS } from 'src/Utils/Common/language_Tables';
import { detect_Lang } from 'src/Utils/Common/lang';
```

## Honest note on the Jav_Web Common files

Jav_Web's `Common/helper.js` is the historical "everything in one file" file: it contains language tables AND directionality maps AND local-script names AND a `t_Translate` map that overlaps with the translation system. It works but is a smell. New projects should start with one file per concern from day one and avoid the helper-as-junk-drawer pattern. See [linker_Utils.md](./linker_Utils.md) for the broader rule on naming files by what they *do*, not by where they sit.
